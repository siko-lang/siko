# Allocation Flow — Implementation Investigation

What it would actually take to build the analysis from `allocation_flow.md`
(and the follow-ups in `allocation_flow_expanded.md`) into the compiler.
Findings are grounded in the current sources; file references are to real
code read during this investigation.

## 0. One correction first: Siko safe code mutates

`allocation_flow_expanded.md` §3 claims post-lowering safe Siko has no field
assignment. **That is wrong.** `Statement.Assign(Expr, Expr)`
(`siko/Common/src/AST/Statement.sk:25`) takes an arbitrary expression LHS,
and field-assignment through references is pervasive compiler style
(`ctx.scc_members = members`, `r.instances[iid].children.insert(...)`, …).
`HIR/Program.sk:58` says it outright: "Map.clone() preserves its values,
which is normally desirable for Siko's reference types … compiler passes
mutate definitions in place." Reference semantics, alias-visible in-place
mutation.

The design survives (mutation is just an edge — see §5 below for why the
profile vocabulary covers it), but the analysis needs a first-class **store
rule**, and the expanded doc's precision argument needs rewording: the
benefit is not "no stores exist" but "stores into *boundary* slots are
expressible in profiles, so nothing is lost."

## 1. Pipeline placement

Slot: **between `value-types` and `lower-ifs`** in `build_full_passes`
(`siko/Compiler/src/Passes.sk:160-177`).

Why exactly there:

- `value-types` must have run: the `value_type` flag on
  `StructDef`/`EnumDef` (`AST/Data.sk:57,98`) is what distinguishes
  pointer-carrying positions from inline payload, and it is computed by
  `calculate_value_types` (`Common/src/ValueTypes.sk:212`).
- `lower-ifs` / `lower-basic-block` must *not* have run: at value-types
  time, bodies still have `Statement.Loop / ForLoop / WhileLoop`
  (`AST/Statement.sk:28-30`), `Match`, `Block` scopes — everything the
  scope-instance lifetime rule needs. After basic-block lowering it's
  `Goto`/`Label`/`Switch` straight-line form (see the whitelist in
  `LowerEnums.lower_expr`, `Common/src/LowerEnums.sk:111-151`) and scope
  structure is gone.
- Ctor call sites are still recognizable: `FunctionKind.StructCtor /
  VariantCtor` on `FunctionCallInfo` survives until `lower-enums` retargets
  them to generated `new_ctor` functions (`LowerEnums.sk:126-129`). The
  stamp goes on exactly these calls.
- Closures and coroutines are already structs by then, so **closure
  environments and coroutine frames are ordinary allocation sites** — a
  non-escaping closure gets stack placement with zero extra machinery. This
  is likely a significant chunk of the win, for free.

The pass module belongs in `siko/Common/src/` (like every other lowering)
so the check/macro pipelines can share it if ever needed; registration is a
`Pass(name: "alloc-flow", …, run_sanity_check: True)` entry.

Amusing detail: the motivating example in `allocation_flow.md` (`struct Foo {}`)
is already a value type today — `build_required_candidates` forces empty
structs by-value (`ValueTypes.sk:28`). The real targets all have fields.

## 2. The var-threading decision (the big one)

The closure machinery attaches analysis identity to program points by
**threading TypeVars through the types themselves**: `generalize_datatypes`
gives each datatype SCC a var pool and rewrites `Named` type args to carry
them (`GeneralizeDataTypes.sk:80-102, 156-203`); `generalize_functions`
does signatures; then the flow walk reads vars straight off `expr.ty`
(`esignature_vars`, `Flow.sk:88`).

Could allocation flow use side tables instead, since it changes no layout?
**No — and the reason is a hard constraint: `Expr` has no stable identity.**
`AST/Expr.sk:23` — an expression is `{kind, location, ty}`; locations are
not unique, there are no node ids. There is nothing to key a side table on.
The var-threading trick *is* the mechanism for attaching per-program-point
analysis state in this compiler. So:

