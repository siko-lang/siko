# Siko programming language.

Status: ![](https://github.com/siko-lang/siko/workflows/Master/badge.svg)

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
  - [ ] Associated types
- [X] Instances
- [X] Auto currying
- [X] Iterators
- [ ] Effects
- [ ] Class objects
- [ ] Bigints

## Compiler progress
- [X] Parser
- [X] Name resolver
  - [X] HIR lowering
- [X] Type checker
- [X] MIR lowering
  - [X] Monomorphization
  - [X] Defunctionalization
  - [X] Auto derive instances
  - [ ] Inlining
  - [ ] Effects
  - [ ] Class objects
- [X] Backend
  - [ ] Ownership inference
- [X] Rust transpiler


VSCode support for [siko](https://github.com/siko-lang/siko).