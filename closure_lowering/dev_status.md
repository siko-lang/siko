# Dev Status

What actually exists in the compiler right now, as opposed to what
*dataflow.md* and *implementation_plan.md* describe as the target design.
This file is expected to go stale fast — update it whenever the real state
moves, don't trust it blindly.

## What's implemented

Layout in `siko/ClosureMerge/`:

- [GeneralizeDataTypes.sk](../siko/ClosureMerge/GeneralizeDataTypes.sk) —
  Phase A (datatype closure-field generalization).
- [GeneralizeFunctions.sk](../siko/ClosureMerge/GeneralizeFunctions.sk) —
  tags closure infer vars into every function's signature **and body
  expression/pattern types** (per-function numbering, restarting at ?0).
- [InferProfiles.sk](../siko/ClosureMerge/InferProfiles.sk) — Phase B
  driver: per-SCC walk, summarization, entry point (`infer_profiles`).
  Split into submodules under `InferProfiles/`:
  - [Profile.sk](../siko/ClosureMerge/InferProfiles/Profile.sk) — the
    shared vocabulary (`FlowNode`, `ClassRef`, `Profile`,
    `ApplySummary`/`CreationSummary`); later subpasses (Phase C) consume
    these.
  - [CollectDeps.sk](../siko/ClosureMerge/InferProfiles/CollectDeps.sk) —
    call-graph collector + SCC grouping.
  - [Flow.sk](../siko/ClosureMerge/InferProfiles/Flow.sk) — var collection,
    slot tables, union-find, `FlowCtx`, the `Flow` trait + instances,
    per-construct rules, profile instantiation, `@closure_alias`. Still the
    big one (~880 lines); natural further cuts if needed: type utilities /
    call rules / alias rule.
  - [Debug.sk](../siko/ClosureMerge/InferProfiles/Debug.sk) — all debug
    printing (`debug_dump_classes`, `dump_profiles`).

Phase C (concrete resolution / duplication) **does not exist**, and the
residual parts of Phase B (dynamic-call and capture flow, see below) are
recorded but not resolved.

### Verified behaviors (manual fixtures in `test.sk`, via `--debug-closure`)

Each of these was predicted first and confirmed against the dump:

- identity transfer: `id(t: T) -> T` gives `{?in, ?out}` per slot, no
  cross-slot merging;
- branch merging: `select(t1, t2)` unions both args' slots with the result
  (and, per Steensgaard, with each other);
- escape through result: `create() -> State` gives `{?0, lambda ...}`;
- captures from a callee's return (not from args): the creation record's
  capture set shows the concrete lambda planted by the callee's
  instantiated profile;
- arg mutation as output: `fill(h: Holder)` storing a lambda into `h.slot`
  puts the lambda into the *arg's* class — profiles don't distinguish
  in/out parameters, mutation makes every arg var a potential output;
- container round-trip: `stash(s) -> Vec[State]` infers `{?arg, ?result}`
  straight through `Vec.push`'s internals (needs `@closure_alias` on the
  two `Std.Ptr` primitives, nothing else);
- coroutines: a lambda escaping via `yield` and a different one via
  `return` stay in separate classes end-to-end through
  `co f()` → `resume` → `match` (the consumer extracting only `Yielded`
  sees only the yielded lambda).

### GeneralizeFunctions (signature + body type generalization)

