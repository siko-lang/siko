# Study: User-Overridable Allocators as an Effect

## Motivation

Today, heap allocation in Siko is completely invisible to the user. The compiler
decides what is heap allocated and constructs those objects through
`siko_malloc`, which only materializes at the very end of the pipeline. Nothing
in the language surface can observe or influence it.

The idea: define an `Allocator` effect in the standard library and let users
override the allocator by installing a handler with `with`, like any other
effect. Constructors executing in that dynamic context then allocate through
the user's handler; everywhere else they use the default `siko_malloc`. Since
handlers are ordinary functions monomorphized in the context of their use, an
allocator handler can use implicits, other effects, closures — the full
language.

```siko
effect Allocator {
    fn allocate(size: Int) -> *U8
}

fn my_alloc(size: Int) -> *U8 {
    // count, trace, arena-bump, whatever — full language available
    ...
}

fn main() {
    with Allocator.allocate = my_alloc {
        // every heap construction in this dynamic extent
        // (transitively, through monomorphization) uses my_alloc
    }
}
```

## Where allocation lives today

The relevant pipeline order (`build_full_passes`,
`siko/Compiler/Passes.sk`):

```
check → lambda-lift → lower-match → mono → propagate-implicits
      → lower-closures → lower-coroutines → lower-tuples → lower-builtins
      → value-types → lower-ifs → lower-basic-block → lower-pointers
      → lower-data → llvm
```

Key facts:

- **Heap-ness is decided late.** The `value-types` pass
  (`siko/Compiler/ValueTypes.sk`) computes which types are by-value via an SCC
  analysis over layout dependencies — *after* mono, implicit propagation,
  closure lowering and coroutine lowering (it has to run late because those
  passes create new types: ctx structs, closure structs, coroutine frames).
- **Ctor bodies don't exist until lower-data.** The last pass before the
  backend synthesizes every struct/variant constructor as a real function
  (`generate_struct_ctor` / `generate_variant_ctor` in
  `siko/Compiler/LowerData.sk`) and emits `ExprKind.Alloc(ty)` for non-value
  types (`siko/Common/AST/Expr.sk`, the `Alloc` variant). Before that, ctor
  call sites are just `FunctionCall` with kind `StructCtor`/`VariantCtor` and
  no callee body anywhere.
- **`Alloc` lowers to `siko_malloc(sizeof ty)`** in the LLVM backend
  (`lower_alloc`, `siko/Compiler/LLVM/Lower.sk`), and `siko_malloc` is a
  null-checked `GC_malloc` emitted as a runtime helper
  (`emit_runtime_functions`, same file).

## What the existing machinery already provides

The effect system does most of the heavy lifting for free:

- `MonoContext` (effect handler resolutions + witnesses + implicit set) is
  structurally part of every function's mono identity
  (`siko/Common/AST/QualifiedName.sk`, `MonoContext` / `HandlerInfo` /
  `MonoName`). Two `with` sites binding the same handler with different
  witnesses already produce distinct specializations.
- When an effect method is called, the handler is instantiated **with the call
  site's full mono context** — its own effect resolutions and implicit set
  included (the `EffectMethod` arm of `process_user_defined_fn`,
  `siko/Compiler/Monomorphizer/Expr.sk`). This is exactly the instantiation
  semantics the allocator needs; the only missing piece is that nothing
  *calls* `allocate`, so nothing triggers the instantiation.

## What has to change

### The core problem: ctors are context-blind

Ctor call sites are keyed by `data_mono_key`
(`siko/Compiler/Monomorphizer/Mono.sk`), which deliberately strips effect
handlers and implicits from the context (keeping only data-relevant effect
associated types). All allocator contexts collapse onto a single ctor today.

The fix must split **ctor function identity** (per type-args × allocator
resolution) from **data type identity**, which stays shared — a `Vec[Int]`
built under an arena must be the same *type* as one built under malloc,
otherwise nothing unifies.

### Design decision: the allocator is an attribute of the ctor *call site*

Mono attaches the instantiated allocator's mono qname to the call:

