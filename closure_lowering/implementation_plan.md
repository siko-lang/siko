# Implementation Plan: Data-Flow Driven Closure Merge

This document maps the design in *Data-Flow Driven Closure Merge* onto the
actual compiler (`/Volumes/workspace/git/siko/siko`, self-hosted Siko, not
Rust). It is a design-phase document only — no code is written here.

## Existing precedent this plan builds on

Two existing passes already establish the pattern this plan follows
throughout: **derive-based traversal, hand-written code only at interception
points, graph algorithms kept separate from tree walking.**

- `ClosureLowering/Pass.sk` — the pass this plan partially replaces. It
  declares a local `trait Rewrite[T] { fn rewrite(t: T, ctx: ClosureLowerer)
  -> T }` and ~45 one-line `instance Rewrite[X] = auto(Map, ...)` declarations
  (lines 47–126) covering every AST node type. Only three places get
  hand-written bodies because they intercept specific structure: the
  `TypeDef` instance (lines 59–87, intercepts `TypeDef.Fn` to allocate/reuse a
  closure enum), `ctx.rewrite_expr` (lines 171–234, intercepts
  `ExprKind.CreateClosure` and `ExprKind.DynamicCall`), and the generic
  `Option[T]`/`Vec[T]` container instances. Everything else recurses purely
  through the derive. Handler/enum generation (`generate_enum`,
  `generate_handler`, lines 239–352) is plain data construction, not
  traversal.