Runs after GeneralizeDataTypes on the same program. Every type occurrence in
a function — argument types, result/yield types, and the `ty` annotation on
every expression and pattern in the body — gets its closure slots tagged
with fresh vars from a per-function allocator. Same two interceptions as the
datatype pass (`ClosureInfo` tags a var; a bare `NamedInfo` reference to a
generalized datatype is instantiated with fresh vars matching that
datatype's arity, read back from its `type_arg_defs`). Everything else,
including the whole `FunctionDef` recursion, is `auto(Map)` derive
boilerplate. Key property this establishes: any two structurally equal types
carry closure-var lists of equal length in deterministic structural order,
so related types can be zipped positionally — which is what InferProfiles
relies on throughout.

### InferProfiles (Phase B, data-flow profiles)

- **Call graph + SCC**: its own `CollectDeps`/`auto(Fold)` collector
  (deps = user-defined `FunctionCall` targets plus `CreateClosure` lambda
  names), fed to `Utils.SCC.compute`, processed bottom-up.
- **Unification, not directed flow**: every value copy in Siko is an alias
  (structs are mutable references — writes through either side reach the
  other), so slot relationships are equivalences. The analysis is a
  union-find over `FlowNode(owner_fn, TypeVar)` nodes, with the lambda
  literals inhabiting each class stored at the class root. No edge list, no
  fixpoint — `unify` is called by zipping the closure-var lists of related
  expression types. This is Steensgaard-style; the deliberate imprecision
  is at control-flow joins (both branches of an `if` unify with each other,
  not just with the result).
- **Traversal**: the `Flow[Expr]`/`Flow[Statement]`/`Flow[Pattern]`
  instances are hand-written (this is where the unify rules live — the one
  place manual dispatch is genuinely necessary, since a derive can't relate
  parent/child types); every other node type defers to `auto(Fold)`
  derives.
- **Per-construct rules**: field access/construction/destructuring zip
  through slot tables (datatype pool position per struct field / variant
  item, built once from the generalized defs); intra-SCC calls unify
  directly with the callee's real signature vars; cross-SCC calls
  instantiate the callee's finished profile through a per-call-site
  substitution (this is what keeps unrelated call sites separate);
  coroutine-create calls map callee yield/result onto the two halves of the
  `Co` type; `resume` (a bodyless builtin) has a precise hand model — it
  behaves as a constructor, unifying the Co's yield half with the
  `CoResult.Yielded` payload slots and the return half with `Returned`
  (verified: a lambda escaping via `yield` and a different one via `return`
  stay in separate classes end-to-end through create → resume → match);
  all other extern/builtin calls get a conservative everything-connects
  model.
- **Profile**: per function, `groups: Vec[ClassRef]` — a partition of its
  boundary vars (vars in arg/result/yield types) into equivalence classes,
  each with the lambda literals inhabiting it. Vars of other SCC members
  are dropped (their own profiles cover those flows); classes with a single
  var and no lambdas are omitted.
- **`@closure_alias` annotation** (parsed in `Parser/Annotations.sk`,
  `Annotation.ClosureAlias`): structurally identical signature types
  (compared with closure vars erased) alias each other's closure content.
  Needed where the body launders pointers through integers, which erases
  flow at the type level (an integer type has no closure slots to carry the
  sets). Applied to `Std.Ptr.offset_of` and `Std.Ptr.copy`; those two make
  `Vec` internals (`push`/`get`/`grow`) fully inferable with no further
  annotations.
- **Residuals, recorded but NOT resolved**: dynamic calls (`ApplySummary`:
  callee/args/results as `ClassRef`s) and closure creations
  (`CreationSummary`: lambda + capture `ClassRef`s). The design docs never
  specified how closure application propagates flow symbolically; the
  current stance is that Phase C resolves these against concrete lambda
  sets (note: that resolution can grow sets that feed other applies, so
  Phase C has a fixpoint in it despite the design doc claiming otherwise).
  Consequence: closures flowing *through* a dynamic call (e.g. a lambda
  passed into a higher-order function and returned from it) are not yet
  linked. Cross-SCC profile instantiation also does not yet re-instantiate
  the callee's residuals at the call site.

### Wiring (updated)

The pipeline moved from `main.sk` to `siko/Driver/Passes.sk`. The
`--debug-closure` flag inserts a `closure-merge-debug` pass before closure
lowering that clones the program, runs generalize_datatypes +
generalize_functions on the clone, prints the generalized program, then runs
infer_profiles and dumps **every** function's profile with its generalized
signature, e.g.

```
fn Main.stash(__implicit_context_, Main.State[?0]) -> Std.Vec.Vec[Main.State][?1]:
  {?0, ?1}
```

(`(no boundary flows)` for empty ones — nothing is silently skipped, that
caused confusion once). `config.debug_closure` also gates internal dumps in
the pass itself: per SCC, every non-trivial equivalence class with fully
qualified nodes plus the raw apply/create records. The real compiled
program is still never touched.

### GeneralizeDataTypes: what it actually does

- Builds a datatype dependency graph (struct/enum → every named struct/enum
  reachable through its field types, including through `fn(...)->...`
  signatures) and runs `Utils.SCC.compute` over it.
- Processes SCCs bottom-up. Per SCC, in two passes over the SCC's own
  members (not the whole program):
  - Pass 1: every field whose type is `TypeDef.Fn(ClosureInfo)` gets a
    fresh named type var (`?0`, `?1`, ...) written into
    `ClosureInfo.closure_infer_var`. Every field that's a `TypeDef.Named`
    reference to an **already-finished** (outside-this-SCC) datatype with
    nonzero arity gets fresh vars threaded in as that reference's type
    arguments. References to a datatype **inside** the current SCC are left
    alone in this pass (arity not known yet).
  - Pass 2: the SCC's pool of vars is now final. Re-run the same walk with
    a `finalize` flag set, which patches the intra-SCC references left
    alone in pass 1 to use the full, final pool, and stamps every member's
    `type_arg_defs` with that same full pool.
- Variable numbering is scoped per SCC, not globally — every SCC starts
  again at `?0`. A multi-member (mutually recursive) SCC shares one flat
  pool of vars across all its members; this is a deliberate
  over-approximation (see "Known imprecision" below), not yet the precise
  per-member reachability the design docs gesture at.
- Mutates the `Program` passed to it **in place**. It does not clone its
  input itself.

### How the traversal is built

Matches the ask that drove this design: almost no hand-written tree
walking. There is a local `trait Generalize[T] { fn generalize(t: T, ctx:
Generalizer) -> T }`, `auto(Map)`/`auto(Map, =)` instances for every AST
node type it touches, and **exactly two hand-written interceptions**:
`ClosureInfo` (tags the var) and `NamedInfo` (threads/patches type
arguments on datatype references). Everything else — including the
`StructDef`/`EnumDef` top-level recursion itself — is a bare `auto(Map)`
derive instance. Identity (`auto(Map, =)`) instances exist for whatever the
derive would otherwise recurse into but shouldn't (function bodies,
annotations, method maps, etc.) so the pass provably never looks at
anything but field/item types.

