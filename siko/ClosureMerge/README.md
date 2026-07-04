# ClosureMerge

`Siko.ClosureMerge` is the data-flow driven closure lowering pass. It is
enabled by `--lower-closure-v2` and is called from the normal
`lower-closures` driver pass.

The short version:

1. Mark every closure-carrying type position with a fresh closure infer
   variable.
2. Analyze how those variables flow through functions.
3. Starting from `Main.main`, resolve each variable to the concrete set of
   closure instances that can reach it.
4. Rewrite the program so every closure slot is represented by an enum for
   exactly that concrete set.

This pass is a replacement for the old by-signature closure lowering strategy.
The old strategy creates one closure enum for each function type signature,
for example one enum for every `fn(Int) -> Bool` closure in the whole program.
That is simple, but it merges closures that have the same type even when their
values can never reach the same runtime slot.

ClosureMerge keeps the same final representation shape — closure values become
enums, dynamic calls become handler calls — but chooses the enums by data flow
instead of by signature alone.

## Motivation

Closures with the same function type do not necessarily belong to the same
runtime domain.

For example, two unrelated libraries may both create a `fn() -> ()` closure.
The old closure lowering pass puts both lambdas into the same enum just because
their signatures match. Any dynamic call of a `fn() -> ()` then dispatches
through a handler that contains arms for both lambdas, even if a given call site
can only ever receive one of them.

That over-merging has several costs:

- It creates larger closure enums and larger dispatch handlers than needed.
- It weakens later analyses, because unrelated lambdas now appear to be
  possible callees of the same dynamic call.
- It interacts badly with coroutine lowering. If one lambda in a signature-wide
  enum is or calls a coroutine, the shared handler can become a coroutine path
  for every other lambda with that signature.
- It hides useful specialization opportunities. A helper called with one
  closure set in one place and a different closure set elsewhere should be
  duplicated, just like generic code is duplicated by monomorphization.

ClosureMerge fixes this by treating "which closures can be here" as another
specialization axis. Type equality is still required: a `fn(Int) -> Bool` and a
`fn() -> ()` can never share an enum. But equal signatures are no longer enough
to merge. Two lambdas share an enum only when the analysis finds that their
closure values can reach the same closure slot.

## Entry Points And Wiring

The main entry point is:

```siko
pub fn lower_closures_v2(program: Program) {
    generalize_datatypes(program);
    generalize_functions(program);
    let profiles = infer_profiles(program);
    let r = resolve(program, profiles);
    rewrite_program(r, program);
}
```

This lives in `Lower.sk`.

Driver behavior:

- `--lower-closure-v2` makes the `lower-closures` pass call
  `ClosureMergeLower.lower_closures_v2`.
- Without that flag, the compiler uses the older `Siko.ClosureLowering` pass.
- `--debug-closure` inserts a separate diagnostic pass before closure
  lowering. It runs ClosureMerge on a cloned program and dumps the generalized
  program, profiles, resolver output, datatype instances, and closure enums.

## Core Terms

### Closure Infer Variable

A closure infer variable is a `TypeVar` attached to a closure-carrying type
position. It does not mean a source-level type parameter. It means "the concrete
set of closure instances that can inhabit this slot".

Examples of slots:

- a direct `fn(A) -> B` field,
- a datatype type argument that represents a closure-bearing field inside that
  datatype,
- a function argument, result, yield type, expression type, or pattern type that
  contains closure-bearing structure.

### Flow Node

`FlowNode(owner, var)` is the analysis identity of a closure infer variable.
Function-local variables are numbered per function, so the owner function is
part of the key.

### Closure Instance

A closure instance is a lambda plus the closure contents of its captures.

The resolver stores these as integer ids (`c0`, `c1`, ...). Each instance has:

- the lambda function name,
- one capture set per flattened closure-carrying capture slot.

This is more precise than just "lambda name". The same lambda can be a
different closure instance when it captures different closure contents.