- **Decision: mutate types in place, analyze, then erase.** Run a
  generalize pass (clone of the two-phase SCC pool builder in
  `GeneralizeDataTypes.sk`) threading fresh provenance vars into datatype
  defs and function signatures/bodies, run the analysis, then strip vars
  with the existing `erase_vars` before `lower-ifs`. Closure lowering
  already established that transiently var-polluted types between two
  passes is a workable state; the sanity checker runs *between* passes, so
  erase must happen inside the same pass.
- Which positions get vars: a field of non-value `Named` type, `Ptr`,
  `VoidPtr`, `FnPtr` — `TypeDef.has_ptr` (`AST/Type.sk:128`) is close to
  the needed predicate but not exact (it ignores non-value `Named`s, which
  are the *main* case post-value-types since they become pointers in
  `lower-pointers`). A new `is_pointer_position(ty, program)` predicate
  consulting `value_type` flags is needed.
- Unlike closure lowering there is **no datatype duplication and no
  rewrite** — `GeneralizeFunctions` + `Lower.sk`'s whole
  enum-synthesis/rewrite machinery (`Lower.sk:76+`) has no analogue here.
  The output is only stamps on call sites.

## 3. SCC grouping and the symbolic phase

`build_groups` (`InferProfiles/CollectDeps.sk:123`) is directly reusable —
it is `pub`, generic over the AST via `auto(Fold)` derives, and its one
special case (ctor calls contribute no dep, `CollectDeps.sk:84-93`) is also
correct for allocation flow: ctors have no bodies until lower-enums, so
there is nothing to order against.

The flow walk (`Flow.sk`, instance `Flow[E.Expr]` at 442) is the template
but **cannot be shared as-is**, because every rule changes meaning under
directionality:

| construct | closure rule (unify) | allocation rule (directed) |
|---|---|---|
| `Var` | env ↔ out | env → out |
| `FieldAccess` (rvalue) | pool slot ↔ out (`field_access_edges`, 687) | slot → out |
| `Assign` (`Flow.sk:618-622`: `rhs ↔ lhs`) | symmetric | **rhs → lhs target**; needs an lvalue walk mode (see §5) |
| ctor args (`ctor_edges`, 873) | arg ↔ pool slot | arg → slot, **plus seed a fresh abstract pointer at the site** |
| `If`/`Match` arms | branch ↔ out | branch → out |
| `Return`/result | expr ↔ result vars | expr → result |
| `Transmute`, len-mismatch fallback (`FlowCtx.unify`, 360) | cross-product union | cross-product edges *both ways* (lossy, safe) |
| bodyless default (`bodyless_signature_edges`, 908) | all boundary vars one class | all-pairs edges both ways **+ → BlackHole** |

So the realistic shape is a new `AllocationFlow/Flow.sk` written in the
same trait-with-`auto(Fold)` style, with a `Edge(src, dst)` emitter instead
of `uf.union`, plus two extra sink nodes (`BlackHole`, `Heap`) in the node
space. The `FlowNode(owner_fn, var)` interning idea and the
`DataTables` position pools (`Flow.sk:172-224`) port unchanged.

**Profile publishing** replaces `publish_profile`'s class-canonicalization
(`InferProfiles.sk:125-167`) with reachability: compute boundary→boundary,
boundary→BlackHole, boundary→Heap closure over the SCC graph, restricted to
`boundary_vars` (which, with generalized signature types, automatically
includes the datatype slot vars of every arg/result type — this is why
mutation stays expressible, §5). The profile struct is smaller than
`FunctionProfile` (`Profile.sk:25`): arg/result types + an edge list; no
`contains`/`applies` analogues needed because closures are already gone at
this pipeline stage. Cross-SCC call sites instantiate profiles by the same
positional `fill_subst` zip (`Flow.sk:998-1008`).

## 4. The concrete phase: deliberately simpler than closures

Closure lowering's concrete phase (`Resolve.sk`) is a **context-sensitive
whole-program instance walk** — `InstanceKey(fn_name, inputs)` per distinct
input closure-set vector, iterated to fixpoint from entry points
(`Resolve.sk:470-528`), because closure *contents* must be known globally
to build enums.

