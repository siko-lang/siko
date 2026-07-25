# Allocation Flow — Expanded Notes

Commentary on `allocation_flow.md`, checked against the current compiler and
revised after discussion (§3, §4, §6, §7 record the resolutions). The short
version: the design is sound and the closure-lowering machinery really does
carry most of it. The load-bearing clarifications: lifetimes are per **scope
instance** (which makes loops fall out of the base rule), **only escape
forces heap — physical placement never does**, and "allocation follows the
value" applies to the directly-returned root, not to pointers escaping
*inside* other values. The practical prerequisites (transparent-container
profiles for the unsafe std layer, GC interaction, decision consumption in
lower-enums) determine whether the analysis produces wins in practice or
classifies everything as Dunno.

## 1. Where it sits in the pipeline

The analysis needs to know which positions are pointer-carrying, and that is
decided by `value-types` (`siko/Common/src/ValueTypes.sk`), which runs in the
backend pipeline after `lower-builtins`. The decision is consumed by
`lower-enums`, which is where `ExprKind.Alloc` is born today
(`LowerEnums.sk:208`). So the natural slot is:

```text
lower-builtins → value-types → [allocation-flow] → lower-ifs → ... → lower-enums
```

This placement is fortunate for another reason: closures, coroutines, tuples,
implicits and trait objects are all already lowered. In particular:

- **No dynamic dispatch problem.** Closure calls are already enum dispatch
  over a known lambda set; coroutine frames are ordinary structs. The one
  remaining indirect construct is `FnPtr`, and closure lowering already shows
  how to handle it (`fn_ptr_edges`): treat the function-pointer type's
  interior vars as the target's boundary. Only *unknown* function pointers
  (FFI-supplied) are black holes.
- **Implicit ctx structs are ordinary args.** A pointer escaping into an
  implicit context is just a pointer flowing into a struct passed to a call —
  no special case.

One ordering wrinkle: `lower-ifs` / `lower-basic-block` restructure control
flow after this point. The analysis as described is flow-insensitive (sets
per slot, not per program point), so it does not care — but the *loop rule*
in §4 needs back-edge information, which is easier to read off before
basic-block lowering flattens everything into gotos. Another argument for
running right after value-types.

## 2. What directionality changes mechanically

The closure machinery is built on `UnionFind` (`Flow.sk:265`): `unify` is
symmetric, classes are cheap, and publishing a profile is "canonicalize each
boundary var to its class root" (`publish_profile`). Directionality replaces
each of these:

| closure lowering | allocation flow |
|---|---|
| `UnionFind.union(a, b)` | directed edge `a → b` |
| class root per boundary var | reachability relation between boundary vars |
| profile = signature with repeated vars | profile = signature vars + edge list |
| instantiate = replay class merges | instantiate = replay edges |

Concretely:

- **Profile projection is transitive closure restricted to boundary vars.**
  `?arg → ?local1 → ?local2 → ?result` becomes `?arg → ?result` by computing
  reachability and keeping only boundary-to-boundary pairs. Worst case this
  is quadratic in boundary size — fine, boundaries are small. Inside an SCC,
  cross-function edges are wired directly (the analogue of the direct-wiring
  branch in `function_call_edges`, Flow.sk:813) and each member's profile is
  reachability over the whole SCC graph restricted to that member's boundary.
- **The cross-product fallback survives.** `FlowCtx.unify` falls back to a
  full cross product when the var lists don't zip (transmute-like constructs,
  Flow.sk:360). The directed version is the same: all-pairs edges. Lossy but
  safe, since edges only ever *add* pointers to sets.
- **Per-construct rules split by direction.** `field_access_edges` becomes a
  slot → out edge; `ctor_edges` becomes arg → slot; `named_pattern_edges`
  becomes slot → binder. The rules themselves — the DataTables position pools
  mapping fields/variant items to type-arg positions — transfer unchanged.
- **Bodyless functions.** `bodyless_signature_edges` merges all boundary vars
  into one class. The directed analogue is edges both ways between every
  boundary pair *plus* an edge to BlackHole — an extern function may retain
  what it is given. This default makes every extern a heap-forcer, which is
  correct and is also why §6 (precise std models) matters so much.

### Datatype generalization is analysis-only

