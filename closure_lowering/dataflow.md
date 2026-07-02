# Data-Flow Driven Closure Merge

The current closure lowering pass (see *Closure Lowering*) allocates one closure
enum per **function-type signature**: every lambda of type `fn(A, B) -> R`,
anywhere in the program, becomes a variant of the same enum and is dispatched
by the same handler. This is simple but imprecise, and it has a specific
documented cost: if any lambda of a given signature is (or transitively calls)
a coroutine, the shared handler becomes a coroutine, and so does every other
lambda merged into it — regardless of whether they are actually related.

This document describes a replacement pass that allocates enums by **data
flow** instead of by type: two lambdas share an enum only if a value could
actually flow from one lambda's creation site to the other's. Type equality
remains a necessary condition (flow never crosses signatures) but is no
longer sufficient. The pass reuses the compiler's existing SCC
infrastructure (already used for coroutine classification) at two different
granularities — datatypes, then functions — and finishes with a
monomorphization-style walk that duplicates functions and datatypes wherever
their closure content actually differs.

The pass replaces Phase 1 (enum and handler assignment) of closure lowering.
Phases 2–4 of that document (lambda site rewriting, call site rewriting,
handler generation) are unchanged in *shape*; they simply operate on the
enums this pass produces instead of one-enum-per-signature.

## Phase A — datatype closure-field variablization (SCC order over datatypes)

The program is already monomorphized, so no datatype has generic
parameters — except that closure-typed fields are about to reintroduce them.

Walk every datatype in the datatype-dependency graph, processing SCCs
bottom-up (dependencies before dependents, mutually recursive datatypes
processed together as one unit). For each datatype `D`, every field whose
type is a closure/function type, or transitively contains one, is replaced
by a fresh **closure infer variable** `α`:

```
struct Box { f: fn(Int) -> Bool }
```

becomes, conceptually,

```
struct Box<α> { f: α }
```

`α` does not yet mean anything concrete — it is a named placeholder for
"whatever closure representation ends up living in this field," to be
resolved by later phases.

Bottom-up SCC order matters because a datatype `D` may embed another
datatype `E` that itself has closure fields. By the time `D` is processed,
`E`'s fields have already been turned into named variables, so `D` can refer
to them directly (either re-exposing them as its own fresh variables, or
threading them through) rather than encountering an unresolved nested
closure type. Mutually recursive datatypes are processed as a single unit
for the same reason no member of the cycle can safely go first.

This phase only assigns *names* to the unknowns. It does not look at any
function bodies, call sites, or lambda creation sites — that begins in
Phase B.

## Phase B — function data-flow profile construction (SCC order over functions)

This phase computes, for every function, a **profile**: a closed,
composable summary of how closure-carrying values can move from that
function's inputs to its output. Profiles are built once per function and
then reused by instantiation at every call site, rather than re-analyzing
function bodies repeatedly.

Functions are processed in call-graph SCC order, bottom-up, exactly as in
coroutine classification. Each SCC is processed as a unit.

### B.1 — Build the intra-SCC constraint graph

For the set of functions in one SCC, build a single constraint graph whose
nodes are closure infer variables: the SCC's own parameter and return
variables (including any `α`s from Phase A reachable through parameter or
return datatypes), plus one fresh variable per local binding, temporary, and
lambda-literal creation site in the bodies of these functions.

Edges are **subset-flow** edges: `src → dst` means "the set of closures
reachable at `src` can flow into `dst`." Edges are added per construct:

- **Lambda literal.** A lambda creation site is a concrete singleton source:
  its node starts pre-seeded with `{ this lambda }`.
- **Assignment / binding.** `x = y` adds `y → x`.
- **Struct/variant construction and destructuring.** Fields are matched up
  structurally against the Phase A variables of the datatype in question;
  each matched pair gets an edge, not a full cross product of all fields
  against all fields.
- **Calls to a callee outside the current SCC.** The callee already has a
  finished profile from a previously processed SCC. That profile is
  **instantiated fresh** — every variable in it is freshly renamed, the same
  way a polymorphic type scheme is instantiated at a call site in
  typechecking — and the fresh copy's input-side variables are unified with
  the call's actual argument variables, and its output-side variable with
  the variable receiving the result. Because the profile is instantiated
  fresh per call site, unrelated calls to the same callee never contaminate
  each other's flow sets.
- **Calls to a callee inside the current SCC** (including self-calls). No
  finished profile exists yet — the callee is still being processed. These
  calls are wired **directly**: the call's actual argument variables are
  connected straight onto the callee's real signature variables (the same
  variables already in this graph), with no copy. This is what introduces
  cycles into the graph for recursive and mutually recursive functions.

### B.2 — Saturate to a fixpoint

Because intra-SCC calls are wired directly rather than instantiated, the
graph may contain cycles (self-recursion, mutual recursion). The graph is
saturated by repeatedly propagating along edges — if `a → b` and `b → c`
then effectively `a → c` — until no node's reachable set can grow further.
This closes every cycle into its final, stable set of reachable sources
before anything is emitted. All iteration needed for recursion happens here,
once, symbolically; nothing downstream needs to re-iterate.