The interning identity is name-truncated: capture contents are reduced to
sorted lambda-name lists, one level deep. That keeps the instance universe
finite when closures capture closures of their own lineage (middleware
chains, CPS-style recursion). The actual capture contents are instance-id
sets stored in the intern table and grow monotonically.

### Function Instance

A function instance is a function reached with a concrete tuple of closure sets
on its boundary variables. This mirrors ordinary monomorphization, except the
key is closure content instead of type arguments.

If the same function is reached with different closure-set tuples, ClosureMerge
creates different function instances.

### Concrete Set

A concrete set is a sorted list of closure instance ids. Each distinct concrete
set used as a closure representation becomes a closure enum during rewriting.
Non-empty sets become enums with one variant per closure instance; an empty set
would become an enum with no variants.

## Phase A: Generalize Datatypes

Implemented in `GeneralizeDataTypes.sk`.

This phase rewrites datatype definitions so closure-bearing fields are expressed
through fresh closure infer variables. It only looks at datatype field types.
It does not inspect function bodies, call sites, or lambda creation sites.

The pass processes datatype dependency SCCs bottom-up:

- If a datatype has a direct closure field, the field's `ClosureInfo` gets a
  fresh `closure_infer_var`.
- If a datatype refers to another already-generalized datatype, the reference is
  instantiated with fresh variables matching the referenced datatype's closure
  arity.
- If datatypes are mutually recursive, the whole SCC shares one variable pool.
  The pass first discovers the pool, then runs again in finalize mode to patch
  intra-SCC references to that final pool.

Conceptually:

```siko
struct Box {
    f: fn(Int) -> Bool
}
```

becomes:

```siko
struct Box[?0] {
    f: fn(Int) -> Bool/?0
}
```

The concrete enum for `?0` is not known yet.

## Phase A.5: Generalize Functions

Implemented in `GeneralizeFunctions.sk`.

After datatypes have published their closure arities, this phase walks every
function and tags every closure slot in:

- argument types,
- result types,
- yield types,
- expression type annotations,
- pattern type annotations.

Variable numbering restarts per function. A `?0` in one function is unrelated
to `?0` in another function; profiles use `FlowNode(owner, var)` to keep them
distinct.

This phase also instantiates bare references to generalized datatypes. If a
function mentions `Box` and `Box` has one closure infer parameter, the function
type occurrence becomes a fresh `Box[?n]`.

The important invariant established here is positional consistency: two
structurally related types produce closure variable lists of the same length in
deterministic order. Later phases rely on that when they zip variables between
an argument and a parameter, a field and a receiver, or a result and a call
expression.

## Phase B: Infer Profiles

Implemented by `InferProfiles.sk` and the `InferProfiles/` subdirectory.

This phase builds a data-flow profile for every function. A profile summarizes
how closure content can move between that function's boundary variables:

- closure variables in arguments,
- closure variables in result type,
- closure variables in yield type.

The analysis is SCC-based over the static call graph. Dependencies include
ordinary function calls, function pointer references, and closure creation sites
because lambdas are functions too.

### Flow Model

The core flow model is a union-find over `FlowNode`s.

Siko values can alias mutable structure, so this pass models closure-slot flow
as equivalence rather than one-way subset flow. If two positions can refer to
the same closure-carrying storage, they are unified. Lambda literals are stored
as seeds on the equivalence class that receives the created closure.

The result is a partition of closure slots, with each class optionally seeded
by known lambda literals.

### Per-Construct Rules

The hand-written flow walker in `InferProfiles/Flow.sk` handles the constructs
where parent and child type positions need to be related:

- variable use unifies the variable's environment nodes with the expression
  result nodes,
- lets and assignments unify right-hand side slots with the binding or left
  side,
- blocks, returns, yields, `if`, `match`, tuples, indexing, deref, addr-of, and
  transmute propagate closure variables through their result types,
- struct and variant constructors zip argument slots into datatype pool
  positions,
- field access and pattern destructuring zip datatype pool positions back out,
- static calls record a `CallRecord` and either wire directly to an in-SCC
  callee or instantiate an already-finished out-of-SCC profile,