Closure lowering *duplicates* datatypes per closure content because content
changes layout (`GeneralizeDataTypes.sk`). Allocation provenance changes no
layout: a `Holder` holding a stack `Foo` and one holding a heap `Foo` are the
same type. So `Holder<?value>` generalization threads provenance vars through
`Named` type args exactly like closure lowering does — same `DataTables`,
same signature-var plumbing in profiles (`FunctionProfile.arg_types` already
carries `TypeDef`s with vars) — but no duplication, no rewrite of the
program's types. The whole apparatus is scaffolding that exists only during
the pass. This makes the pass considerably cheaper and less invasive than
closure lowering even though it reuses its shape.

## 3. Immutability: a precision win, not a soundness prerequisite

(Revised after discussion — an earlier draft called this invariant
load-bearing for soundness. That was overstated: mutation is just another
edge in the flow graph. A store `h.value = foo` into a pre-existing object is
an edge from `pFoo` into a slot the storing function did not create — a slot
of unknown lifetime — so the storing function itself decides `pFoo` heap.
Per-function decisions survive; nothing structural breaks.)

What immutability actually buys is precision. Post-lowering safe Siko has
**no field assignment**: `ExprKind` has no store-into-field node; mutation
exists only behind unsafe `Std.Ptr` operations and atomics. So a datatype
slot is filled exactly once, at the container's construction site, in the
same function that evaluates the ctor call, and the ways a local allocation
`pFoo` leaves its function reduce to:

1. it (or a fresh container transitively holding it) flows to the result
   slot;
2. it is passed *down* into a callee argument;
3. it reaches a black hole.

Case 2 is where the profiles earn their keep: passing a pointer down the
stack is intrinsically lifetime-safe (the callee frame dies first), *unless*
the callee's profile shows the arg reaching its own result / a black hole —
and then the instantiated links turn case 2 into case 1 or 3 in the caller's
graph automatically. Escape-into-caller-owned-storage — the case that drives
heap verdicts everywhere in imperative escape analyses — does not occur in
safe code, so far fewer sites go heap and profiles stay small. A structural
advantage, not a prerequisite.

## 4. Lifetime granularity: scope instances, not frames

(Revised after discussion — an earlier draft added a separate "cardinality"
rule for loops. It is unnecessary; what it was groping at is a definition.)

"Every slot containing `pFoo` dies within its owner's lifetime" must be read
at **scope-instance** granularity, not frame granularity. One iteration of a
loop body is one scope instance; a binding declared in the body dies at the
back edge. With that definition the base rule already covers loops:

```siko
fn f() {
    let all = Vec.new();
    for x in xs {
        let foo = Foo(x);     // per-iteration scope instance
        all.push(foo);        // ?slot of `all` outlives foo's instance → heap
    }
}
```

A pointer from iteration *i* that is live at iteration *i+1* has necessarily
reached a slot declared in an outer scope (`all` here, or a `prev` local
above the loop) — a longer-lived slot, heap by the existing rule. No extra
clause. Conversely:

- a pointer dead by the back edge reuses **one entry-block alloca** per site
  across iterations (allocas belong in the entry block — a non-entry alloca
  inside a loop grows the stack dynamically; reuse is sound because at most
  one instance per site is live at a time within a frame, and recursion gets
  fresh frames);
- aliasing is free: pushing the *same* `foo` into N slots is fine as long as
  every slot is bounded by `foo`'s scope. This is not Rust; sharing costs
  nothing.

Recursive datatypes need no special case: a per-iteration `List` node whose
`next` holds the previous node stores iteration *i*'s pointer into iteration
*i+1*'s live slot → outer-scope slot → heap.

The practical consequence: the analysis must model per-scope-instance
lifetimes, which is easiest while scope structure still exists — another
argument for running before `lower-ifs` / `lower-basic-block` flatten
everything into gotos.

## 5. "Allocation follows the value" — true for the root, not the tree

The demand-driven story (`some_func` returns `Foo` by value, `bar` chooses
storage) is right, but only for the *directly returned* value. Consider:

```siko
fn make() -> Holder {
    Holder(Foo())      // pFoo → ?value slot of the result
}
```

`Holder` can be returned by value. `Foo` cannot: `Holder.value` is a pointer
field, so `Foo` needs a stable address *inside `make`*, before the caller's
demand is knowable. Options, in increasing order of ambition:

1. **v1 (recommended): only the root is demand-driven.** An allocation that
   reaches the result slot *as the top-level value* is returned by value; an
   allocation that reaches the result *inside a container slot* is heap.
   Simple, sound, already captures the common `fn new() -> T` constructor
   pattern that motivates the whole exercise.
2. **Freshness profiles.** Extend the profile with "the result's `?value`
   slot contains only allocations fresh in this call." The caller could then
   provide storage for the whole tree (an sret-like out-pointer per slot).
   This is real "functional-language escape analysis" and a large project;
   note it and defer.
3. **Dual ctor specialization.** Generate value-returning and heap-returning
   variants per function and pick at call sites. Code-size blowup for
   marginal wins; skip.

Note the ABI consequence even for v1: today every non-value type is a pointer
everywhere, uniformly. By-value return of a "heap type" means the function's
return convention now depends on analysis results — the signature itself
changes (or an sret out-param is threaded). That is a per-function rewrite in
the same spirit as what implicit propagation already does to signatures, but
it must be stamped consistently: the decision has to live on the function
instance, and every call site must agree with it.

## 6. Containment comes for free from slot vars

(Revised after discussion — an earlier draft proposed an explicit must-heap
fixpoint over a function's own sites. It is redundant: the generalized slot
vars already carry containment.)

```siko
let f = Foo();          // pFoo
let h = Holder(f);      // pHolder, ?value = {pFoo}
```

If `pHolder` escapes (flows to the result slot), the result type's `?value`
var is linked to the holder's `?value` var by ordinary type-arg wiring — so
`pFoo` appears in an escaping slot through plain reachability. Nothing about
`pHolder`'s *decision* needs to feed back into `pFoo`'s; escape propagates
through the flow graph on its own.

The sharper principle that fell out of the discussion: **physical placement
never forces anything; only escape does.** A container that is heap-allocated
(for whatever reason — forced by a profile annotation, too big, whatever) but
never escapes its scope holds slots nobody can read after the scope dies;
stack contents inside it are fine. "Reaches a longer-lived slot" is about the
slot's *reachability lifetime*, not about what kind of memory the container
physically sits in. (Answering the "will (may?)" question: *may* — heap-ness
of a wrapper only forces heap-ness of its contents when the wrapper
escapes, and then the slot vars already say so.)

Two hand-written transitivity rules do remain:

- **Black holes swallow contents.** A value entering a black hole hands over
  everything reachable from it: link *all* slot vars of the flowing value's
  type to BlackHole, not just the top-level pointer.
- **Forced-heap profile slots.** Profiles should be able to say "whatever
  lands in this boundary slot is heap, period" (a Heap sink element in the
  lattice, weaker than BlackHole — it forces allocation but doesn't destroy
  knowledge).

This also answers a question the original leaves open: argument contents.
Per-function analysis never needs to know what callers put in `?arg` — the
sets can carry an opaque "caller pointer" element or nothing at all — because
decisions are only ever made about the function's *own* sites.

## 7. Std primitives decide whether this works at all

`bodyless_signature_edges`' conservative default (everything merges, plus
BlackHole) applied to the `Std.Ptr` layer means: every `Vec.push`, every
`Map.insert`, every `String` operation black-holes its arguments →
practically every interesting value in a real program is Dunno → the analysis
returns "heap everywhere," matching today at great expense.