Allocation flow needs none of that. Decisions are only about a function's
own ctor sites, and by the escape formulation the caller's pointer contents
are irrelevant. The concrete phase is one pass over `program.functions`,
independently per function:

1. Re-walk the body building the local directed graph (the `walk_data`
   caching idiom, `Resolve.sk:111-159`, shows how to reuse the symbolic
   walk for this).
2. Seed: every ctor call whose constructed type is non-value gets a fresh
   abstract pointer id, in traversal order (traversal order is
   deterministic, which substitutes for the missing expr identity —
   the decision phase and the stamping phase must simply be the *same*
   traversal).
3. Propagate along edges (plain worklist; instantiate callee profiles at
   call sites as extra local edges).
4. Classify each site: reaches result vars / a boundary slot var /
   BlackHole / Heap / an outer-scope slot while loop-born → heap (or
   Dunno for BlackHole); else stack.
5. Stamp the call site and discard the graph.

No global fixpoint, no instance interning. This is much cheaper than the
closure resolve pass on the same program.

**Scope instances:** the walk context (`FlowCtx.env` is
`Map[VariableName, Vec[TypeVar]]`, `Flow.sk:342`) grows a scope depth: push
on entering `Loop/ForLoop/WhileLoop` bodies, record the current depth on
each `Let`/`Declare` binding and on each ctor site. The heap rule "reaches
a slot live across the back edge" becomes: reaches a var bound at a
shallower loop depth than the site. Block scoping finer than loop depth is
unnecessary for correctness (only loops re-execute sites).

## 5. The store rule in detail

`Assign(lhs, rhs)` needs an **lvalue mode** in the expression walk. Today's
symmetric `flow` doesn't care; directed flow must produce, for an LHS of
`FieldAccess(recv, f)`: edge `rhs → slot_var(recv.ty, f)` (the same
`struct_fields` position pool lookup as `field_access_edges`, just used as
a target). LHS forms to handle: `Var` (rebinding — edge into the binding's
vars), `FieldAccess`, `TupleIndex`, `Index`, `Deref` (unsafe — edge to
BlackHole). Nested paths (`a.b.c = x`) resolve to the innermost receiver's
slot var, which the type-arg threading already exposes.

Why this stays sound across functions: the receiver's slot var, when the
receiver is (reachable from) an argument, **is a boundary var** — profiles
carry `?arg_p → ?arg_obj.slot` as an ordinary boundary-to-boundary edge, so
"callee stores one caller value into another caller object" is visible to
the caller. The one case profiles cannot express — "callee stores *its own
fresh allocation* into a caller object's slot" — doesn't need expressing:
the callee sees its own pointer reach a boundary slot and decides heap
locally. The caller doesn't need to know (heap pointers don't poison).

This resolves the mutation question cleanly: mutation costs one walk mode
and zero new profile machinery.

## 6. Stamping: the `FunctionCallInfo` field

```siko
pub enum AllocDecision { Heap, Stack, Dunno }

pub struct FunctionCallInfo {
    ...
    pub allocation: AllocDecision = AllocDecision.Heap,   // NEW
}
```

Two facts make this low-risk:

- `FunctionCallInfo` already uses field defaults
  (`AST/Expr.sk:108-115`: `witnesses = WitnessInfo.new()`, `named_args =
  Vec.new()`, `is_coroutine_create = False`), so no construction site in
  the compiler needs touching.
- Every intermediate pass moves `FunctionCallInfo` through `auto`-derived
  instances or in-place field mutation (e.g. `LowerEnums.sk:123-133`
  mutates `info.args`/`info.qname` and keeps the rest), so the stamp
  survives `lower-ifs` → `lower-basic-block` → `lower-pointers`. A pass
  that ever rebuilt the struct positionally would silently reset to `Heap`
  — which is the *safe* failure mode. Worth one sanity-check assertion
  anyway (count stamped sites before/after, behind the debug flag).