- `CoroutineLowering/FunctionGroupBuilder.sk` — the `Fold` counterpart. A
  local `trait Collect[T] { fn walk(t: T, c: CallCollector) }`, `auto(Fold,
  ...)` for ~40 node types, one hand-written instance
  (`Collect[E.FunctionCallInfo]`, lines 98–109) that records call-graph edges
  while still recursing into children via `walk(...)`. The resulting
  `Map[QualifiedName, Vec[QualifiedName]]` graph feeds `SCC.compute` directly
  (`build_groups`, lines 183–218), and `build_coroutine_map` (lines 117–159)
  consumes the SCCs in the bottom-up order Tarjan already guarantees
  (`Utils/SCC.sk`: "SCCs come out in reverse topological order — callees
  before callers").

**`Fold`/`Map` are compiler-builtin auto-derived instance bodies**, not a
separate macro crate: `Resolver/Autoderive/{Map,Fold}.sk` synthesize a
`FunctionDef` AST (a match over each variant/field) at resolve time; wiring
is in `Resolver/Resolver.sk:814-872`. `Map` = structure-preserving `T -> T`
transform; `Fold` = structure-consuming `T -> ()` visit. A trailing `=`
(`auto(Map, =)`) means "identity, don't recurse" (leaf/opaque types like
`Int`, `QualifiedName`); a trailing `!` means "unreachable if ever called"
(node kinds the pass should never encounter).

**Distinction to hold onto throughout this plan:** `Fold`/`Map` cover *AST
tree traversal only* (`Expr`, `TypeDef`, `Pattern`, `Statement`, ...). The
graph algorithms in the dataflow design — SCC, fixpoint saturation, memoized
worklist duplication — are not tree walks; they're iteration over
`Map`/`Vec` graph structures, exactly like the existing `Utils/SCC.sk` and
`Monomorphizer/Mono.sk`'s queue-based worklist. Those remain ordinary
iterative Siko code. "No manual tree walking" constrains the parts that
touch AST node types, not graph fixpoint loops.

## Module layout

New package `Siko.ClosureMerge`, replacing Phase 1 (enum/handler assignment)
of `ClosureLowering`. Phases 2–4 of the existing pass are reused, not
rewritten (see "Net effect on ClosureLowering/Pass.sk" below).

### `ClosureMerge/FlowVar.sk`

A fresh id type for flow-graph nodes, used both for Phase A's `α`s and
Phase B's per-function locals/temporaries/lambda-creation-sites. Mirrors the
existing `TypeVar.Var(Int)` (`AST/Types.sk`) / `VarAllocator` pattern already
used elsewhere in the compiler for fresh-variable allocation.

### `ClosureMerge/PhaseA_Variablize.sk` — datatype closure-field variablization

- **Graph:** copy the shape of `Monomorphizer/DataPreprocessor.sk:38-64` —
  scan every struct/enum field type for named-type dependencies
  (`collect_direct_deps`-style), build `Map[QualifiedName, Vec[QualifiedName]]`,
  call `SCC.compute`. Bottom-up SCC order (already guaranteed by
  `Utils/SCC.sk`) processes dependencies before dependents, and mutually
  recursive datatypes fall out as a single SCC automatically — no separate
  handling needed.
- **Rewrite:** a `Variablize[T]` trait, `auto(Map, ...)` for every node type,
  hand-written only for the `TypeDef` instance. It intercepts `TypeDef.Fn`
  (and anything transitively containing one, reached via the derive's normal
  recursion into nested fields) and replaces it with `TypeDef.Var(TypeVar.Var(fresh))`.
  This is structurally the same interception `ClosureLowering/Pass.sk:59-87`
  already performs for `TypeDef.Fn` — just emitting a fresh var instead of an
  enum reference. `TypeDef.Var` is already a first-class representation in
  `AST/Types.sk` (used for ordinary generics pre-monomorphization), so Phase
  A introduces **no new AST node** — it reuses the existing variable
  representation in a context where the program is otherwise fully
  monomorphized.

### `ClosureMerge/PhaseB_Profile.sk` — function data-flow profiles

- **Call graph + SCC:** reuse `CoroutineLowering/FunctionGroupBuilder.build_groups`'s
  shape directly. Note there are already three near-identical copies of this
  call-graph-SCC pattern in the codebase (`FunctionGroupBuilder.sk`,
  `PurityCheck/GroupBuilder.sk`, and the datatype-graph variant in
  `DataPreprocessor.sk`) — this is a reasonable point to extract a shared
  `Utils/CallGraph.sk` helper rather than add a fourth copy, but that's a
  refactor decision, not a blocker.
- **B.1 — edge collection:** a `Collect[T]` trait, `auto(Fold, ...)` for
  every node type except the handful of `ExprKind` cases that create flow
  edges:
  - Lambda literal → seed node `{this lambda}`.
  - `let`/assignment → edge `y → x`.
  - Struct/variant construction & destructuring → edges matched structurally
    against Phase A's `α` positions for that datatype.
  - Calls → **outside current SCC**: instantiate the finished callee
    profile fresh (rename all its variables, unify at the call's actual
    args/result). **Inside current SCC** (incl. self-calls): wire directly
    onto the callee's real signature variables (no copy) — this is what
    introduces cycles for recursive/mutually-recursive functions.

  This is structurally identical to `Collect[E.FunctionCallInfo]` in
  `FunctionGroupBuilder.sk:98-109` — a handful of hand-written match arms
  that record edges into a context struct, with every other node type
  falling through to the derived `auto(Fold)` recursion.
- **B.2 — saturation** and **B.3 — splicing/elimination**: plain graph
  algorithms over `Map[FlowVar, Set[FlowVar]]` (propagate-to-fixpoint, then
  eliminate non-boundary nodes by splicing incoming/outgoing edge pairs).
  Not tree walking — worklist/fixpoint code over a graph structure, living
  in a new `Utils/FlowGraph.sk` so both Phase B and Phase C can call it.
- **Output:** one profile per function (a set of links between its own
  input/output flow variables), stored for reuse when later SCCs are
  processed (outside-SCC call case above) and again in Phase C.

### `ClosureMerge/PhaseC_Resolve.sk` — concrete resolution and duplication

- **Walk from `main`, memoized worklist:** same shape as
  `Monomorphizer/Mono.sk`'s `fn_queue`/`fn_mono_key` (`Mono.sk:90-115`) —
  swap "type-args tuple" for "concrete lambda-set tuple" as the memo key.
  Mint fresh identity for each duplicated instantiation via a new `BaseName`
  variant (e.g. `ClosureInstance(index)`) alongside the existing
  `ClosureEnum`/`ClosureVariant`/`ClosureHandler` (`AST/QualifiedName.sk:22-24`),
  rather than reusing `as_mono`'s type-substitution encoding (the axis of
  variation here is a lambda-set, not a type-argument tuple).
