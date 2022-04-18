# Introduction

The idea of Siko was born out of frustration of current programming languages (as it is usually the case with any new programming language). I noticed that you can write most algorithms in most of the programming languages but you cannot simply reuse that algorithm in another programming language. You have to manually convert the algorithm from one language to another, considering all the little differences between the two languages. There is something fundamentally broken in the way we express ourselves in a programming language. The computer cannot really understand your code and it cannot just convert the fundamentals of the algorithm between languages. They are so different at the basic level that you cannot include a Haskell module in a C program or cannot simply use a Java lib in a python module without extraordinary hacks.
I believe the root cause of this issue is that programming languages have well defined runtime semantics and those are usually fully incompatible with some other programming language. However, the code we write does not necessarily have this property. Siko is an experiment in finding a way that can truly capture the essence of an algorithm where both the reader and the compiler can understand the code equally. Siko tries to be a runtime independent programming language, every concept/behaviour in the language is designed to be fully unabmiguous regardless of the actual runtime of target system.
Siko compiles everything down into primitives so general that any runtime can execute them equally. Siko does not have its own runtime and it could be transpiled into any other language or even compiled into runtime-less native code.
The current compiler only supports transpiling into Rust.

The innovative bits and pieces of Siko are its runtime semantics, so as syntax something well defined and well established was chosen. The syntax of Siko follows the syntax of Haskell like programming languages with a few differences.

A taste of Siko:

```Haskell
module Main where

data City = { name :: String,
              population :: Int
            }

bigCities = List.filterMap cities (\c -> if c.population > 1000000 then Some c.name else None)

main = do
    cities <- [City "New York" 18823000,
               City "Los Angeles" 12459000,
               City "Los Alamos" 20131]
    println "Big cities {}" % (bigCities cities)
```