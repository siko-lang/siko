# Siko programming language.

Status: ![](https://github.com/siko-lang/siko/workflows/Master/badge.svg)

## The vision

Siko is an imperative, statically typed, runtime agnostic programming language that looks and acts like a functional language.
It should be possible to write most algorithms in Siko without thinking/worrying about the target
environment much and it should just work. Siko does not have a runtime, all its features are compile time.
It does full program type inference, compile time effects and whole program ownership inference.
It should be at least as fast as its target language. Currently only Rust is supported as a target language,
theoretically any imperative language would work (or even LLVM/WASM). The language currently has a Haskell like syntax.

## Current state of the project

Siko is heavily under development and its compiler is self hosted. The error messages are not user friendly/very terse.
The full program analysis is very computation heavy and it is a work in progress to make it quick enough for everyday programming.
Currently Siko transpiles to Rust but there is nothing Rust specific in the design, it should be possible to transpile it to
any (mostly imperative) language (including garbage collected ones).
Development is happening on the master branch and as long as there are no external contributors it will remain this way.
The master branch is sometimes broken in various ways.

## Building instructions

Dependencies: Rust, make, bash, python

Building the compiler from the generated Rust code
```
make stage0
```

Building stage1 (the Siko compiler built by stage0)
```
make stage1
```

Building stage2 (the Siko compiler built by stage1, should be identical to stage1)
```
make stage2
```

Run tests
```
make test
```

## Documentation

The documentation is basically non existent. There are a few unfinished attempts at writing various versions of the documentation
in the doc folder, check them out for a peek into my brain.

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
- [ ] LSP Server
- [ ] Auto formatter
- [ ] Package manager


## Contributors

The project is too ambitious and it is very unlikely that I can finish it alone so the most important help
the project needs is actual work. You can select anything on the roadmap and implement it or just help fix bugs
or clean up the compiler.

## License

MIT

## VSCode support

Very crude VSCode support for [Siko](https://github.com/siko-lang/siko), only contains syntax highlight.