# Allocation Flow: Implementation Investigation

This note maps the allocation-flow idea onto the current compiler. It separates
verified compiler facts, a concrete implementation shape, and decisions that
still need policy.

## Conclusion

The analysis fits the compiler well, but it is not a small variant of
`ClosureLowering`.

The reusable architecture is:

```text
datatype SCC shapes
        ↓
function SCC profiles (symbolic directed links)
        ↓
cheap per-function origin propagation
        ↓
constructor/call placement decisions
```

The main additions beyond closure inference are:

1. value copies and reference aliases have different flow rules;
2. lifetimes must be attached to local slots;
3. unsafe pointer primitives need trusted summaries;
4. caller-chosen return storage needs a `fresh result` fact in addition to
   ordinary input-to-output links;
5. `lower-enums` must gain placement construction.

The recommended first milestone is local stack placement of constructor roots.
Caller-chosen returns should be the next milestone, built on the same analysis.

## 1. Verified compiler state

### Pass position

The backend pipeline in `siko/Compiler/src/Passes.sk` is:

```text
lower-builtins
→ value-types
→ lower-ifs
→ lower-basic-block
→ lower-pointers
→ lower-enums
→ llvm
```

The analysis belongs immediately after `value-types`:

```text
lower-builtins
→ value-types
→ allocation-flow
→ lower-ifs
→ lower-basic-block
→ lower-pointers
→ lower-enums
```

At this point:

- closures, coroutines and tuples have been lowered;
- functions are monomorphic closure-specialized instances;
- struct/variant constructors are still `FunctionCall` nodes with
  `StructCtor` / `VariantCtor` kind;
- no `ExprKind.Alloc` expression has been emitted yet;
- structured `Block`, `If`, `Match`, and `Loop` nodes still exist;
- `lower-builtins` may already have inserted scalar `Label` / `Goto` /
  `Switch` nodes.

This is the last convenient point for assigning lexical scope IDs.

### What `value-types` means today

`siko/Common/src/ValueTypes.sk` sets one global `value_type` bit per datatype:

- empty structs and explicitly `@value_type` structs are required value types;
- enums are value-type candidates unless layout cycles prevent it;
- ordinary non-empty structs are heap types by default.

Therefore the empty `Foo` used in the conceptual example is already returned by
value today. An implementation test must use a non-empty struct:

```siko
struct Foo {
    value: Int
}
```

Allocation flow must not change `value_type`. It chooses storage for one
instance of a non-value type; it does not change the representation of that
type everywhere.

### Where allocation happens

`siko/Common/src/LowerPointers.sk` later makes the global representation
explicit:

```text
value type      Named(Foo)
heap type       Ptr(Named(Foo))
```

`siko/Common/src/LowerEnums.sk` then generates one constructor function per
struct/variant. A non-value constructor currently:

```text
Alloc(Foo) → initialize fields → return *Foo
```

`ExprKind.Alloc` becomes `siko_malloc(sizeof(Foo))` in
`siko/Compiler/src/LLVM/Lower.sk`.

The LLVM backend already puts every `Declare`/`Let` local in the function entry
block (`FunctionLowerer.add_local`). This is useful: if `lower-enums` injects an
inline `Declare(Foo)`, the backend automatically creates one entry-block alloca
per syntactic site. A loop reuses it; recursion gets a fresh function frame.

### Mutation exists in the safe subset

The analysis cannot assume immutability. `Statement.Assign` supports field
stores, and safe std functions use them:

```siko
// Std.Vec.push
v.size = v.size + 1;
*offset_of(v.data, v.size) = item;
```

Stores must therefore produce pointer-flow edges. Immutability may improve
precision, but it is not an IR invariant.

## 2. Do not put analysis variables into real `TypeDef`s

Closure lowering can use temporary `TypeVar`s because it already rewrites and
duplicates closure-dependent types. Allocation provenance does not affect
layout or type identity.

