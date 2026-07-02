# Dev Status

What actually exists in the compiler right now, as opposed to what
*dataflow.md* and *implementation_plan.md* describe as the target design.
This file is expected to go stale fast — update it whenever the real state
moves, don't trust it blindly.

## What's implemented

One file: [siko/ClosureMerge/GeneralizeDataTypes.sk](../siko/ClosureMerge/GeneralizeDataTypes.sk).
This is **Phase A only** (datatype closure-field variablization from
*dataflow.md*), and it has already drifted from that doc in a few ways (see
below). Everything else in *dataflow.md* — Phase B (function data-flow
profiles), Phase C (concrete resolution / duplication) — **does not exist**.
No SCC-over-call-graph code, no flow graph, no profile instantiation, no
duplication/monomorphization-style walk. None of it.

### What the implemented pass actually does

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

### Wiring

- `siko/main.sk`: gated behind `get_config().debug_closure`. When set,
  clones `program` (`Program` now derives `Clone` — see below), runs
  `generalize_datatypes` on the clone, and prints `cloned.format()`. **The
  real compiled program is never touched** — this is a side-channel debug
  dump, not a pipeline stage. `lower_closures`/`lower_coroutines` etc. run
  on the original, untouched `program` exactly as before.
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
- No test suite exists for this pass yet. Verification so far is manual:
  hand-written `.sk` fixtures + `--debug-closure` + eyeballing the printed
  struct/enum defs.

## Explicitly not started

- Phase B (function data-flow profiles, SCC over call graph, constraint
  graph saturation/splicing).
- Phase C (concrete resolution from `main`, duplication/memoization).
- Any integration with the real `ClosureLowering/Pass.sk` — that pass is
  completely unmodified. The two systems don't talk to each other yet.
- Coroutine-classification interaction (`CoroutineLowering/*`) — untouched,
  irrelevant until Phase C exists.