Closure lowering hit the same wall and solved it twice: the hand-written
precise model for `resume` (`resume_signature_edges` — "resume is a
constructor from a data-flow standpoint") and the `@closure_alias` annotation
for functions that launder pointers through opaque types. Allocation flow
needs the equivalent, and the discussion produced a cheap v1 for the big
case:

- **Transparent containers.** A `Vec` forces nothing: it is a `?slot` such
  that whatever goes in can come out. That is exactly the conservative
  bodyless default *minus the BlackHole* — merge all boundary content vars
  bidirectionally, assert nothing else. One annotation
  (`@alloc_transparent` or similar) on the unsafe collection primitives
  gives every container a usable profile. Lossy (a Map's keys and values
  merge) but safe; finer custom profiles per collection can come later. Note
  the corollary from §6: the physically-heap `gc_allocate_array` buffer is
  irrelevant to decisions — a non-escaping Vec can hold stack pointers, and
  one Vec can simultaneously hold stack and heap pointers, because the slot
  tracks flow, not placement. Aliasing is allowed; this is not Rust.
- `gc_allocate_array` / `gc_allocate` (`std/Common/src/Ptr.sk`): a *source*
  of fresh heap pointers — `UnknownPtr`-like but known-heap. Importantly not
  a black hole for anything else.
- `copy`, `offset_of`: alias-preserving; the `@closure_alias` shape-matching
  trick transfers directly.
- Thread spawn (`GC_pthread_create` bindings): captured closure struct →
  BlackHole (transitively, per §6), non-negotiable — cross-thread lifetimes
  are unknowable here.

With transparent containers the wins are no longer limited to temporaries:
locally-consumed collections and everything staged through them become
candidates, on top of intermediate structs, `Option`/`Result` wrappers,
iterator states, and constructor-then-consume patterns.

## 8. GC interaction

The runtime GC is Boehm (`bdw-gc`, linked in `Compiler/src/Passes.sk`), and
conservative collection shapes the story favorably:

- **Dangling pointers:** a *reachable* heap slot holding a stack pointer
  past the stack object's scope would be fatal; the escape tracking (§6)
  exists to rule that out. Unreachable heap memory holding stale stack
  pointers is harmless — nobody, including the collector, dereferences it.
  Worth a debug-mode checker regardless.
- **Root scanning:** conservative stack scanning sees stack objects' pointer
  fields for free — stack-allocating aggregates needs zero GC-side work.
  Every by-value temporary is also one less object to trace and one less
  false-retention candidate.
- **Heap buffers holding stack pointers:** legitimate under transparent
  containers (§7); Boehm inherently ignores addresses outside its own heap.
  A hypothetical precise collector would need two accommodations: tolerate
  non-heap addresses in pointer slots, and register pointer-carrying stack
  aggregates as roots with layout — with a natural fast path for aggregates
  whose fields are all value types (nothing to register).

## 9. Consuming the decisions

- Stamp each ctor call site with `stack | heap | dunno` — a new field on
  `FunctionCallInfo`, defaulting to heap, which is exactly today's behavior.
- `lower-enums` today synthesizes one ctor per type whose body does `Alloc` +
  field stores. Stack sites instead need the fields stored into
  caller-provided storage: a *placement* ctor taking a `*T` (the entry-block
  alloca), sharing the field-store body with the heap ctor.
- By-value returns (§5) additionally rewrite the function signature; that's
  the invasive part and can ship after the purely-local wins (stack
  placement of locally-dead allocations needs no ABI change at all — start
  there).
- LLVM backend: allocas in the entry block, full stop.

## 10. Cost and validation

- The symbolic phase is closure-lowering-shaped: per-SCC graphs, discarded
  after publishing. Closure lowering's `Statistics.sk` counter scaffolding
  should be cloned from day one — profile sizes, edge counts, and crucially
  a `stack / heap / dunno` site histogram, because §7 predicts the first
  runs will be Dunno-dominated and the histogram tells you which missing
  std model to write next.
- Directed reachability is worklist-cheap at these sizes; the risk is not
  time but *profile edge blowup* on functions with fat boundaries (many
  args × many result slots). Transitive-reduce profiles before publishing.
- Validation is unusually easy: every decision "stack" is an optimization of
  a working program. Ship behind a flag, run the full regression suite and
  the self-host bootstrap with it on, diff `--re` outputs. A debug mode that
  heap-allocates anyway but *tags* would-be-stack objects and asserts no
  heap→tagged pointer at GC time gives a dynamic soundness check before
  trusting the static one.

## 11. Summary (updated after discussion)

| # | Point | Resolution |
|---|---|---|
| 1 | Loops / allocation count | no extra rule — lifetimes are per *scope instance*, and the base rule subsumes it; one entry-block alloca per site |
| 2 | Containment vs heap decisions | automatic via slot-var linking; physical placement never forces, only escape does; profiles get a forced-heap slot element |
| 3 | By-value return over-promised | v1: root value only; interior escapes → heap |
| 4 | Mutation / immutability | mutation is just extra edges; immutability is a precision win, not a soundness prerequisite |
| 5 | Extern/std defaults will drown the analysis | transparent-container annotation = conservative merge minus BlackHole; custom profiles later |
| 6 | GC not mentioned | free under Boehm (conservative scan); a precise collector would need non-heap-address tolerance + stack-aggregate roots |
| 7 | Decision consumption unspecified | call-site stamp + placement ctors in lower-enums |