There is also a mechanical problem: `NamedInfo` has spare type arguments after
monomorphization, but `Ptr`, `VoidPtr`, arrays, and other pointer-bearing forms
have nowhere to store an inference variable.

Use a separate analysis shape:

```text
FlowShape(TypeDef)
    root pointer slot?          // the object/reference itself
    content slots[]             // pointer-bearing fields reachable through it
```

Examples:

```text
Int                         []
Foo (non-value struct)      [Foo.root, Foo.contents...]
InlinePair[Foo, Int]        [Pair.foo.root, Pair.foo.contents...]
*Foo                        [ptr.root, pointee.contents...]
```

This keeps the HIR unchanged during profile construction and handles explicit
raw pointers without extending every AST type.

### Share representation classification

The shape builder must agree exactly with `LowerPointers.is_heap`, including
its builtin scalar exceptions. That logic should be extracted into a shared
representation helper rather than copied. A disagreement would either miss a
real pointer or invent one where the backend emits an inline scalar.

## 3. Datatype SCC shapes

Build a finite pool of content slots for every datatype SCC, following
`ClosureLowering.GeneralizeDataTypes` and its `DataTables`.

For each datatype record:

```text
DataShape {
    pool_size
    field projections: field index → pool positions
    variant projections: variant/item index → pool positions
}
```

A use of a non-value `Named(Foo)` gets:

```text
[fresh root] + [fresh instance of Foo's content pool]
```

A value-type use gets only the content pool. Recursive references within one
datatype SCC share pool positions conservatively, preventing an infinite
shape.

Example:

```siko
struct Holder {
    value: Foo
}
```

```text
Holder instance
├── Holder.root
└── Holder.value.root  ── field projection for `value`
```

The analysis scaffolding is discarded after decisions; datatypes are not
duplicated or rewritten.

## 4. Directed flow still needs alias edges

Pointer contents propagate directionally:

```text
source → destination
```

But copying a non-value Siko object copies a reference. Both variables then
observe the same field storage. This requires two concepts:

```text
value transfer       src → dst
shared storage       src ↔ dst    // represented as two directed edges
```

No union-find is required; pointer-set propagation remains directed. The paired
edges merely encode observable aliasing.

Each flattened shape slot should carry a copy mode:

```text
Copy     add src → dst
Alias    add src → dst and dst → src
```

For a non-value `Foo`:

```text
Foo.root       Copy
Foo.contents   Alias
```

For an inline struct containing a `Foo`, the field's `Foo.root` is copied, but
the pointee contents alias. A shared `copy_value(type, src_slots, dst_slots)`
helper should be used for:

- `let` and assignment;
- function arguments and results;
- constructor fields;
- field access;
- pattern destructuring;
- dereference/index loads and stores.

This distinction is necessary for field mutation. Treating every slot as
one-way misses writes through aliases; treating every slot as an equivalence
class recreates closure inference and loses useful directionality.

## 5. Function profiles

Fork the architecture of:

- `ClosureLowering/InferProfiles/CollectDeps.sk`;
- `ClosureLowering/InferProfiles.sk`;
- `ClosureLowering/InferProfiles/Flow.sk`.

Do not initially refactor the closure implementation into a generic framework;
the construct rules and graph semantics differ too much.

### Boundary vocabulary

Give every pointer-bearing position in arguments and the result a stable
boundary slot:

```text
Arg(0).root
Arg(0).field(1).root
Arg(1).root
Result.root
Result.field(0).root
```

A profile contains:

```text
Profile {
    edges: BoundarySlot → BoundarySlot
    heap_sinks: BoundarySlot
    black_hole_sinks: BoundarySlot
    unknown_outputs: BoundarySlot
}
```

`Heap` is a known escape. `BlackHole` means destination unknown. They should be
separate so diagnostics and future optimizations retain the distinction.

The profile describes dependence on caller-provided pointers. It cannot by
itself describe pointers created inside the callee. After concrete analysis,
publish a second, small summary:

```text
OutcomeSummary {
    boundary_sources: BoundarySlot → {KnownHeap, Unknown}
    fresh_result: Option[FreshResult]
}
```

For example, if a callee creates `Foo()` and stores it into an argument's
field, that `Foo` is heap in v1 and the argument-content boundary gains a
`KnownHeap` source. If the callee can return a fresh movable `Foo`, the result
gets `FreshResult` instead.

### SCC processing

Build the static call graph and process SCCs in dependency order:

```text
same SCC call       wire callee boundary graph directly
finished SCC call   instantiate the callee profile at the call site
```

Publishing a profile means computing directed reachability from the function's
boundary slots and retaining only:

- boundary-to-boundary reachability;
- reachable `Heap` / `BlackHole` sinks;
- unknown boundary outputs.

Locals disappear:

```text
Arg(0) → local7 → local12 → Result

publishes as

Arg(0) → Result
```

Publishing all reachable boundary pairs is the simplest correct first version.
Profile compression can later collapse boundary SCCs and remove redundant
edges.

### Function arguments need entry separation

A parameter variable is local and can be reassigned without changing the
caller's variable:

```siko
fn f(a: Foo, b: Foo) {
    a = b; // does not reassign the caller's variable
}
```

Model an argument boundary and a separate local parameter value:

```text
ArgIn.root → local_a.root
ArgIn.contents ↔ local_a.contents
```

This keeps root reassignment local while still exposing mutation of the object
passed by reference.

## 6. Per-function concrete analysis

After every symbolic profile exists, process functions one at a time. No
program-wide allocation graph or allocation-ID set is built.

Each local flow node holds a set of origins:

```text
Origin =
    LocalSite(site_id)
    Argument(arg_index)
    KnownHeap
    Unknown
```

Example:

```text
slot 3 = {LocalSite(0), Unknown}
```

`Unknown` does not contaminate `LocalSite(0)`. They are independent set
members.

At a static call, replay only the callee's compact profile between caller-local
slots. Callee locals never enter the graph.

### Constructor sites

Only non-value struct/enum constructors seed a local root origin:

```text
Foo(1) result root = {LocalSite(0)}
```

Value-type constructors only transfer their field origins.

### Scope IDs

During the structured walk, assign:

```text
ScopeId {
    parent
    is_loop_body
}
```

Record the declaring scope of every local and constructor site. Content nodes
instead retain the local or receiver root that owns their storage.
`VariableName` is already unique within a function.

There are two kinds of storage lifetime:

```text
local root / inline field    owned by a lexical local
field behind a reference    owned by the receiver's root origins
```

The second case is essential. In:

```siko
outer.value = Foo(1);
```

the lifetime of `outer.value` is the lifetime of the object referenced by
`outer`, not the scope containing this assignment. Every reference-content
node must therefore retain its owning root node.

The conservative rule from the design is:

```text
destination scope is the same as or below the site's home scope → local
destination is a strict lexical ancestor                    → escapes home
result/argument content/Heap sink                           → escapes frame
BlackHole                                                   → unknown
```

For a pointer stored behind a reference, apply that rule to every possible
owner origin:

```text
owner is Argument / KnownHeap       the stored pointer escapes this frame
owner is Unknown                    Dunno
owner is LocalSite(other)           compare against `other`'s home scope
```

If `other` is later given heap placement, pointers stored inside it must also
be given heap placement. Compute escape/unknown reason flags as a small
monotone fixed point within this function, starting with local candidates and
only adding reasons. Derive `Stack | Heap | Dunno` afterward. This is still
per-function; no program-wide origin graph is retained.

A loop body's scope is a new instance on every iteration. Flow into an
outer-scope slot therefore prevents reuse of one stack slot while an older
iteration's object remains reachable.

