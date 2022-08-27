# Siko programming language.

Status: ![](https://github.com/siko-lang/siko/workflows/Master/badge.svg)

## The vision

Siko is an imperative, statically typed, runtime agnostic programming language that has very strong functional flavours.
It should be possible to write most algorithms in Siko without thinking/worrying about the target
environment much and it should just work. Siko does not have a runtime, all its features are compile time,
it does full program type inference, compile time effects and whole program ownership inference.
It should be at least as fast as safe rust. The language currently has a Haskell like syntax but that may be temporary.

## Status

Siko is under development but the real Siko compiler is self hosted, so a large amount of the features are implemented.
It's original compiler written in rust is NOT feature complete, the generated code by the rust compiler is very slow/dumb.
The Siko compiler written in Siko is much smarter and contains most of the features but may contain bugs and the error
messages are not user friendly/very terse. The full program analysis is very computation heavy and the code generated
by the bootstrapping (rust) compiler is very dumb, so the source code of the siko compiler contains various workarounds to hack around it.
When the siko compiler is feature complete enough to fully replace the rust compiler, these workarounds/hacks can be removed.
Currently siko transpiles to rust but there is nothing rust specific in the design, it should be possible to transpile it to
any (mostly imperative) language (including garbage collected ones).

## Building

Dependencies: rust, make, bash, python

Building the compiler written in rust
```
make rust_sikoc
```

Building stage0 (the siko compiler built by the rust compiler, this compiler is very slow)
```
make stage0
```

Building stage1 (the siko compiler built by stage0)
```
make stage1
```

Building stage2 (the siko compiler built by stage1, should be identical to stage1)
```
make stage2
```

## Language features
- [X] Functions
- [X] Algebraic data types
- [X] Tuples
- [X] Records
- [X] Field access
- [X] Pattern matching
  - [X] String literals
  - [X] Integer literals
  - [X] Char literals
  - [X] Char ranges
  - [X] Bindings
  - [X] ADT and record destructuring
  - [X] Or patterns
  - [X] Tuple patterns
  - [X] Wildcard
  - [X] Guards
  - [ ] Integer ranges
  - [ ] List patterns
- [X] Type classes
  - [X] Single parameter type classes
  - [X] Dependencies
  - [X] Associated types
- [X] Instances
- [X] Auto currying
- [X] Iterators
- [X] Effects
- [ ] Class objects
- [ ] Big integers
- [ ] Type aliases
- [ ] Builtin tests
- [ ] Interpreter

## Compiler progress
- [X] Parser
- [X] Auto derive instances
- [X] Name resolver
  - [X] HIR lowering
     - [X] Effects
     - [X] Inlining
- [X] Type checker
- [X] MIR lowering
  - [X] Monomorphization
  - [X] Defunctionalization
  - [ ] Class objects
- [X] Backend
  - [X] Ownership inference
- [X] Rust transpiler

## Tooling progress
- [] LSP Server
- [] Auto formatter
- [] Package manager


VSCode support for [siko](https://github.com/siko-lang/siko).