# Effect syntax

## Effect definition

Effects are defined using the ```effect``` keyword, otherwise they are very similar to type class definitions. This is not a coincidence. Effects are very similar to type classes, they define an interface, however effect 'instances' are not bound to types, but scopes.

An effect definition:

```Haskell

effect (Show a) => Factory a where
    create a :: a

```

The effect above defines a factory that has a single call called ```create``` which can create an instance of some type a that implements the interface of the Show typeclass. The effects user does not know what the returned type of a will be, only that it can be converted into a String (using Show).

## Declaring effects in function signatures

```Haskell

someFunc a :: Int -> String using (Factory a)
```

Effects in function signatures are declared using the ```using``` keyword. The type parameters of effects are part of the type parameter list of the function signature. Used effect declarations are optional part of a function signature.