There is a precision choice here: LLVM storage lives for the whole frame, so a
non-loop inner-block object could safely have its storage lifetime promoted to
the frame. The lexical rule rejects that case but is simpler and safe. Keep it
for v1; relax it only with explicit back-edge/liveness reasoning.

Functions containing compiler-generated gotos that cross structured scopes
should conservatively fall back to heap until the scope model accounts for
them.

### Verdict and placement are different

Keep reasons, then derive code generation:

```text
Analysis reason                 Placement
local only                      Stack
known longer-lived destination  Heap
BlackHole only                  Heap fallback, report Dunno
definite escape + BlackHole     Heap, with both reasons retained
```

This avoids forcing `Dunno` to erase a separately proven escape.

## 7. The extra fact needed for caller-chosen returns

Ordinary links answer:

```text
if an input pointer enters X, can it reach Y?
```

They do not say that a function creates a fresh movable result. Without that
fact, `bar` cannot treat `some_func()` as a new allocation site.

```siko
struct Foo {
    value: Int
}

fn some_func() -> Foo {
    Foo(1)
}

fn bar() {
    let f = some_func();
}
```

After profiles are complete, process functions in call-graph order and publish
the `OutcomeSummary` described above. A result gets `FreshResult` when:

- the result root contains exactly one local origin;
- no argument, unknown, or known-heap origin can reach the result root;
- that local origin does not also reach result contents, argument contents,
  `Heap`, or `BlackHole`;
- the producer site is not in a repeatable loop scope;
- the function is not recursive, address-taken, an entry point, extern-facing,
  or a C callback in v1.

This is deliberately restrictive. Multiple mutually exclusive fresh
constructor sites can be supported later with control-flow-sensitive proof.

When a caller invokes a `FreshResult` function, that call becomes a new local
origin:

```text
some_func() result = {LocalSite(call_site)}
```

The caller can classify it as stack, heap, or another forwarded fresh result.
This keeps concrete analysis per-function while allowing freshness to propagate
up call chains. At every other call, replay the flow profile and add the
callee's `KnownHeap`/`Unknown` boundary sources. Recursive SCCs require a
monotone outcome-summary fixed point; v1 should simply disallow
`FreshResult` within them.

## 8. Persisting decisions through later passes

The result must survive `lower-ifs`, `lower-basic-block`, and `lower-pointers`.
A side map keyed only by source location is unsafe: generated calls commonly
share representative locations.

The robust choice is a field on `FunctionCallInfo`:

```text
allocation:
    Default
    Stack
    Heap
    Dunno
    ForwardResult
```

It applies to constructor calls and calls to `FreshResult` functions.

This is mechanically noisy: positional `FunctionCallInfo(...)` patterns must be
updated, and auto-derived walkers need an identity instance for the new enum.
The compiler has several such walkers. The noise is preferable to a fragile
location/ordinal side table.

`ForwardResult` should be emitted only after the containing function passes the
full freshness test. A later scan can recover the set of fresh-return functions
from these markers without adding a field to every `FunctionDef`.

## 9. Consuming decisions in `lower-enums`

### Placement constructors

For a non-value struct, change the generated constructor shape from:

```text
fn Foo.ctor(fields...) -> *Foo {
    let out = Alloc(Foo);
    initialize(out, fields...);
    out
}
```

to:

```text
fn Foo.ctor(out: *Foo, fields...) -> *Foo {
    initialize(out, fields...);
    out
}
```

Then expand a call according to its decision after `lower-pointers`:

```text
Stack:
    declare storage: Foo
    let f: *Foo = Foo.ctor(&storage, fields...)

Heap / Dunno:
    let storage: *Foo = Alloc(Foo)
    let f: *Foo = Foo.ctor(storage, fields...)

ForwardResult:
    let f: *Foo = Foo.ctor(function_out, fields...)
```

`LowerEnums.lower_stmts` currently emits one statement per input statement. It
must be refactored to allow a constructor/call statement to expand into several
statements. The normalizer guarantees call arguments are atomic, so moving the
physical allocation before the call does not reorder argument effects.

