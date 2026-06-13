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

## Declaring a package as pure

Add `pure = true` to the package's `package.toml`:

```toml
name = "MyLib"
version = "1.0.0"
pure = true

[dependencies]
Std = "0.1.0"
```

The compiler will verify this claim. If any function in the package is impure the build fails with an error.

## The `@pure` annotation

A function marked `@safe` contains internal unsafe code but promises callers that it is safe to call. Sometimes such a function also makes a purity promise — it performs no observable side effects despite its internal mechanics. In that case it can be additionally annotated `@pure`:

```siko
@safe
@pure
pub fn fast_copy(dst: Ptr, src: Ptr, len: Int) {
    // unsafe memory copy internally, but no side effects visible outside
    ...
}
```

`@pure` is only valid on `@safe` functions. The compiler enforces this: annotating a function `@pure` without `@safe` is an error.

The annotation acts as an unconditional trust boundary for the purity analysis: callers treat the function as pure without inspecting its body. This is what allows `Vec.new()` and `Vec.push()` — which internally call unsafe code — to be treated as pure by code that uses them.

## What a pure package guarantees

A package deemed pure by the compiler satisfies all of the following:

- Every function in the package contains no unsafe code — no `@safe` or `@unsafe` annotations, no extern or builtin calls — unless it is also annotated `@pure`, in which case it is trusted.
- It only calls functions that are themselves computed to be pure (or explicitly annotated `@pure`).
- It may call effect methods freely — these are resolved by the caller, so they carry no purity commitment of their own.

## Handlers and purity

While *calling* an effect method is always pure, *supplying an impure function as a handler* is not. The handler runs inside the caller's scope, so its side effects belong to the caller:

```siko
// LibA — not pure: greet calls println
pub fn greet(msg: String) {
    println(msg);
}

// MyApp — also not pure: it supplies an impure handler
fn main() {
    with Logger.log = LibA.greet {
        do_work();
    }
}
```

If `MyApp` declared `pure = true` the compiler would reject it, because `LibA.greet` is impure and `main` uses it as a handler.

## Why this matters

This system makes it possible to statically audit a package. A package deemed pure by the compiler is guaranteed to have no builtin side effects and to call nothing that does. Any side effects that appear at runtime are entirely in the hands of whoever supplies the effect handlers — and the supplier has full, unrestricted control over what those handlers do.

For example, a `TcpConnect` effect could be handled by a function that only connects to a specific host, or one that limits the number of connections, or one that enforces a maximum duration. The library being called has no say in any of this. The caller decides the rules, and the pure library operates within whatever boundaries the caller imposes — without knowing or caring what they are.

Siko's lightweight effect and implicit system supports library authors so they never have to reach for dependency injection frameworks or other workarounds. They call effectful functions and write straightforward, seemingly synchronous code without caring how the caller will actually instantiate their library. Because mocking is trivial with effects and implicits, a pure library can be developed and tested using only pure functions throughout.

## Inspecting purity verdicts

Pass `--dump-package-purity` to print the computed verdict for every package in the build:

```
PURE: MyLib
IMPURE: MyApp
```

This is useful when tracking down why a package that should be pure is not.
