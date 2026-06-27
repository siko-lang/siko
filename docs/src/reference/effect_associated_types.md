title: Effect Associated Types
layout: reference
priority: 95

# Effect Associated Types

An effect can declare a type that is left open and chosen by the handler. Then its
operations may be written against that abstract type; the concrete type is fixed at
the `with` site, alongside the handlers.

```siko
effect MyEff {
    type Foo

    fn create() -> Foo
    fn do(f: Foo)
}
```

`Foo` has no definition inside the effect. `create` produces a `Foo` and `do`
consumes one, but neither says what `Foo` actually is. Code that uses the effect
only knows that whatever `create` returns is the same type `do` accepts.

---

## Binding the type

The associated type is part of the handler, so it is supplied where the handlers
are supplied. There are two ways to do that.

### With individual handlers

Bind the type with `with type`, next to the function handlers:

```siko
struct Boo {}

fn create_handler() -> Boo { Boo() }
fn do_handler(f: Boo) {}

fn main() {
    with type MyEff.Foo = Boo,
        MyEff.create = create_handler,
        do = do_handler {
        let a = create();   // a : Boo
        do(a);
    }
}
```

Inside the block, `Foo` *is* `Boo`. `create()` returns a `Boo`, and you can pass
it back to `do`. The handler signatures must agree with the bound type — here
both handlers work in terms of `Boo`.

### With an instance

An effect instance bundles the type assignment with the handlers in one place:

```siko
instance I MyEff {
    type Foo = Boo

    fn create() -> Boo { Boo() }
    fn do(f: Boo) {}
}

fn main() {
    with MyEff = I {
        let a = create();
        do(a);
    }
}
```

The instance's `type Foo = Boo` provides the binding, so the `with` block only
needs to name the instance.

---

## Using the type

A value of the associated type behaves like any other value inside the block.
You can store it in a struct or enum whose field is declared as the effect's
type:

```siko
struct Q {
    a: Foo,
}

fn main() {
    with MyEff = I {
        let a = create();
        let q = Q(a);
        do(q.a);
    }
}
```

Effect operations can be called in either form — as a free call or as a method
on a value of the associated type:

```siko
with MyEff = I {
    let a = create();
    a.do();        // same as do(a)
}
```

---

## Requiring operations on the type

Sometimes the effect's code needs to *do* something with a value of the abstract
type — compare it, clone it, call a trait method on it. Since the concrete type
is unknown inside the effect, you state up front which traits it must implement.
The bound goes in brackets on the effect:

```siko
trait MyTrait[T] {
    fn some_func(t: T)
}

effect[Foo: MyTrait[Foo]] MyEff {
    type Foo

    fn create() -> Foo
    fn do(f: Foo)
}
```

`Foo: MyTrait[Foo]` means the type chosen for `Foo` must have a `MyTrait`
instance. With that bound in place, `MyTrait`'s methods are available on any
`Foo` value, in both call forms:

```siko
fn main() {
    with type MyEff.Foo = Boo,
        MyEff.create = create_handler,
        do = do_handler {
        let a = create();
        some_func(a);   // free call
        a.some_func();  // method call
        do(a);
    }
}
```

The bound is checked where the type is bound. If the chosen type does not satisfy
it, the `with` block is rejected:

```siko
// with type MyEff.Foo = Boo, ...   where Boo has no MyTrait instance
ERROR: No instance of `Main.MyTrait` found for `[Main.Boo]`
```

When the type needs more than one trait, separate the bounds with commas, the
same as on a function or trait:

```siko
effect[Foo: MyTrait[Foo], ToString[Foo]] MyEff {
    type Foo
    ...
}
```