The generated inline `Declare(Foo)` is inserted after `lower-pointers`.
Consequently it remains an inline `Named(Foo)` local rather than being wrapped
back into `Ptr(Foo)`. LLVM's existing local lowering supplies the entry-block
alloca; no new LLVM operation is needed.

### Fresh-return functions: destination passing

Do not implement this by physically returning and copying an aggregate. Rewrite
fresh-return functions to take caller-provided storage:

```text
fn some_func(out: *Foo) -> *Foo
```

The original pointer return can remain, so most of the body is unchanged. Its
`ForwardResult` producer constructs directly into `out`.

Callers provide:

- `&local_storage` for `Stack`;
- `Alloc(Foo)` for `Heap`/`Dunno`;
- their own `out` parameter for `ForwardResult`.

This is sret-like destination passing and preserves stable address identity
during construction. That matters for safe wrappers such as mutex
initialization, although precise FFI models are required before those examples
can optimize.

This requires a signature rewrite as well as call-site rewriting:
`FunctionDef.args` gains the destination parameter before bodies are lowered,
then calls receive the selected destination. `lower-enums` already walks every
function and ordinary call, so it can host this rewrite, although a small
placement-lowering submodule would keep it separate from enum-layout logic.

Disqualifying address-taken/callback functions in v1 avoids needing two ABIs or
adapter wrappers.

### Heap enums

A heap variant constructor currently allocates:

1. the enum container;
2. a separate payload struct when the variant has payload.

Root-only v1 can place the container in caller storage while leaving the
payload allocation unchanged. That is sound and matches the rule that only the
direct result root is demand-driven. Eliminating the payload allocation would
require variant-specific caller storage or a redesigned inline payload
representation and should be separate work.

## 10. Unsafe and std models

Treating every unsafe function body or extern as a black hole is sound but will
make normal `Vec` use unoptimizable. `Vec` itself does not need a magical
transparent annotation: its body is present and its field stores can be
analyzed. The raw primitives underneath it need models.

Minimum trusted models:

| Primitive | Allocation-flow model |
|---|---|
| `Std.Ptr.offset_of` | input root → result root; pointee contents alias |
| `Std.Ptr.copy` | source pointee contents → destination pointee contents |
| `allocate*` / `gc_allocate*` | result root gets `KnownHeap`; empty contents |
| null constructors | empty origin set |
| unknown extern | every input/content → `BlackHole`; outputs get `Unknown` |
| thread handoff | transferred contents → `Heap` or `BlackHole` |

`Transmute`, unknown function-pointer calls, `AddrOf` escaping through unknown
code, and atomic/raw operations default to conservative sinks.

These models must be trusted. A user-spoofable annotation that claims
`noescape` could create dangling stack pointers. For v1, key models by exact
compiler-known std qnames in an `AllocationFlow.Intrinsics` module. A later
annotation mechanism must be restricted to trusted packages.

The existing `@closure_alias` implementation is a useful pattern but not the
right allocation semantics: `offset_of`, `copy`, allocation, and thread
handoff all need different directed models.

## 11. Suggested modules and touch points

```text
siko/Common/src/AllocationFlow/
    Pass.sk             orchestration
    Shape.sk            datatype SCC pools and slot shapes
    Profile.sk          boundary slots, edges and sinks
    InferProfiles.sk    function SCC driver and projection
    Flow.sk             construct rules and call instantiation
    Analyze.sk          per-function origins and site-verdict fixed point
    Outcome.sk          post-analysis boundary sources and FreshResult
    Intrinsics.sk       trusted unsafe/std models
    Debug.sk            profile/decision dumps
    Statistics.sk       edge, profile and verdict counters
```

Existing files that need changes:

- `siko/Compiler/src/Passes.sk`: insert the pass after `value-types`;
- `siko/Common/src/LowerPointers.sk`: share heap/value representation logic;
- `siko/Common/src/AST/Expr.sk`: persist allocation decisions on calls;
- positional call destructuring and derived walkers: carry the new field;
- `siko/Common/src/LowerEnums.sk`: statement expansion, placement constructors,
  destination parameters, enum root placement;
- `siko/Common/src/Config/Config.sk` and `siko/Compiler/src/CLI.sk`: feature
  flag and debug/profile/stat flags;
- `siko/LSP/src/main.sk`: initialize any new config fields.

No LLVM change is required for local stack placement.

## 12. Implementation order

### Milestone 0: observation only

- Build shapes and symbolic profiles.
- Dump profiles and graph statistics.
- Run on the full test suite and compiler bootstrap.
- Add no placement decisions.

This exposes profile blowups and missing std models without risking codegen.

### Milestone 1: local non-value roots

- Seed only direct struct/variant constructor roots.
- Compute per-function sets and `Stack | Heap | Dunno`.
- Initially optimize non-loop, non-goto struct sites.
- Teach `lower-enums` caller storage plus placement construction.
- Keep fresh-result calls and enum payloads on the current heap path.

This validates the core safety rule with the smallest ABI surface.

### Milestone 2: containers and loops

- Add precise field-store, dereference, index and pattern rules.
- Add the trusted `Std.Ptr` models.
- Enable loop sites when no origin reaches an outer-scope slot.
- Enable root placement for heap enums while retaining heap payloads.

### Milestone 3: caller-chosen result storage

- Track argument/unknown origins.
- Prove restrictive `FreshResult`.
- Treat fresh calls as local sites in callers.
- Publish and consume per-function outcome summaries.
- Add destination parameters and `ForwardResult`.
- Keep recursive/address-taken/callback cases on the heap ABI.

### Later precision work

- mutually exclusive fresh result sites;
- frame-lifetime promotion for non-loop inner scopes;
- recursive fresh-result SCCs;
- adapter ABIs for address-taken fresh functions;
- caller storage for enum payloads;
- profile compression and bitsets.

## 13. Tests and measurements

Add pass-output tests similar to
`test/success/std/closure_inference_pass`:

```text
local ctor                         Stack
ctor returned as root              FreshResult / caller decision
ctor stored inside returned value  inner allocation Heap
field assignment to outer object   Heap
local Vec.push + consume            Stack after Ptr models
returned Vec containing local ptr  Heap
loop-local dead each iteration      Stack
loop value stored outside loop      Heap
transmute/unknown extern input       Dunno
Unknown sharing a slot with pFoo     pFoo remains independently decidable
function SCC                         finite directed profile
```

Also test `--pass lower-enums`:

```text
Stack site  → inline Declare, no root Alloc
Heap site   → Alloc + placement ctor
Forward     → enclosing out pointer
```

Counters needed from the first run:

```text
datatype slots
graph edges
profile edges before/after projection
per-function local origins
Stack / Heap / Dunno / FreshResult
top BlackHole-producing qnames
```

A separate codegen policy may reject very large stack objects even when the
analysis proves them stack-safe. No size threshold is chosen here; safety and
placement policy should remain separate.

## 14. Decisions still open

1. **Metadata cost:** accept a typed field on `FunctionCallInfo` and the
   mechanical walker updates, or use a more fragile side table.
2. **Initial lifetime rule:** strict lexical scope, as designed, versus whole
   frame except across loop back-edges.
3. **Unsafe model mechanism:** hardcoded trusted qnames first, or a restricted
   internal annotation system immediately.
4. **First enum scope:** defer all heap enums, or optimize only their root
   container while preserving payload allocation.
5. **Stack-size policy:** always honor `StackSafe`, or apply a later layout-size
   threshold.

None of these blocks the observation-only profile implementation. The
load-bearing soundness points are trusted unsafe models, alias-aware copies,
scope/back-edge handling, and destination passing for fresh results.
