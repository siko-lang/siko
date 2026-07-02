# Coroutine Lowering

Siko lets you write suspendable functions using `co` and `yield`. This document describes how the compiler turns those high-level constructs into ordinary C functions, which is the hardest transformation in the pipeline.

## The source model

A function declared `co(Y) -> R` is a coroutine that yields values of type `Y` and eventually returns a value of type `R`. The caller creates a coroutine object with `co f(args)`, drives it by calling `resume(co_obj)`, and receives a `CoResult` on each call:

- `Yielded(y)` — the coroutine suspended and produced a yield value
- `Returned(r)` — the coroutine finished and produced its return value
- `Completed` — the coroutine was resumed after already returning (illegal)

A function that calls a coroutine without explicitly managing `resume` is a *transitive coroutine*: it implicitly propagates suspension upward to its own caller.

## Phase 1: classification

The compiler builds a call graph and runs a topological analysis (Tarjan's SCC algorithm) to classify every function as either a coroutine or a plain function. A function is a coroutine if it contains a `yield` expression directly, or if it calls any function that is already a coroutine. This classification is the only step that touches the call graph globally; everything after it works function-by-function.

## Phase 2: transitive expansion

Functions that are coroutines only because they *call* other coroutines (transitive coroutines) do not contain `yield` in their source. The compiler rewrites every call to a coroutine callee into an inline drive loop:

```
let co_tmp = co callee(args);       // create the coroutine object
loop {
    match resume(co_tmp) {
        Yielded(_) => { yield (); continue; }  // propagate suspension
        Returned(v) => { result = v; break; }  // unwrap the return value
        Completed   => { ... }                 // unreachable
    }
}
```

This makes the callee's suspension visible to the caller's own `resume`, so suspension propagates up the call chain automatically. After this step every coroutine function has explicit `yield` expressions.

## Phase 3: frame generation

A coroutine must survive across multiple `resume` calls, so its local state cannot live on the C call stack. The compiler converts each coroutine function into a heap-allocated *frame struct* that holds:

- All function arguments
- All local variables declared in the body
- A *state* field that records where to resume next

The state is an enum with one variant per suspension point: `Start`, `AfterYield_0`, `AfterYield_1`, …, `Completed`. Each `yield` in the source maps to one `AfterYield_i` variant.

## Phase 4: handler generation

The original coroutine function body is rewritten into a *handler* — a plain C function that takes the frame struct by pointer and returns a `CoResult`. The handler begins with a dispatch:

```
match frame.state {
    Start        => goto seg_0
    AfterYield_0 => goto seg_1
    AfterYield_1 => goto seg_2
    Completed    => return Completed
}
```

Each `goto` target is a labelled segment of the original body. When execution reaches a `yield` expression, the handler stores the next state variant into `frame.state` and returns `Yielded(value)`. When it reaches a `return`, it returns `Returned(value)`. This is a textbook Duff's-device–style coroutine transformation.

## Phase 5: resume generation

The `resume(co)` builtin is replaced with a generated function that:

1. Extracts the current state variant from the coroutine wrapper
2. Dispatches to the appropriate handler for that variant
3. Returns the `CoResult` the handler produced

All coroutines with the same `(yield_type, return_type)` signature share one wrapper type and one `resume` function. Multiple coroutine functions with the same signature become separate *variants* inside the shared state enum, so the dispatcher can tell them apart.

## Summary of generated types

For each unique `(Y, R)` coroutine signature the compiler emits:

| Type | Role |
|---|---|
| `CoWrap` | The user-visible coroutine object; wraps the state enum |
| `CoState` | Enum with one variant per coroutine function of this signature |
| `Frame_fn` | Struct holding the live locals and state for one specific function |
| `FrameState_fn` | Enum encoding the suspension point within that function |

And for each coroutine function:

| Function | Role |
|---|---|
| `handler_fn(frame)` | The rewritten body; returns `CoResult` on each call |
| `resume(co)` | Dispatches to the right handler based on current state |