### B.3 — Eliminate non-boundary variables

Once saturated, every variable that is *not* one of an SCC-member function's
own signature variables (its inputs and its output) is eliminated by
splicing: for an internal node `n` with incoming edges `{a → n}` and
outgoing edges `{n → b}`, replace them with direct edges `{a → b}` for every
pair, then delete `n`. Repeating this until only signature variables remain
leaves exactly the **profile**: a list of links whose endpoints are drawn
solely from the SCC's own functions' input/output variables. This is the
same shape as path-elimination / gen-kill summarization used for other
interprocedural summary problems.

The finished profile is a symbolic transfer function — "given whatever sets
eventually land on my inputs, here is how they propagate to my output" — not
yet resolved to any concrete set of lambdas, since a function's actual input
sets depend on all of its callers, most of which have not been visited yet
in SCC order (that happens in Phase C).

Profiles are stored per function and become available for **outside-SCC**
instantiation (rule B.1, first bullet) when later SCCs are processed.

## Phase C — concrete resolution and re-monomorphization (walk from `main`)

The final phase walks the call graph starting at `main`, propagating
**concrete** sets of lambda literals outward and monomorphizing every
function and datatype against the concrete sets it actually receives.

At each call site, reached with a set of concrete lambda-literal sets
already sitting on the actual arguments:

1. Instantiate the callee's profile at those concrete sets: plug the
   concrete sets in at the profile's input endpoints, and read off the
   resulting concrete sets at the output endpoints by following the
   profile's links, unioning wherever multiple links converge on the same
   endpoint.
2. Recurse into the callee's body using those concrete sets, which resolves
   every `α` occurrence inside it — construction sites, destructuring,
   further nested calls — to an actual, concrete set of lambda literals.

Because profiles were already saturated and closed in Phase B, applying a
profile once here is sufficient even for recursive functions — no fixpoint
iteration is needed in this pass; all of it already happened symbolically
during profile construction.

**Duplication, not merging.** When the same function is reached from two
different call sites with *different* resolved concrete sets on the same
parameter, the function is **duplicated**: two separate instantiations, each
closed over its own distinct closure content, propagating transitively into
any datatype instantiated along with it. This mirrors ordinary
monomorphization of generic code over types, except the axis of variation is
"which concrete set of closures," not "which type." This is a deliberate
choice, not a cost cut: the entire point of the pass is to give data-flow
separated closures distinct closure types and distinct handlers, so
duplication is preferred over widening a shared parameter to the union of
its callers' sets.

Resolution is monotone over a finite lattice (the program's finite universe
of lambda literals), so instantiating a given function against a given
concrete input-set tuple always produces the same output-set tuple.
Instantiation is therefore memoized by `(function, input-set-tuple)`,
exactly as ordinary monomorphization is memoized by `(function,
type-tuple)`, so the same function reached with the same concrete sets from
multiple call sites is only resolved once.

## Output and downstream consumption

Each distinct duplicated instantiation from Phase C carries its own
concrete, resolved set of lambda literals per closure-typed slot. This
concrete set is exactly the input the enum/handler assignment step needs:

- **One enum per resolved concrete set**, rather than one enum per
  signature. A signature may now correspond to many enums, each covering a
  disjoint (or at least data-flow-justified) subset of that signature's
  lambdas.
- **One handler per duplicated instantiation**, closed over that
  instantiation's specific enum.

This directly resolves the limitation noted in *Closure Lowering*: a
coroutine lambda's concrete set only appears in the duplicated
instantiations that actually reach it. Handlers outside that flow stay
non-coroutine, instead of every lambda of the same signature being pulled
in uniformly.

## Interaction with coroutine classification

Coroutine classification (the SCC analysis over the call graph, described in
*Coroutine Lowering*, Phase 1) must run **after** this pass, over the
**duplicated** call graph produced by Phase C, not the original one.
Duplication can split what was a single function into a coroutine copy and a
non-coroutine copy of the same source function; the pre-duplication call
graph has no representation for that distinction, so classifying against it
would reintroduce the same over-merging this pass exists to eliminate.

## Summary of phases

| Phase | Granularity | Order | Produces |
|---|---|---|---|
| A | Datatypes | SCC, bottom-up | Closure fields replaced by fresh infer variables (`α`) |
| B | Functions | SCC, bottom-up | Closed data-flow profiles: links between each function's own input/output variables |
| C | Call graph, from `main` | Single walk, memoized | Concrete resolved closure sets per instantiation; duplicated functions/datatypes where sets differ |

| Phase B edge rule | Applies to | Behavior |
|---|---|---|
| Outside-SCC call | Callee profile already finished | Instantiate fresh, unify at call site (acyclic) |
| Inside-SCC call (incl. self) | Callee still being processed | Wire directly onto callee's real signature variables (cyclic, needs saturation) |

## Open items for later design

- **Naming.** Mechanical naming scheme for duplicated instantiations (e.g.
  `f`, `f$1`, `f$2`, …) is not yet decided.
- **Precision vs. Phase A granularity.** Phase A operates per-field; whether
  container-like datatypes with many closure fields need any special
  handling beyond straightforward per-field variablization is not yet
  explored.