- dynamic calls record an `ApplyRecord` for concrete resolution,
- closure creation records a `CreationRecord` and seeds the closure value class
  with the lambda,
- inside a `@closure_alias` function, signature types with the same shape
  (closure vars erased) alias each other, and a `sizeof(T)` aliases the
  signature type with the same pointer-peeled shape — this is how
  `allocate`'s `malloc(sizeof(T))` stays in sync with its `*T` result even
  though the size travels as a bare `U64`.

Bodyless extern or builtin functions have no body to walk, so their profiles
are synthesized from the signature alone: conservative by default (every
boundary variable lands in one class), precise for known models such as
`resume`. With a profile in place they take the same path as every other
callee — call sites instantiate the profile and record a `CallRecord`, and
concrete resolution creates instances for them like any other function.

### Profiles

A profile contains:

- `groups`: interesting equivalence classes over the function's boundary vars,
- `applies`: unresolved dynamic call sites,
- `creations`: unresolved closure creation sites and their capture sources.

Only boundary variables are exposed. Internal expression and local variables are
not part of the public profile. That keeps profiles reusable at call sites.

Dynamic calls and closure creations are deliberately left as residual records.
The symbolic profile phase does not know which concrete closure instances will
reach a dynamic call. Phase C resolves those residuals after actual concrete
sets are known.

## Phase C: Resolve Concrete Closure Sets

Implemented in `Resolve.sk`.

This phase starts from `Main.main` with empty closure inputs and discovers every
reachable function instance and closure instance.

It maintains two interned universes:

- closure instances: lambda plus capture contents,
- function instances: function name plus concrete input-set tuple.

For each function, the resolver computes and caches `WalkData`: the local
union-find classes, apply records, creation records, static call records,
boundary variables, and named datatype occurrences. This depends only on the
function body, so all concrete instances of the same function share it.

### Instance Processing

For one function instance, resolution repeatedly applies four rules until no
set grows:

1. Seed boundary classes from the instance input tuple.
2. Process closure creations. A creation site contributes its pinned closure
   instance id to the closure value class, and its capture classes feed the
   closure instance's capture sets. The capture sets also flow back into the
   creation site's classes: captures alias caller memory, so contents added
   by the lambda's execution (or by another creation site of the same
   instance) are visible through the caller's value too.
3. Process static calls. The caller-side sets determine the callee input tuple,
   which interns or reuses a callee function instance. The callee's resolved
   boundary sets can also flow back into the caller through mutation or returned
   closure content.
4. Process dynamic calls. The concrete callee set tells the resolver which
   closure instances may be called. Each member becomes a call to that lambda
   instance, with capture sets, argument sets, and result sets wired into the
   lambda boundary.

### Fixpoint Structure

Closure instance identity depends on capture contents, but capture contents are
also discovered by executing the fixpoint. To avoid minting identities from
half-converged data, resolution uses an outer and inner loop:

- The inner loop grows sets for the currently pinned creation identities until
  stable.
- The outer loop derives fresh creation-site identities from the converged sets.
  If any pin changes, computed sets are reset and the inner loop runs again.

Interned identities survive resets; computed class sets and capture contents do
not. This is the key invariant of the resolver: closure instance identities are
derived only from converged data, and every instance that executed in a round is
eligible for correction.

## Phase D: Rewrite Program

Implemented in `Lower.sk`.

After resolution, the pass rewrites the program to concrete closure
representations.

### Type Rewriting

Every tagged `TypeDef.Fn(ClosureInfo(..., Some(var)))` becomes the enum type
for the concrete set resolved for `var`.

Datatype references are instantiated by concrete closure-set tuples. The
lowerer emits one concrete datatype definition per live tuple. Arity-zero
datatypes are still emitted through this path so the output program contains
only the rewritten live world.

### Function Rewriting

Every reachable function instance is emitted as a function definition.

Instance naming is stable for the common case:

- the first live instance of a function keeps the base name through
  `with_closure_instance(0)`,
- later instances get higher closure-instance indices.

