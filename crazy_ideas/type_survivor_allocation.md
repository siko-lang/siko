# Type-Survivor Allocation

## Goal

Given a fresh non-value struct/enum:

```siko
let a = createFoo();
// arbitrary code using and moving `a`
// later code cannot possibly observe `a`
```

stack-allocate it when nothing capable of containing it outlives its scope
instance.

This is deliberately a cheap sufficient-condition analysis, not general
ownership inference.

## Type-survivor heuristic

For an allocation site producing `Foo` in scope `S`, collect the types of
everything that can survive `S`:

```text
SurvivorTypes(S) =
    function argument types
  + function result type
  + types of locals declared in scopes that outlive S
```

Scopes are lexical block instances. A loop-body instance ends at the back edge,
so locals outside the loop survive an allocation inside it.

Then ask:

```text
can_contain(T, Foo)
```

This follows the fields of structs and payloads of enums transitively. Recursive
datatypes use a visited set. The program is monomorphized, and trait objects,
`Error`, `Any`, and similar erasure mechanisms are already explicit
datatypes at this pipeline position.

The decision is:

```text
no survivor type can contain Foo  → Stack
otherwise                         → Heap
```

This pessimistically assumes that `a` may be stored into any surviving
argument/local whose type can hold `Foo`. It does not inspect whether such a
mutation actually happens.

Raw pointers, transmute, unknown FFI, or another representation that can hide
`Foo` bypass the type barrier and force the conservative heap result.

## Fresh results

A constructor is a known fresh root. A function call may also be treated as a
fresh allocation site when the callee is known to return a fresh root.

A small root-only analysis is enough:

```text
constructor            → Fresh(site)
fresh-returning call   → Fresh(call_site)
whole-value assignment → preserve freshness
argument/ordinary call → NotFresh
```

Field use and field mutation do not change the root's freshness.

A function can publish `FreshReturn(Foo)` when:

1. its result type is exactly `Foo`;
2. every possible returned root comes from an eligible fresh site;
3. for each such site, the only surviving type capable of containing it is the
   function result.

This allows complicated creators:

```siko
fn complicatedCreator(many: Args) -> Foo {
    let foo = Foo(...);
    // arbitrary work on `foo`
    foo
}
```

Calls to `complicatedCreator` are then fresh `Foo` sites in their callers.

For a conservative first version, do not apply the result-root exception when
`Foo` can strictly contain another `Foo`. Otherwise this is unsafe:

```siko
let foo = Foo(...);
foo.child = foo;
foo
```

Later, an interior-flow check could recover these cases.

## Caller-provided destinations

Semantically, fresh creators return `Foo` by value. Physically, use destination
passing so the value is never copied:

```text
fn createFoo(out: *Foo, ...) -> *Foo
```

The caller chooses the destination:

```text
Stack:
    declare storage: Foo
    createFoo(&storage, ...)

Heap:
    let storage = Alloc(Foo)
    createFoo(storage, ...)

Forward:
    createFoo(current_function_out, ...)
```

The `Forward` case lets an arbitrary chain of fresh-producing functions avoid
making an allocation decision:

```text
Foo.ctor(out)
        ↑ same pointer
complicatedCreator(out)
        ↑ same pointer
wrapper(out)
        ↑ same pointer
final caller chooses Stack or Heap
```

Functions that may return an existing argument/object cannot use this fresh
destination ABI: copying that object into `out` would change reference
identity.

## Incremental implementation

The first phase needs no heuristic:

1. Change non-value struct/enum constructors to accept a destination pointer
   and never allocate internally.
2. Rewrite every constructor call to allocate on the heap immediately before
   the call and pass that pointer.

This preserves current behavior while moving allocation control to the caller.

Subsequent phases:

1. implement `can_contain(T, Foo)` and lexical survivor collection;
2. choose stack storage for directly local constructor sites;
3. add root freshness and `FreshReturn(Foo)`;
4. propagate caller destinations through fresh-producing functions.

The resulting analysis is function-local and site-centric. It needs no
datatype inference variables, function profiles, pointer sets, or
whole-program ownership graph.
