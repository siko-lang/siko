# Closure Lowering V3 Design

This document describes a third closure lowering path. It is intentionally
separate from the existing closure lowering pass and from `ClosureMerge` v2.
The goal is to keep the working implementations intact while building a new
pipeline that is easier to validate visually and avoids the expensive outer
pin-stabilization loop.

The core idea is:

```text
datatype closure slots
  -> function/SCC profiles
  -> profile monomorphization into concrete closure sets
  -> rewrite using those sets
```

Profiles are caller-facing summaries. A profile should contain exactly the
information a caller needs to calculate its own closure sets after calling a
function, not a dump of every private thing the callee does internally.

## Design Goals

- Do not modify or destabilize the first closure lowering pass.
- Do not modify or destabilize `ClosureMerge` v2 while v3 is being built.
- Run the analysis in small phases that can each be dumped and inspected.
- Keep function bodies processed once by the profile builder.
- Replace the v2 outer pin-stabilization loop with monotone profile execution.
- Make lambda identity stable. A lifted lambda function name is the globally
  unique id for the closure value introduced by its `CreateClosure` site.
- Make all equality visible in profile syntax by reusing the same closure infer
  variable name, instead of requiring a separate equality section.

## Pipeline Overview

```text
Program after monomorphization
        |
        v
Phase 1: datatype closure infer var extension
        |
        v
Phase 2: function/SCC profile building
        |
        v
Phase 3: profile mono / closure set resolution from Main.main
        |
        v
Phase 4: concrete rewrite, enums, handlers
```

Every phase must have a debug dump. The intended development style is to build
one phase, validate its dump on small fixtures, and only then move to the next
phase.

## Phase 1: Datatype Closure Infer Vars

This phase is the same conceptual step as `ClosureMerge` v2
`GeneralizeDataTypes`.

The program is already post-monomorphization, but datatypes that contain
closure-bearing fields become generic again over closure infer variables.

Example:

```text
struct Box {
    f: fn(Int) -> Bool
}
```

becomes:

```text
struct Box[?0] {
    f: fn(Int) -> Bool/?0
}
```

For datatype SCCs, the SCC gets processed as one unit. Mutually recursive
datatypes share the closure infer pool needed to represent all closure-bearing
slots reachable inside that SCC.

### Phase 1 Dump

The dump should show only the datatype result of this phase:

```text
=== ClosureLoweringV3: datatype closure vars ===

struct Main.Box[?0] {
    f: fn(Int) -> Bool/?0
}

enum Main.Tree[?0, ?1] {
    Leaf(fn() -> ()/?0)
    Node(Main.Pair[?1])
}
```

This dump answers: "Which datatype slots can contain closures, and which infer
vars name those slots?"

## Phase 2: Function/SCC Profile Building

Functions are processed in static call-graph SCC order. Inside one SCC, the
whole SCC is analyzed as a unit.

For each function, the profile builder first instantiates every type occurrence
with fresh closure infer variables:

- function arguments,
- function result type,
- function yield type,
- expression types in the whole body,
- pattern types in the whole body,
- datatype references introduced by Phase 1.

After this, every place that can contain a closure has a visible infer variable.

Then the profile builder runs unification over the instantiated body:

- assignments and lets unify source and destination slots,
- struct and enum constructors unify argument slots with datatype pool slots,
- field access and destructuring unify datatype pool slots with extracted slots,
- returns unify returned expression slots with result slots,
- yields unify yielded expression slots with yield slots,
- ordinary calls instantiate the callee profile if the callee is already done,
- ordinary calls inside the current SCC wire directly to the callee's current
  instantiated argument/result/yield vars,
- dynamic calls become `apply` profile rules,
- closure creation becomes `contains lambda` profile rules,
- `@closure_alias` functions additionally unify values with the same closure
  shape, including the pointer/`sizeof` cases needed by `std/Common/Ptr.sk`.

At the end of unification, equal closure slots are represented by the same
canonical variable. There is no separate equality section in the profile. The
same `?X` appearing twice is the equality constraint.

### Visual Example

Source-shaped view:

```text
fn foo(arg1: SomeArgType, arg2: SomeArgTypeFoo) -> SomeResultType {
    ...
}
```

After type instantiation:

```text
fn foo(
    arg1: SomeArgType[?0],
    arg2: SomeArgTypeFoo[?1],
) -> SomeResultType[?2, ?3] {
    ...
}
```