### CLI/Config and incidental changes

- CLI/Config: `--debug-closure` flag (`siko/CLI.sk`), `Config.debug_closure`
  (`siko/Config/Config.sk`), following the existing `--debug-co` pattern
  exactly.
- Incidental: `Siko.HIR.Program.Program` and `Siko.AST.Instances.InstanceDef`
  (plus its transitive field types `InstanceKind`, `TypeAssignment`,
  `ExternalDeriveMode`, `FnName`, `BodyStrategy`, `ExternalDeriveParams`,
  `ExternalDerivedInstanceInfo`) now derive `Clone`, purely so
  `program.clone()` works for the debug path above. This was a deliberate,
  requested addition (not scope creep) — general `Program` cloning was
  wanted independently of this pass.

### Latent compiler bug found and fixed along the way

This work's new structs surfaced a CBackend bug: user struct field names
were emitted **raw** into the generated C. A field literally named `result`
collided with the ctor bodies' local `result` variable and broke the
self-build; a field named after any C keyword would have too. Fix: all
struct field names are now mangled through `struct_field_name`
(`f_<name>`, `CBackend/Util.sk`) at every emission site — struct defs,
field access, ctor params/assignments, match-struct destructure, and the
string-literal compound initializer (`String`'s `value`/`len` were
hardcoded). Enum variant payloads (`field_N`), `tag`/`payload`, and the
builtin array struct's `data` are synthetic backend names and stay raw. The
whole C runtime is MiniC-generated (no hand-written C touches Siko struct
fields), which is what made the universal mangle safe.

### Prerequisite work this pass depends on that predates it

`TypeDef.Fn` is no longer a bare `(Vec[TypeDef], TypeDef)` pair — it now
wraps a `ClosureInfo { args, return_ty, closure_infer_var: Option[TypeVar]
}` (`siko/AST/Type.sk`). That refactor (adding the `closure_infer_var`
slot) landed as part of this work and is what makes Phase A's "tag in
place" approach possible instead of replacing the whole field type with a
bare `TypeDef.Var`.

## Known imprecision / open questions (not yet resolved, not yet asked)

- **SCC-wide pool sharing is coarser than necessary.** Every member of a
  multi-member SCC gets the *entire* SCC's pool as its `type_arg_defs`,
  even fields/members that don't actually reach every var. Confirmed
  correct-but-dense on the one worked example so far (`Foo`/`Bar` mutual
  recursion via `Option[Bar]`, four vars, both get all four). Whether this
  needs tightening to true per-member reachability is unresolved; no
  correctness problem has been found, just visual density.