Calls, constructors, patterns, and function pointer references are retargeted to
the exact concrete function or datatype instance found by the resolver.

Bodyless functions (externs and builtins) go through the same machinery:
each live instance is emitted with its signature types substituted — there
is simply no body to rewrite. Bodyless functions never referenced from live
code are dropped.

### Closure Creation

`CreateClosure(lambda, captures...)` becomes a variant constructor call for the
enum representing the closure value's concrete set.

The selected variant is the pinned closure instance for that creation site.
Payload fields are the rewritten types of the lambda's captured values.

### Dynamic Call

`DynamicCall(callee, args...)` becomes a call to a generated closure handler.

Handlers are keyed by:

- the callee enum id,
- the concrete sets of closure-carrying argument slots,
- the concrete sets of closure-carrying result slots.

Each handler matches on the closure enum and dispatches each variant to the
exact lambda function instance for that call context.

### Closure Enums

Each distinct concrete set used as a closure representation becomes one enum.
Each closure instance in the set becomes one variant. Variant payloads are the
rewritten capture types for that closure instance.

Payload computation is lazy because captures can refer to enums that are also
being generated, including recursive self-capturing closure structures.

## Debug Output

Use `--debug-closure` to inspect the analysis without mutating the real driver
program at that point. The debug pass runs on a clone and prints:

- the generalized program,
- per-function data-flow profiles,
- closure instances,
- reachable function instances,
- concrete datatype instances,
- closure enums.

`InferProfiles/Debug.sk` owns the profile/class formatting.
`Resolve.dump` owns the concrete instance dump.

## Module Map

- `Lower.sk`: pass entry point, concrete rewriting, enum generation, handler
  generation.
- `GeneralizeDataTypes.sk`: datatype SCC walk and closure infer variable pools.
- `GeneralizeFunctions.sk`: function signature/body type generalization.
- `InferProfiles.sk`: profile phase driver and profile summarization.
- `InferProfiles/Profile.sk`: shared profile data structures.
- `InferProfiles/CollectDeps.sk`: static call graph dependency collection.
- `InferProfiles/Flow.sk`: closure variable collection, slot tables,
  union-find, flow rules, profile instantiation, and closure alias support.
- `InferProfiles/Debug.sk`: debug formatting for flow classes and profiles.
- `Resolve.sk`: concrete closure instance and function instance resolver.

## Important Invariants

- Closure infer variables are local to their owner function unless they are
  datatype pool variables.
- Structurally related types must expose closure variables in deterministic
  positional order.
- Datatype generalization must process dependencies before dependents, and must
  treat mutually recursive datatypes as one SCC.
- Profiles expose only boundary variables. Internal variables must not leak into
  a caller-facing profile.
- Dynamic calls and closure creations are residual in Phase B and concrete in
  Phase C.
- Resolver identities must be derived from converged sets, not mid-pass partial
  sets.
- Only instances reachable from `Main.main` should be emitted.
- Rewriting must use the same input-tuple derivation as resolution; otherwise a
  retargeted call can point at a function instance the resolver never created.

## Relationship To Coroutine Lowering

ClosureMerge must run before coroutine lowering. It duplicates functions and
splits closure handlers according to actual closure flow. Coroutine lowering
then sees the duplicated call graph, which can distinguish a coroutine-reaching
copy of a function from a non-coroutine copy of the same source function.

Running coroutine classification before this split would reintroduce the
signature-wide over-merging that ClosureMerge is designed to remove.

Two contracts between the passes are worth spelling out:

- ClosureMerge leaves the mono name args of duplicated definitions stale: an
  instance's name is an opaque identity, not a description of its types.
  Coroutine lowering therefore never reconstructs names from types; it
  resolves the `CoResult` enum by variant payload shape and `resume` defs by
  argument type, synthesizing them (under uniquified fabricated names) only
  when the program has none.
- The target of an explicit co-create is a coroutine by construction. The
  precise split can produce a copy of a co-created function that never
  reaches a yield; classification must still treat it as a coroutine (it
  runs to completion on the first resume).
