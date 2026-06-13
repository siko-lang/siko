title: Traits & Instances
layout: reference

# Traits & Instances

How Siko does polymorphism.

---

## Traits

A trait declares a named interface — a set of methods that any type can implement. There is no implicit self type; you must explicitly declare the type parameter and pass it to each method.

```siko
trait Greet[T] {
    fn greet(t: T)
}
```

The type parameter in `[...]` is the type the trait is implemented for. Multiple type parameters are allowed:

```siko
trait Into[T, U] {
    fn into(value: T) -> U
}
```

### Required and default methods

A method with no body is **required** — every instance must provide it. A method with a body is a **default** — instances inherit it and can optionally override it.

```siko
trait Iterator[T] {
    type Item

    fn next(i: T) -> Option[Item]           // required

    fn enumerate(i: T) -> EnumerateIterator[T] {   // default
        EnumerateIterator(i)
    }
}
```

### Associated types

Use `type Name` inside a trait to declare an associated type — a type that each instance fills in:

```siko
trait Add[T] {
    type Output
    fn add(a: T, b: T) -> Output
}
```

Instances assign it with `type Output = ...`.

---

## Instances

An `instance` provides the implementation of a trait for a specific type:

```siko
struct Point { x: Int, y: Int }

instance Greet[Point] {
    fn greet(t: Point) {
        println("hi from ${t.x}, ${t.y}")
    }
}
```

### Canonical vs named instances

The presence or absence of a name controls scoping and coherence.

**Canonical (unnamed) instances** are always in scope everywhere in the program — you never have to import them. This gives global coherence: one implementation of a trait for a given type, visible to all code.

```siko
instance ToString[Int] {
    fn to_string(a: Int) -> String { ... }
}
```

**Named instances** are only in scope where they are explicitly imported (or in the module that defines them). They allow specialization — a more specific implementation for a particular type that would otherwise be covered by a generic canonical instance. For example, you might have a canonical `ToString[Vec[T]]` that works for any element type, and a named `ToString[Vec[MyFoo]]` that formats `MyFoo` elements differently:

```siko
// canonical — covers all Vec[T], always in scope
instance ToString[Vec[T]] {
    fn to_string(v: Vec[T]) -> String { ... }
}

// named — specialized for Vec[MyFoo], only in scope when imported
instance MyFooVecToString ToString[Vec[MyFoo]] {
    fn to_string(v: Vec[MyFoo]) -> String { ... }
}
```

When `MyFooVecToString` is in scope it takes priority over the canonical instance for `Vec[MyFoo]`. Where it isn't imported, the canonical instance is used. You control where the specialization applies through the import system.

### Instance resolution order

When the compiler resolves which instance to use, it tries three levels in order, stopping as soon as it finds a match:

1. **Local instances** — defined in the current module. Ambiguous → error.
2. **Imported named instances** — brought in explicitly. Ambiguous → error.
3. **Canonical instances** — unnamed, globally available. Ambiguous → error.

If nothing is found at any level, it's a compile error. Local always beats imported, which always beats canonical.

### Generic instances

Instances can be generic — they implement a trait for all types that satisfy some bounds:

```siko
instance[T] Into[T, T] {
    fn into(value: T) -> T { value }
}
```

Bounds on type parameters are written after the colon. Multiple bounds are separated by commas:

```siko
instance[T: Foo[T], Bar[T]] MyFooForQ Foo[Q[T]] {
    fn foo(q: Q[T]) {
        bar(q.t);
    }
}
```

### Assigning associated types

When a trait has an associated type, instances assign it with `type Name = ConcreteType`:

```siko
instance Add[Int] {
    type Output = Int
    fn add(a: Int, b: Int) -> Int { a + b }
}
```

---

## Using trait methods

Once a trait is in scope (either defined locally or imported), you can call trait methods directly — no qualifying needed:

```siko
fn main() {
    let p = Point(x: 1, y: 2);
    greet(p);       // calls Greet.greet
}
```

The compiler resolves which instance to use based on the argument or result types.

### Trait bounds on functions

To write a function that works for any type with a given trait, add a bound in `[...]`:

```siko
fn print_all[T: ToString[T]](items: Vec[T]) {
    for item in items {
        println(item.to_string());
    }
}
```

Multiple bounds:

```siko
fn stuff[T: Foo[T], Bar[T]](t: T) {
    foo(t);
    bar(t);
}
```

---

## `@derive`

Common traits can be auto-derived with the `@derive` annotation instead of writing the instance manually:

```siko
@derive(Clone, PartialEq)
struct Point {
    x: Int,
    y: Int,
}
```

The compiler generates the instance for you. Currently the derive implementations are hardcoded in the compiler.

---

## Instances and imports

Canonical (unnamed) instances need no importing — they're always available. Named instances must be imported to be in scope. Importing a module also pulls in all named instances from that module, so you generally get the instances you need automatically when you import a module.