The default `Heap` means: no analysis run ≡ today's behavior; the pass is
trivially flag-gatable in `Config`.

## 7. Consumption in lower-enums

Today the generated ctor body allocates or declares based on the *type*
(`CtorBuilder.new_local`, `LowerEnums.sk:200-214`: heap → `ExprKind.Alloc`
via `Let`, value → `Declare`). The decision moves from per-type to
per-call-site. Two viable shapes:

- **(a) Placement ctor variant.** For each type with ≥1 stack-stamped
  site, additionally synthesize `ctor_place(out: *T, fields...)` whose body
  is the same `assign_field` sequence into `Deref(out)` instead of a fresh
  local, returning `out`. Call sites: `Declare tmp: T` + pass
  `AddrOf(tmp)`. All the pieces exist — `AddrOf`/`Deref` construction is
  exactly what `payload_read` does (`LowerEnums.sk:64-81`), and
  `make_ctor_fn` (`LowerEnums.sk:227`) is reusable scaffolding with a
  second qname (`new_ctor` has room for a sibling constructor kind in
  `QualifiedName`).
- **(b) Inline at site.** Ctor bodies are trivial straight-line
  `Declare` + assigns; a stack site could be expanded in place. But at
  lower-enums time call sites sit in arbitrary expression positions of
  post-basic-block statements, so this needs temp introduction — more
  fiddly than (a) for no gain. **Choose (a).**

Important discovery about the return type: heap ctors return `Ptr(T)`
(`generate_struct_ctor`, `LowerEnums.sk:251-275`: `ret_ty = Ptr(struct_ty)`
iff `!value_type`). A stack site keeps consuming a `*T` (from `AddrOf`), so
**downstream types don't change at all** — the pointee just lives in a
frame. Only the by-value-return upgrade (expanded doc §5) would touch
signatures; it is cleanly separable and should be phase 2. Phase 1 =
locally-dead allocations only, zero ABI change.

**Alloca placement:** the stack site lowers to `Declare` in whatever block
the call was in. Whether the backend emits `Declare` as an entry-block
alloca or an in-place one must be verified in `LLVM/Lower.sk` before
shipping — a loop-resident `Declare` that becomes a loop-resident `alloca`
would grow the stack unboundedly. If it isn't already hoisted, hoisting
`Declare`s of stack-stamped ctors to function entry during lower-enums is a
few lines (straight-line statement lists, `LowerEnums.lower_stmts`).

## 8. Extern/builtin models

The conservative default for bodyless functions goes through the same
synthesized-profile path as closures (`bodyless_signature_edges` → ordinary
profile, `InferProfiles.sk:210-215`), directed version per §3's table. On
top of that:

- **Precise models by qname**, following the `resume` precedent
  (`resume_signature_edges`, `Flow.sk:924`, dispatched by comparing
  `B.get_resume_qn().base()` — the `Constants.Builtins` registry is the
  lookup mechanism to extend).
- **`@alloc_transparent` annotation** for the unsafe container layer:
  all-pairs boundary edges *without* the BlackHole. The annotation
  machinery is ready — `Annotation` predicates like `is_value_type()` /
  `is_builtin()` (`AST/Data.sk:59-84`) and `FunctionDef.is_closure_alias()`
  show both the data-level and function-level patterns; parsing lives with
  the other `@` annotations.
- `gc_allocate` / `gc_allocate_array` / `allocate` (`std/Common/src/Ptr.sk:13-62`):
  model as heap-pointer *sources* (edge `Heap → result`), not black holes.
- `Deref`/`AddrOf`/`Transmute`/atomics appear as expr kinds, not calls
  (`AST/Expr.sk:263-296`) — they need walk rules, not profiles: `AddrOf`
  and `Transmute` pass-through (cross-product), `AtomicStore` value →
  BlackHole, `Deref` of raw pointers → BlackHole-sourced unknown.

## 9. Debug / stats / validation plumbing

All conventions exist and should be cloned:

- `Statistics` as an implicit (`with stats = s`, `Lower.sk:35-52`), with a
  final histogram: sites total / stack / heap / dunno, dunno-by-cause
  (which extern qname black-holed it) — the cause histogram is the tool
  that tells you which model to write next.
- Config flags (`get_config().debug_closure*` pattern): `debug_alloc_flow`
  dumping per-function decisions, plus the master enable flag while the
  pass is experimental.
- `Pass.run_sanity_check: True` (pass records in `Driver/Passes.sk:262`).
- The pass dump hook (`dump: Some(...)`) printing the histogram, like
  value-types does with `dump_stats` (`ValueTypes.sk:267`).
- Validation sequence: flag off by default → full test suite + self-build
  with flag on → compare outputs. Every `Stack` verdict is an optimization
  of a working program, so differential testing is decisive.

## 10. Suggested file layout

```
siko/Common/src/AllocationFlow/
    Generalize.sk      // provenance-var threading (clone of GeneralizeDataTypes shape) + erase
    Tables.sk          // pointer-position pools (DataTables analogue)
    Flow.sk            // directed edge walk, lvalue mode, scope depths
    Profile.sk         // AllocProfile: boundary edge list + BlackHole/Heap rows
    InferProfiles.sk   // SCC driver (build_groups reused from ClosureLowering)
    Decide.sk          // per-function seed/propagate/classify/stamp
    Models.sk          // extern/builtin precise models + @alloc_transparent
    Statistics.sk
    Debug.sk
```

plus: `AllocDecision` + field in `AST/Expr.sk`, annotation parsing, config
flags, pass registration in `Compiler/src/Passes.sk`, ctor_place generation
in `LowerEnums.sk`.

## 11. Open items surfaced by the code (decide before/while building)

1. **`Declare` → alloca placement in the LLVM backend** (§7) — must be
   read before the first stack site ships; determines whether lower-enums
   needs the hoist.
2. **String literals / `List` literals / `Range`** — which of these
   allocate, and through what path? They don't go through ctor calls, so
   phase 1 simply doesn't touch them; worth confirming they can't *receive*
   stack pointers either (they can't — their element types are what they
   are — but the walk must still propagate through them).
3. **`TypeDef.Array` fields and tuples** — `lower-tuples` ran before
   value-types, so tuples should be gone from bodies; arrays embed inline
   and their element positions need pool entries like fields.
4. **Shadowing / `VariableName` uniqueness in `env`** — the closure walk
   happily overwrites (`ctx.env.insert`) which is correct for its
   flow-insensitive merge; for scope depths, rebinding at a different depth
   must keep the *max-escape* depth. Small, but easy to get wrong.
5. **Entry points and `main`** — no special handling needed (unlike
   closure resolve which seeds from `program.entry_points`,
   `Resolve.sk:486`); every function is analyzed uniformly.
6. **Interaction with `is_coroutine_create` call sites** — co_create was
   lowered away before this pass, but the flag persists on
   `FunctionCallInfo`; assert it never co-occurs with a ctor kind here.
7. **The expanded doc's §3 needs the correction from §0** — mutation
   exists; update the prose before it misleads anyone.

## 12. Effort sketch

| piece | size | risk |
|---|---|---|
| Generalize + Tables | medium — clone of a known-working pass | low |
| Directed Flow walk + lvalue mode + scopes | the core work | medium — rule-by-rule review against `Flow.sk` |
| Profiles + SCC driver | small — simpler than closures' | low |
| Decide + stamp | small | low; determinism coupling between walk and stamp orders |
| Models + annotation | small each, open-ended total | the precision budget lives here |
| LowerEnums ctor_place | small | low; alloca-placement check first |
| AST field + plumbing | trivial | none |

Phase 1 (locally-dead allocations, no ABI change, conservative models +
`@alloc_transparent` on Vec) is a self-contained deliverable that already
covers temporaries, wrappers, iterator states, and non-escaping closure
environments. Phase 2 (by-value root returns) and phase 3 (finer container
models) layer on without rework.
