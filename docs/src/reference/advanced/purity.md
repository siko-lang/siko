title: Purity
layout: reference

# Purity in Siko

Purity in Siko means that a function has no builtin side effects. This is not the same as purity in Haskell or other traditional pure functional languages.

```siko
effect DbQuery {
    fn query(s: SQL) -> Result
}

fn execute_query(q: SQL) {
    DbQuery.query(q);
}

fn main() {
    let q = SQL("SELECT * FROM users");
    execute_query(q);
}
```

`execute_query` is pure in Siko's sense — it calls no function with builtin side effects. The fact that it invokes an effect method is fine: effect methods are holes whose implementation is supplied by the caller, so the callee makes no commitment about what they do.

## Purity is a pre-monomorphization concept

Like safety, purity is reasoned about before monomorphization. It is entirely possible that an effect handler injected at the call site introduces side effects — but that is the caller's business, not the library's. A package can be declared pure before you know anything about how its effects will be handled.

## What a pure package guarantees

A package deemed pure by the compiler satisfies all of the following:

- Every function in the package is truly safe — not merely marked `@safe`, but containing no unsafe code anywhere.
- It only calls functions marked `@pure`, which covers pure functions from the standard library (even if those contain internal unsafe code) and from any other package.
- It may call effect methods — these are resolved by the caller, so they carry no purity commitment of their own.

Note that a pure package cannot define `@safe` functions, because `@safe` requires unsafe code inside, and pure packages have none.

## Why this matters

This system makes it possible to statically audit a package. A package deemed pure by the compiler is guaranteed to have no builtin side effects and to call nothing that does. Any side effects that appear at runtime are entirely in the hands of whoever supplies the effect handlers — and the supplier has full, unrestricted control over what those handlers do.

For example, a `TcpConnect` effect could be handled by a function that only connects to a specific host, or one that limits the number of connections, or one that enforces a maximum duration. The library being called has no say in any of this. The caller decides the rules, and the pure library operates within whatever boundaries the caller imposes — without knowing or caring what they are.

Siko's lightweight effect and implicit system supports library authors so they never have to reach for dependency injection frameworks or other workarounds. They call effectful functions and write straightforward, seemingly synchronous code without caring how the caller will actually instantiate their library. Because mocking is trivial with effects and implicits, a pure library can be developed and tested using only pure functions throughout.

