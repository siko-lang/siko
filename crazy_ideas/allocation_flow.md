# Allocation Flow

## Goal

After lowering, the safe subset of Siko is mostly structs, enums, and
functions. At that level, allocation can be decided by tracking where each
abstract pointer can flow.

```text
construct value ──► local slots ──► function boundaries
       │
       └── stack / heap / unknown
```

This is similar to closure inference, with one important change:

```text
closure inference: slots are unified       ?1 = ?2
allocation flow:  flow is directional      ?1 → ?2
```

## Symbolic datatype slots

Give every pointer-carrying position in a datatype an inference variable.
Process recursive datatype SCCs together, as in closure lowering.

```siko
struct Holder {
    value: Foo
}
```

Conceptually:

```text
Holder<?value>
```

`?value` describes the pointer provenance that can occupy that field. Recursive
datatypes may produce many such variables, but they remain symbolic at this
stage.

## Symbolic function profiles

Process function SCCs together. Inside an SCC, build directed flow links
directly. Outside the SCC, instantiate the already completed callee profile.

```siko
fn choose(a: Foo, b: Foo) -> Foo {
    if condition() { a } else { b }
}
```

Profile:

```text
?a → ?result
?b → ?result
```

Profiles contain only links visible to callers. Local variables are projected
away:

```text
?arg → ?local1 → ?local2 → ?result

becomes

?arg → ?result
```

After SCC processing, every function has a compact, allocation-independent
transfer profile.

## Concrete analysis stays per function

Do not create one program-wide graph containing every allocation. Once all
profiles exist, analyze functions independently.

For each local inference variable, keep the literal set of abstract pointers
that it may contain:

```text
?1 = {pFoo}
?2 = {pFoo, pBar}
?3 = {pFoo, UnknownPtr}
```

A link performs set propagation:

```text
?1 → ?2    means    contents(?1) ⊆ contents(?2)
```

At a call, instantiate only the callee's boundary profile as links between the
caller's local slots:

```text
caller ?7 ──► callee ?arg
callee ?result ──► caller ?9

using ?arg → ?result becomes:

caller ?7 ──► caller ?9
```

Seed the function's allocations, propagate the sets, make its allocation
decisions, then discard the sets. Callee bodies never join the caller's graph.

## Unsafe operations and the black hole

FFI, transmute, unknown function pointers, and similar unsafe operations are
exceptions. A known pointer entering such an operation goes into a black hole:

```text
?3 = {pFoo}
?3 → BlackHole

result for pFoo: Dunno
```

Only that pointer path becomes undecidable. A pointer coming *from* the black
hole is represented independently:

```text
BlackHole → ?4
?4 = {UnknownPtr}
```

`UnknownPtr` does not alter knowledge about other pointers. This is valid:

```text
?5 = {pFoo, UnknownPtr}
```

`pFoo` may still be stack-safe if every slot containing `pFoo` dies within its
owner's lifetime. The unrelated unknown pointer does not poison it.

## Allocation follows the value

The useful consequence is that crossing a function boundary does not
automatically require heap allocation.

```siko
struct Foo {
}

fn some_func() -> Foo {
    Foo()
}

fn bar() {
    let f = some_func();
}
```

Inside `some_func`, suppose the `Foo()` result reaches only the function's
result slot:

```text
pFoo ──► ?result
```

There is no surviving alias that requires the original object's address.
`some_func` can therefore return `Foo` by value. It does not choose heap or
stack storage for its caller.

```text
some_func                    bar

construct Foo ──by value──► choose storage
                             ├── keep on bar's stack
                             └── heap-allocate if bar's uses require it
```

Dead aliases inside `some_func` do not matter. What matters is whether every
slot that can contain `pFoo` is bounded by the lifetime of the variable
originally owning that allocation, and whether `pFoo` reaches a black hole.

```text
all containing slots die in time    → stack-safe
pointer reaches a longer-lived slot  → allocation required
pointer reaches BlackHole            → Dunno
```

Allocation is therefore demand-driven: construction produces a value, while
the context that needs stable or longer-lived storage decides to allocate it.