After unification:

```text
fn foo(
    arg1: SomeArgType[?0],
    arg2: SomeArgTypeFoo[?1],
) -> SomeResultType[?2, ?0] {
    ...
}
```

The repeated `?0` means the closure slot inside `arg1` and the second closure
slot in the result are the same data-flow class.

If the first result slot receives an internally-created lambda:

```text
profile foo:
  sig:
    (SomeArgType[?0], SomeArgTypeFoo[?1]) -> SomeResultType[?2, ?0]

  rules:
    ?2 contains lambda Main.foo_lambda1(captures: [?1])
```

This says that the caller can instantiate `?0`, `?1`, and `?2` into its own
local closure-set variables, and then run the rule in caller space.

## Profile Contract

A profile is a caller-facing constraint recipe.

It is not required to describe private closure islands that do not affect the
caller's closure sets. If a function creates a closure and calls it entirely
internally, and none of the closure effects reach arguments, result, yield, or
another caller-visible rule, the public profile can omit that island.

The profile must include enough information for the caller to compute all
closure sets that can appear in caller-visible slots.

Conceptually:

```text
Profile {
    signature: function signature with canonical closure infer vars,
    locals: profile-local vars needed by rules,
    rules: Vec[ProfileRule],
}
```

The signature carries equality by variable reuse:

```text
(Box[?A], State[?B]) -> Pair[?A, ?C]
```

means the closure slot in `Box` and the first closure slot in `Pair` are the
same class when the profile is instantiated.

Profile-local vars are existential variables. Each call-site instantiation gets
fresh caller-local classes for them.

```text
profile foo:
  sig:
    (fn() -> Box[?F]) -> Box[?R]

  locals:
    ?T

  rules:
    apply ?F(args: [], results: [?T])
    ?R contains lambda Main.wrap_lambda(captures: [?T])
```

## Profile Rules

### Contains Lambda

```text
?Target contains lambda Main.some_lambda(captures: [?A, ?B, ...])
```

When instantiated by a caller, this adds the lambda qname to the caller's
closure set for `?Target`. The capture list records which caller-local closure
sets feed the lifted lambda's capture arguments.

The lambda qname is the stable closure identity. Capture sets are data attached
to the profile-rule instantiation and grow monotonically; they are not used to
mint a new identity.

### Dynamic Apply

```text
apply ?Callee(args: [?A0, ?A1, ...], results: [?R0, ?R1, ...])
```

When instantiated by a caller, this means:

1. Look at every lambda currently in the caller-local closure set for
   `?Callee`.
2. Instantiate that lambda function's profile.
3. Map capture vars from the lambda's recorded capture sets.
4. Map non-capture argument vars from `args`.
5. Map result vars into `results`.
6. Keep iterating until caller-local closure sets stop growing.

This is the summary form for:

```text
if you give this function an argument with lambda set ?Callee,
this function calls it,
and closures returned by that call flow into ?R0/?R1.
```

### Static Call Profile Instantiation

Static calls do not need to remain as profile rules after Phase 2 unless that
turns out to be useful for debugging. The profile builder can instantiate the
callee profile immediately:

```text
callee profile:
  sig:
    (Box[?X]) -> Box[?Y]
  rules:
    ?Y contains lambda L(captures: [?X])

caller call:
  let r: Box[?B] = callee(a: Box[?A])

instantiated into caller profile:
  ?B contains lambda L(captures: [?A])
```

For calls inside the current function SCC, the SCC is processed as a unit.
Intra-SCC calls wire directly to the callee's current instantiated signature
vars. The SCC result is then projected into one published profile per function.

## Phase 2 Dumps

The profile builder should have at least three dumps.

### Instantiated Function Body

This dump happens after fresh type instantiation but before unification:

```text
=== ClosureLoweringV3: instantiated function Main.foo ===

fn Main.foo(
    arg1: SomeArgType[?0],
    arg2: SomeArgTypeFoo[?1],
) -> SomeResultType[?2, ?3] {
    let x: Box[?4] = ...
    ...
}
```

This answers: "Where can closures be present in this function?"

### Unified Function Body

This dump happens after all unification and `@closure_alias` rules:

```text
=== ClosureLoweringV3: unified function Main.foo ===

fn Main.foo(
    arg1: SomeArgType[?0],
    arg2: SomeArgTypeFoo[?1],
) -> SomeResultType[?2, ?0] {
    let x: Box[?2] = ...
    ...
}
```