- **Dynamic-call and capture flow is residual only** (see InferProfiles
  above). Until applies/creations are resolved (Phase C, or an earlier
  symbolic treatment), profiles under-report flows through higher-order
  functions. This is the biggest open semantic hole — the design docs are
  silent on it.
- **Named/rest pattern fields**: `NamedPatternInfo.named_args` and
  `has_rest` are ignored — only positional destructure args get edges (the
  sub-patterns are still walked for env binding). Post-lower-match this may
  be moot; unverified.
- **ForLoop / Index / Try / List / FunctionPtrCall / Transmute** use
  conservative cross-linking rather than precise slot mapping.
- Profile instantiation zips caller/callee var lists positionally and
  silently truncates on length mismatch (`fill_subst` min-length) — a
  mismatch would indicate a generalization bug, and is currently swallowed
  rather than surfaced.
- No test suite exists for these passes yet. Verification so far is manual:
  hand-written `.sk` fixtures + `--debug-closure` + eyeballing the printed
  defs/profiles.

## What we expect from the next passes (Phase C and onward)

Not designed in detail yet, but these expectations came out of building
Phase B and are firmer than what the original *dataflow.md* says:

- **Phase C consumes `Profile` as-is.** The `Profile.sk` submodule is the
  contract: equivalence groups over boundary vars + lambda sets per class,
  plus apply/creation residuals expressed in the same `ClassRef`
  vocabulary. Resolution = walk from `main`, plug concrete lambda sets into
  a callee's groups per call site, memoize by
  `(function, concrete-set-tuple)`, duplicate on mismatch (mint identity
  via a new `BaseName` variant, e.g. `ClosureInstance(index)`).
- **Phase C has a fixpoint**, contrary to the design doc's "no fixpoint
  needed" claim (written before dynamic calls were thought through).
  Resolving an apply against a concrete callee set instantiates the member
  lambdas' profiles, which can push new lambda literals into slots that
  feed *other* applies' callee sets. Iterate until stable.
- **Apply × creation join**: at an apply whose concrete callee set is
  `{L1..Ln}`, wire per lambda: apply args → L's non-capture params, L's
  result → apply results, and — via L's `CreationSummary` — the capture
  sources recorded at L's creation site → L's leading capture params
  (captures are the leading params of the lifted lambda fn,
  `FunctionDefKind.Closure(n)`). The creation site and apply site are
  generally in different functions/instantiations; joining them across
  instantiation boundaries is the hairiest open sub-problem.
- **Enum/handler assignment** then keys on the resolved concrete set per
  slot: `ClosureLowering`'s existing `generate_enum`/`generate_handler` are
  reusable unchanged; only its lazy one-enum-per-signature allocation gets
  replaced by lookups into Phase C's output (see *implementation_plan.md*).
- **Coroutine classification** (`CoroutineLowering`'s SCC prepass) must run
  on the **duplicated** call graph Phase C produces — no code change
  expected there since it rebuilds its graph from `program.functions` each
  run; only pass ordering matters, and the order is already correct.
- **Perf headroom if needed**: the union-find has no union-by-rank and the
  per-SCC state is rebuilt per group; `--debug-closure` on the full
  self-build hasn't been timed. Fine until proven otherwise (it's
  debug-gated), but Phase C will run for real.
- **Annotation extension path**: `@closure_alias` currently takes no
  arguments (all identical-shape signature types alias). If a function ever
  needs finer control, the planned extension is naming specific params
  (`@closure_alias(ptr, result)`); nothing needs it yet. Note
  `Array.as_ptr`-style casts (`*Array[T,N] -> *T`, different shapes) would
  NOT be covered by the identical-shape rule — no closure-carrying arrays
  exist in practice yet, so this is parked.

## Explicitly not started

- Phase C (concrete resolution from `main`, duplication/memoization,
  residual apply/creation resolution against concrete lambda sets).
- Any integration with the real `ClosureLowering/Pass.sk` — that pass is
  completely unmodified. The two systems don't talk to each other yet.
- Coroutine-classification interaction (`CoroutineLowering/*`) — untouched,
  irrelevant until Phase C exists.