```
FunctionCallInfo {
    ...
    allocator: Option[QualifiedName],   // NEW: instantiated allocator instance
}
```

Every downstream pass keys off this field. The auto-derived
`Rw`/`Collect`/`Rewrite` instances over `FunctionCallInfo` carry the field
along mostly for free; the one thing to watch is closure lowering's
function/datatype duplication remapping the qname consistently.

### Pass-by-pass plan

**1. Std + builtins.** Declare the `Allocator` effect in std; register its
qname in `Constants.Builtins`. There is no "default handler" mechanism and
none is needed: an unresolved `Allocator` at a ctor site is not an error — it
means "use `siko_malloc`", preserved as today's `Alloc` path.

**2. Mono — forced instantiation at ctor sites.** Handlers are only
instantiated at effect-method call sites, and nothing calls `allocate`
explicitly. So `process_struct_ctor` / `process_variant_ctor`
(`siko/Compiler/Monomorphizer/Expr.sk`) become implicit call sites: when
`current_mono_ctx` resolves the Allocator effect, run the same instantiation
as the `EffectMethod` arm — `fn_mono_key(handler, targs, witnesses,
current_mono_ctx)` + enqueue — and stamp the resulting instance qname on the
call's `allocator` field. The `EffectMethod` arm should be factored into a
helper shared by both paths.

Consequences, all intended:

- Instantiation uses the *ctor site's* mono ctx, so one `with` can produce
  several allocator instances if ctors fire in dynamically different contexts
  (extra implicits added below the `with`) — mirroring how effect method
  calls already behave.
- The handler's instantiation context contains its own resolution.
  Self-reference is fine: mono terminates on the `fn_mono_key` cache hit, and
  the handler is treated as a perfectly ordinary function that may allocate,
  read implicits, call APIs, or recurse. No special-casing, no stripping.
  Whether a recursive allocator terminates at runtime is the user's business.

**3. Implicit propagation.** IP currently threads the `_ctx` struct only into
`UserDefined` calls and closures (`rewrite_expr`,
`siko/Compiler/ImplicitPropagation/Rewriter.sk`); ctor calls pass through
bare. New case: a ctor call with an allocator gets the *allocator instance's*
ctx struct prepended as an argument — look up its implicit set in
`fn_implicit_ctx`, build the struct with the existing `build_ctx_arg`, keep
the call kind `StructCtor`/`VariantCtor`.

**4. Closure inference.** Ctors are currently modeled as pure field-flow
(`ctor_edges`, `siko/Compiler/ClosureLowering/InferProfiles/Flow.sk`) with no
call dependency (`CollectDeps.sk`). A ctor call with an allocator must also be
modeled as a *call* to the allocator instance with the ctx arg — otherwise the
ctx struct arg would be treated as flowing into a constructed field and
closure profiles would be wrong for any allocator whose implicits carry
closures. Coroutines and closures are separate colorings; this edge is needed
even under the no-coroutine rule below.

**5. Coroutine lowering — a check, not a transformation.** Design decision:
**allocators must not be coroutines** (enforced, see below). With that rule,
lower-coroutines needs zero transformation work — only a check: after
`build_coroutine_map` (`siko/Compiler/CoroutineLowering/FunctionGroupBuilder.sk`)
produces `is_co_map`, walk ctor calls and error if a call's attached allocator
instance is co-classified. This is the right place for the check because
coloring in Siko is a property of the *mono instance*, discovered
transitively by the SCC prepass — not of any signature (see the echo5 test:
identical `App` code becomes threaded or event-loop purely by handler
choice). The same handler source can be a legal allocator under one `with`
and an illegal (yielding) one under another; the check catches exactly the
offending instantiation and can report both the handler and the ctor
location.

**6. Lower-enums.** Generate per-(data, allocator) ctor variants: retarget
each allocator-carrying ctor call to a ctor function whose identity includes
the allocator instance; the generated body takes the `_ctx` param, calls the
allocator instance with `SizeOf` (the expr kind already exists), and
`Transmute`s the returned `*U8` to the pointee type — in place of
`ExprKind.Alloc`. Calls without an allocator take today's path unchanged, so
`siko_malloc` remains the default.

