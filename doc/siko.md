# The Siko programming language

## ! This is a work in progress, unfinished documentation. !

## Introduction

`Siko` is a strict, impure and mostly imperative programming language that visually and sometimes semantically resembles a functional language. All `Siko` programs are function applications transforming immutable values. In `Siko` there are no mutable variables, everything is a value, there are no references. It features heavy type inference and a memory management agnostic runtime model, meaning, it has no dependencies on the runtime (no GC is needed).

## Obligatory `Hello world!`

```Haskell

module Main where

main = println "Hello world!"


```

## Syntax

Syntax of `Siko` is heavily inspired by Haskell, most of Haskell's syntactic elements are borrowed and sometimes repurposed.

### Modules

Modules are the container units of Siko, all code is in modules. Every source file contains one or more modules.
Modules are introduced by the `module` keyword. Module names are started with an upper case letter and can contain the character `.` to
resemble logical hierarchies.

Example

```Haskell

module A where

id x = x

module Another.Module.With.A.Longer.Name where

foo = 1

```

Module declarations begin at the module keyword and end at the next module declaration or at the end of file, consequently modules cannot be nested.

```Haskell

module First where

f = 0

module Second where

f = 0

```

A module declaration contains one or more item declaration. Empty modules are syntactically invalid.

### Functions

Although `Siko` is an imperative programming language, it is heavily functional in its style, thus functions play a crucial role in `Siko` programs.

Function declaration examples

```Haskell

id a :: a -> a
id a = a

factorial :: Int -> Int
factorial n = if n < 2 then 1 else n * factorial (n - 1)

factorial2 0 = 1
factorial2 n = n * factorial2 (n - 1)

```

The body of a function is an expression that is evaluated when the function is called and the value of the expression is the return value of the function.

### Expressions

#### do

```Haskell

foo = do
    msg <- "Hello world"
    println msg

```

The `do` expression is a syntactic equivalent of a normal imperative block. It starts with the `do` keyword and contains one or more expressions which are evaluated in order, `do` blocks also provide scoping for name bindings. The return value of a `do` block is the value of the last expression in the block.

#### bind

```Haskell

foo = do
    (a, b) <- (1, 'a')

```

The bind expression binds the value of the expression on the right hand side to the pattern on the left hand side and they are separated by the `<-` symbol. Only irrefutable patterns are allowed in bind expressions. The return value of a bind expression is `()`.

### function application

The function application expression calls a function with the given arguments, evaluating to the return value of the function call.

```Haskell

id a = a

foo = id 1

```

Functions are curried, partial function application has no special syntax.

```Haskell

adder a b = a + b

foo = do
    f <- adder 1
    res <- f 2

```

### Literals

```Haskell

literals = do
    i1 <- -1
    i2 <- 123
    pi <- 3.14
    msg <- "Hello world"
    char <- 'a'
    tuple <- (i1, '4', ("another",), ())

```

### Patterns


