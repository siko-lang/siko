# Closure Lowering

Siko supports first-class closures: anonymous functions that capture variables from their enclosing scope and can be passed around as values. The target language (C) has no equivalent concept, so the compiler must transform every closure into a plain struct and a plain function before emitting C. This document describes that transformation.

## The source model

A closure is written inline as a lambda expression, for example:

```siko
let threshold = 10;
let f: fn(Int) -> Bool = |x| x > threshold;
```

The lambda `|x| x > threshold` captures `threshold` from its environment and has type `fn(Int) -> Bool`. Values of function type can be called dynamically — the callee is not known at the call site, only its signature is.

## The core problem

A function-type value such as `fn(Int) -> Bool` can be satisfied by many different lambdas across the program, each with a different set of captured variables. C has no way to represent "a callable thing with arbitrary hidden state". The standard solution — and what this pass implements — is to represent each function type as a tagged union (enum) whose variants carry the captures, paired with a dispatch function that unpacks the right variant and calls the right lambda.

## Phase 1: enum and handler assignment

The pass makes a single traversal of every type, function signature, and expression in the program. Whenever it encounters a function type `fn(A, B) -> R` it allocates a *closure enum* for that specific signature (one enum per unique signature, lazily on first encounter). Each closure enum gets:

- A unique name (`closure_0`, `closure_1`, …)
- A paired *handler* function name (`closure_handler_0`, …)

The function type itself is replaced everywhere by the named enum type. This includes struct fields, function argument types, and return types, so the whole program becomes consistent.

## Phase 2: lambda site rewriting

When the pass encounters a lambda creation expression it:

1. Looks up (or creates) the closure enum for the lambda's function type.
2. Allocates a new *variant* inside that enum, named `Variant0`, `Variant1`, etc. The variant's payload fields hold exactly the captured variables, in order.
3. Replaces the lambda creation expression with a constructor call that wraps the captured values into the new variant.

For example, the lambda `|x| x > threshold` that captures `threshold: Int` becomes a call like `closure_0::Variant3(threshold)`.

## Phase 3: call site rewriting

A dynamic call `f(arg)` — where `f` is a closure value — is rewritten into a call to the handler function: `closure_handler_0(f, arg)`. The callee enum and the original arguments are all passed to the handler.

## Phase 4: handler generation

After the whole program has been rewritten, the pass generates the actual enum definitions and handler functions.

Each **closure enum** is a standard tagged union with one variant per lambda of that signature found in the program:

```
enum closure_0 {
    Variant0(Int),      // captures: threshold
    Variant1(String),   // captures: prefix
    ...
}
```

Each **handler function** is a match that dispatches to the right lambda:

```
fn closure_handler_0(closure: closure_0, x: Int) -> Bool {
    match closure {
        Variant0(threshold) => lambda_42(threshold, x),
        Variant1(prefix)    => lambda_99(prefix, x),
        ...
    }
}
```

The lambda functions themselves are already plain functions (the typechecker lifts them out as named functions before this pass); the handler simply calls them with the unpacked captures prepended to the original arguments.

## Key properties

**One enum per signature.** All lambdas with the same `fn(A, B) -> R` type share a single enum and a single handler, regardless of how many such lambdas exist or where they appear. Adding a new lambda of an existing type just adds one more variant to the existing enum and one more arm to the existing handler.

**No boxing.** Captures are stored directly as enum payload fields. There is no heap allocation involved in creating or calling a closure beyond whatever the GC already provides for reference types.

**No function pointers in the enum.** The dispatch is entirely via pattern matching, not a stored function pointer. This is intentional: it keeps the representation uniform and lets the C compiler optimise the dispatch as a switch.

**Consequence for coroutine classification.** Because all lambdas of the same type share one handler, if *any* lambda of type `fn() -> ()` is a coroutine (or calls one), the shared handler becomes a coroutine, and by extension every other lambda of that type does too. This is a known limitation that a future dataflow-based closure splitting pass will address.