**7. Tests** (mirroring `test/success/std/effects/*`): counting allocator;
allocator reading an implicit; nested `with` overriding the allocator;
allocator constructing objects itself (self-recursive instantiation);
should-fail case where the installed allocator is (transitively) a coroutine.

### Conservatism around value types

At mono/IP/closure time nobody knows which types end up heap-allocated
(value-types runs later). So allocator-context specialization applies to
*every* ctor call in an allocator context, including ones that turn out to be
value types. Accepted: the cost is dead ctx args on value-type ctors and some
extra specializations — not observable behavior. Crucially, because
allocation stays inside the lower-data-generated ctor bodies, the runtime
semantics remain exact: **the allocator is called iff the type is actually
heap-allocated**. A counting allocator observes precisely the real
allocations.

## The "allocators must not be coroutines" rule

Async allocators are theoretically possible but were deliberately excluded.
The blocker chain, for the record:

- A yielding call must be expanded by `transform_transitive`
  (`siko/Compiler/CoroutineLowering/TransitiveLower.sk`) into co_create + a
  drive loop — but a ctor has no callee body at that point (bodies appear in
  lower-data, long after the frame/state-machine machinery is gone).
- The workable path would be hoisting the allocation to the call site before
  coroutine lowering — rewrite `T(args)` into
  `{ let mem = A.allocate(SizeOf(T)); T#placement(mem, args) }` — with
  lower-data generating non-allocating *placement* ctors.
- But since heap-ness is unknown before value-types, the hoist would call the
  allocator even for types that end up unboxed. Unlike conservative
  *coloring* (harmless), a spurious *runtime call* is observable in a counting
  allocator. Fixing that requires moving the value-types decision for user
  types ahead of lower-coroutines — a real project of its own (in the spirit
  of lower-match-before-mono), with wrinkles around closure-lowering's
  datatype duplication and late-generated types.

The rule deletes this entire tail, keeps semantics exact and predictable, and
costs one clean error message. Classification-side support (ctor→allocator
dep edges in `build_groups` — trivial once the call-site field exists, since
ctor deps are only filtered out today because ctor qnames aren't in
`program.functions`) can be added at any point; under the rule those edges
are no-ops except for powering the check.

## GC interaction: user's responsibility

`siko_malloc` is `GC_malloc` (Boehm). Boehm only scans memory it knows about
(its own heap, stacks, registers, statics): an object handed out by an
allocator sourcing memory elsewhere (arena, pool, mmap) is invisible to the
collector, so a GC-allocated object referenced *only* from such memory can be
collected while still reachable. Note also that only *constructor*
allocations go through the effect — internal buffers still come from the GC
path regardless (e.g. `Vec`'s buffer via `gc_allocate_array`,
`std/Common/Vec.sk`), so mixed-heap objects are the default outcome under a
non-delegating allocator, not an edge case.

This is explicitly **not** a design problem to solve: an allocator that
provides memory from outside the GC is unsafe code, and the rule for unsafe
is that the consequences are on the author. Allocators that delegate to the
default (counting, tracing, limiting) are unconditionally sound; anything
else is the user's own contract to uphold.

## Summary of decisions

| Decision | Choice |
|---|---|
| Allocator identity | Attached per ctor *call site* (`FunctionCallInfo.allocator`), not per data type — type identity stays shared across allocator contexts |
| Handler instantiation | Forced at ctor sites, with the ctor site's full mono ctx (implicits, nested effects), via the shared effect-method instantiation path |
| Self-recursive handlers | Allowed; handler is an ordinary function; mono terminates via key cache |
| Value-type ctors under an allocator ctx | Conservatively specialized (dead ctx arg), but never call the allocator — semantics stay exact |
| Async allocators | Forbidden by rule; checked after coroutine classification, per mono instance |
| Default | No handler in ctx → `Alloc` → `siko_malloc`, unchanged |