- **Per-instantiation concrete propagation:** reuse the *exact same* B.1
  edge-collector function, invoked again here with concrete sets plugged
  into the boundary flow variables instead of symbolic placeholders. This is
  what resolves `α` at every internal construction/destructuring/call/lambda
  site inside the instantiation, without writing a second traversal — B.1's
  collector is written once and called from both Phase B (profile
  construction) and Phase C (concrete resolution).
- **Final substitution — this is where old Phases 1–3 of
  `ClosureLowering/Pass.sk` actually run now.** Reuse its `Rewrite[T]`/
  `auto(Map)` skeleton (lines 43–144) essentially verbatim, but key
  `ClosureLowerer.fn_type_to_enum` (`Pass.sk:151`) by `(TypeDef,
  ConcreteSetId)` instead of bare `TypeDef`. The `ConcreteSetId` at each
  site comes directly from Phase C's resolution: trivial at `CreateClosure`
  (it's the singleton set containing that lambda), read off the propagated
  concrete set at `DynamicCall` and at any closure-typed field occurrence.
- **Duplication:** when the same function is reached with different
  resolved sets on the same parameter, clone it (and any datatype
  instantiated along with it) under a fresh `ClosureInstance` id — mirrors
  ordinary monomorphization's "duplicate over the varying axis" rather than
  widening to a union.
- **Memoization:** instantiation resolution is monotone over a finite
  lattice (the program's finite set of lambda literals), so `(function,
  concrete-input-set-tuple)` is looked up before recomputing — same
  memoization discipline as `Mono.sk`'s `fn_done`/`struct_done`/`enum_done`
  sets.

Old **Phase 4** (`generate_enum`, `generate_handler`, `Pass.sk:239-352`) is
reused **completely unchanged**. It only ever consumes a `ClosureEnumInfo`
value; it has no dependency on whether that value was produced by lazy
signature-keyed allocation (today) or Phase C's concrete-set-keyed
allocation (this plan).

## Pass ordering

No pipeline reordering is required. `main.sk:237` already calls
`lower_closures` strictly before `lower_coroutines` (`main.sk:240`), which is
exactly the ordering *Data-Flow Driven Closure Merge*'s "Interaction with
coroutine classification" section requires — coroutine SCC classification
must see the **duplicated** call graph produced by Phase C.
`CoroutineLowering/FunctionGroupBuilder.build_coroutine_map` needs no changes
at all: it already rebuilds its call graph fresh from `program.functions` on
every run, so it will pick up Phase C's duplicated functions automatically
the next time it runs.

## Net effect on `ClosureLowering/Pass.sk`

- **Deleted:** the lazy `get_or_create_enum` allocation inside the
  `TypeDef.Fn` interception (`Pass.sk:62-68`) and inside
  `ExprKind.CreateClosure`/`DynamicCall` handling (`Pass.sk:177-229`). These
  become lookups into the `(TypeDef, ConcreteSetId) -> ClosureEnumInfo` map
  Phase C already produced, instead of allocate-on-first-sight during the
  rewrite.
- **Kept as-is:** the entire `Rewrite[T]`/`auto(Map)` scaffolding (lines
  47–144), `generate_enum`/`generate_handler` (239–352), and the overall
  three-pass-over-program structure of `lower_closures` (356–393) —
  structs, then enums, then functions, then emit generated enums/handlers.

## Open items carried over from dataflow.md, now with implementation hooks

- **Naming for duplicated instantiations** (`f`, `f$1`, `f$2`, ...): resolved
  by the new `BaseName.ClosureInstance(index)` proposal above; exact display
  formatting still open.
- **Phase A precision for datatypes with many closure fields:** still
  per-field variablization as specified; no special handling identified as
  necessary during this investigation, but not exercised against a
  worst-case datatype yet.
- **New:** whether to extract a shared `Utils/CallGraph.sk` helper now that
  a fourth call-graph-SCC user (Phase B) would otherwise duplicate
  `FunctionGroupBuilder.sk`/`PurityCheck/GroupBuilder.sk`'s pattern a third
  time over. Not required for correctness; worth deciding before
  implementation starts to avoid a fourth copy-paste.
