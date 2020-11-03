# The Siko programming language

## ! This is a work in progress, unfinished documentation. !

## Introduction

`Siko` is a statically-typed, strict, impure and mostly imperative programming language that visually and sometimes semantically resembles a functional language. All `Siko` programs are function applications transforming immutable values. In `Siko` there are no mutable variables, everything is a value, there are no references. It features heavy type inference and a memory management agnostic runtime model, meaning, it has no dependencies on the runtime (no GC or RC is needed).

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

Module definitions begin at the module keyword and end at the next module definition or at the end of file, consequently modules cannot be nested.

```Haskell

module First where

f = 0

module Second where

f = 0

```

A module definition contains one or more item definition. Empty modules are syntactically invalid.

### Functions

Although `Siko` is an imperative programming language, it is heavily functional in its style, thus functions play a crucial role in `Siko` programs.

Function definition examples

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

### literals

```Haskell

literals = do
    i1 <- -1
    i2 <- 123
    pi <- 3.14
    msg <- "Hello world"
    char <- 'a'
    tuple <- (i1, '4', ("another",), ())

```

### if

The `if` expression can be used to create conditional expressions, the `else` block is always required. The `if` expression evaluates either the true branch or the false branch and returns the value of selected branch.

```Haskell

isLarge n = if n > 10 then True else False

```

### case

The `case` expression is used for pattern matching on values. The `case` expression evaluates a single branch and returns the value of the selected branch.

```Haskell

isLarge n = case n of
    1 -> False
    n if n <= 10 -> False
    _ -> True

```

### record field access

To access a field of a record, use a `.` followed by the name of the field. The value of the record field access expression is the value of the field.

```Haskell

data Person = { name :: String, age :: Int }

getAge :: Person -> Int
getAge p = p.age

```

### tuple field access

To access a field of a tuple, use a `.` followed by the index of the field. The value of the tuple field access expression is the value of the field.


```Haskell

second a b :: (a, b) -> b
second t = t.1

```

### lambda

The lambda expression is an unnamed function definition that evaluates to a function. The lambda expression starts with a `\` followed by the
lambda arguments then a `->` followed by an expression that is the body of the lambda function.

```Haskell

test = do
    f <- \x, y -> x + y
    f 2 3

```

### return

The `return` expression stops the execution of the current block and returns the value of the expression given as the argument.

```Haskell

test = return ()

```

### loop

The `loop` expression resembles an imperative loop. It has a pattern, an initializer expression and a body.
The loop is started by evaluating the initializer expression, its value is then matched with the given pattern and then the body is executed once. After the execution, the value of the body is match with the given pattern and the body is executed again, forever.

```Haskell

loopTest = loop index <- 1 do
            print "Cycle count {}" % index
            index + 1
```

### break

The `break` expression jumps out of the current `loop` and the return value of the loop will be the value of the expression given as the argument of break.

```Haskell

loopTest = loop index <- 1 do
        print "Cycle count {}" % index
        if index > 10
            then break ()
            else index + 1
```

### Patterns