This answers: "Which of those places became the same data-flow class?"

### Readable Profiles

This dump is the published caller-facing profile:

```text
=== ClosureLoweringV3: profiles ===

fn Main.foo:
  sig:
    (SomeArgType[?0], SomeArgTypeFoo[?1]) -> SomeResultType[?2, ?0]

  rules:
    ?2 contains lambda Main.foo_lambda1(captures: [?1])

fn Main.call_and_return:
  sig:
    (fn() -> Box[?0]/?1) -> Box[?2]

  rules:
    apply ?1(args: [], results: [?2])
```

This answers: "What does a caller need to know?"

## Phase 3: Profile Mono / Closure Set Resolution

Phase 3 starts at `Main.main` and executes profiles, not function bodies.

Each function instance is keyed by:

```text
(function qname, concrete closure sets for canonical boundary vars)
```

For an instantiated profile, Phase 3 allocates caller-local closure set cells
for:

- every canonical boundary var in the profile signature,
- every profile-local var,
- every capture-set binding introduced by `contains lambda` rules.

Then it runs the profile's rules until no set grows.

This phase still has a fixpoint, but it is local and monotone:

```text
sets only grow
rules are summary rules
function bodies are not reprocessed
computed sets are not wiped
closure identities are not re-derived from capture contents
```

### Closure Identity In V3

Each `CreateClosure` site has a lifted lambda function qname. That qname is the
stable lambda identity in closure sets:

```text
{Main.foo_lambda1, Main.bar_lambda0}
```

The capture sets for a lambda are solved as data:

```text
Main.foo_lambda1 captures:
  capture slot 0 = {Main.make_lambda0}
  capture slot 1 = {}
```

Capture sets may grow during Phase 3. Growing captures does not invalidate the
lambda identity and does not require a reset.

### Apply Execution

For an apply rule:

```text
apply ?F(args: [?A], results: [?R])
```

Phase 3 repeatedly does:

```text
for each lambda L in set(?F):
    instantiate L's profile
    bind L capture vars from L's solved capture sets
    bind L normal args from ?A
    bind L result vars into ?R
```

If this adds new lambdas to `?R`, or to any capture set, dependent rules are run
again.

## Phase 3 Dumps

Phase 3 needs visual dumps that make the concrete propagation easy to audit.

### Function Instances

```text
=== ClosureLoweringV3: function instances ===

Main.main#0:
  boundary:
    none

Main.foo#0:
  ?0 = {Main.a_lambda}
  ?1 = {}
```

### Rule Firings

Optional but useful on small tests:

```text
=== ClosureLoweringV3: rule trace Main.foo#0 ===

?2 contains lambda Main.foo_lambda1(captures: [?1])
  ?1 = {Main.a_lambda}
  added Main.foo_lambda1 to ?2

apply ?3(args: [?0], results: [?4])
  ?3 = {Main.cb_lambda}
  instantiated Main.cb_lambda#0
  added Main.returned_lambda to ?4
```

### Final Closure Sets

```text
=== ClosureLoweringV3: closure sets ===

Main.foo#0:
  ?0 = {Main.a_lambda}
  ?1 = {}
  ?2 = {Main.foo_lambda1}

captures:
  Main.foo_lambda1 in Main.foo#0:
    capture0 = {}
```

### Final Enum Groups

```text
=== ClosureLoweringV3: closure enums ===

closure_enum_0:
  {Main.foo_lambda1}

closure_enum_1:
  {Main.a_lambda, Main.b_lambda}
```

This dump answers: "Which runtime closure enums will be needed?"

## Phase 4: Rewrite

Only after Phases 1-3 have been validated should v3 rewrite the program.

The rewrite uses the resolved closure sets:

- closure-typed slots become concrete closure enum types,
- datatype references become concrete datatype instances,
- function calls are retargeted to the correct function instance,
- `CreateClosure` becomes the corresponding closure enum variant constructor,
- `DynamicCall` becomes a generated handler call,
- handlers dispatch by lambda qname variants.

This phase should reuse concepts from the existing lowerers, but remain in a
separate v3 module path until the implementation is trusted.

## Proposed Module Shape

Exact names can change, but the v3 implementation should stay separate:

```text
siko/ClosureLoweringV3/GeneralizeDataTypes.sk
siko/ClosureLoweringV3/Profile.sk
siko/ClosureLoweringV3/ProfileBuilder.sk
siko/ClosureLoweringV3/ProfileDump.sk
siko/ClosureLoweringV3/ProfileMono.sk
siko/ClosureLoweringV3/ProfileMonoDump.sk
siko/ClosureLoweringV3/Rewrite.sk
siko/ClosureLoweringV3/Lower.sk
```

The first implementation can reuse v2 datatype generalization directly or copy
it into the v3 namespace if isolation is more important than deduplication.

## Debug Flags

The preferred debug surface is one high-level flag plus narrower phase flags:

```text
--lower-closure-v3
--debug-closure-v3
--debug-closure-v3-datatypes
--debug-closure-v3-profiles
--debug-closure-v3-mono
```

The broad flag can enable all v3 dumps during development. The narrow flags are
useful once the dumps become large.

## Phased Implementation Plan

### Phase A: Datatype Generalization And Dump

Build only Phase 1 and its dump.

Validation:

- direct closure fields get vars,
- nested datatype references get instantiated,
- datatype SCCs get stable pools,
- no function body is touched.

### Phase B: Function Type Instantiation Dump

Instantiate all function-local types with fresh closure vars and dump the body.

Validation:

- args/result/yield/body annotations all contain visible vars,
- datatype references use fresh vars per occurrence,
- variable numbering is readable and stable enough for debugging.

### Phase C: Unification Dump

Run body unification and dump the unified body.

Validation:

- assignments and returns collapse expected vars,
- constructors/field access/destructuring use datatype slot tables correctly,
- `@closure_alias` collapses the pointer cases in `std/Common/Ptr.sk`,
- intra-SCC calls wire directly.

### Phase D: Profile Dump

Project unified function/SCC data into caller-facing profiles and dump them.

Validation:

- repeated vars encode equality,
- private islands are omitted,
- closure creation that reaches a boundary-visible class becomes a
  `contains lambda` rule,
- dynamic calls that reach boundary-visible classes become `apply` rules,
- static callee profiles are instantiated into caller profiles.

### Phase E: Profile Mono Dump

Execute profiles from `Main.main` and dump concrete closure sets. Do not rewrite
yet.

Validation:

- closure sets match hand expectations,
- dynamic applies instantiate lambda profiles,
- capture sets grow without identity resets,
- recursive/cyclic profiles stabilize,
- no function body is reprocessed in this phase.

### Phase F: Rewrite

Emit concrete functions, datatypes, closure enums, and handlers.

Validation:

- generated program typechecks,
- handlers dispatch to the expected lambda function instances,
- v3 output passes existing closure/coroutine tests,
- v1 and v2 behavior remains available for comparison.

## Open Questions To Validate

### Capture Context

The design uses the lambda qname as the stable closure-set member. Capture sets
are solved as data attached to an instantiated `contains lambda` rule.

We must validate cases where the same lifted lambda qname is created through
different function instances with different captured closure sets. The likely
answer is that the lambda id remains the same, while the capture binding is
contextual to the producing function instance and the enum/handler context.

### Enum Keys

The simplest enum key is a sorted set of lambda qnames:

```text
{Main.a_lambda, Main.b_lambda}
```

If capture payload types for the same lambda set differ by context, the rewrite
may need either:

- context-specific enum instances, or
- joined capture payload sets for the shared enum.

This should be decided from concrete dumps before rewrite work starts.

### Profile Projection

The public profile should omit private islands, but profile-local vars are
needed when an internal dynamic call connects boundary-visible effects.

The projection rule should be:

```text
keep boundary vars
keep local vars reachable from boundary-visible rules
drop all other local-only classes and rules
```

This needs validation with examples where an internal temporary sits between a
dynamic call and the result.

### Bodyless Functions

Externs and builtins still need profiles. The initial policy can match v2:

- conservative all-boundary-vars-connected profile by default,
- precise model for `resume`,
- explicit models for pointer helpers marked by `@closure_alias` where needed.

## Success Criteria

The v3 analysis is ready for rewrite work when the dumps can visually explain:

- where datatype closure vars came from,
- where function body closure vars came from,
- which vars unified,
- what each public profile promises to callers,
- how `Main.main` profile execution produces final closure sets,
- which lambdas and capture sets feed each generated closure enum.

