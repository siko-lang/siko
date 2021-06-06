# Effects

Effects meant to be a way to inject functionality into code with very little boilerplate.
The idea behind effects is that a library should use effects to describe all its interactions with the external world and let the caller decide how to implement those interactions. The effect interface provides a type safe communication between the library author and the library user.

Effects can appear in the function type but usually they are not present and inferred by the compiler. The idea is that introducing, changing or removing an effect should not be an activity that affects all function signature in the call chain, only the effect's user and the effect handler's function are affected.

A simple example

```Haskell

effect SimpleEffect where
    effectCall :: String

someFunc :: String using (SimpleEffect)
someFunc = effectCall

myEffectCall = "foo"

main = do
    with { effectCall = myEffectCall } do
        println someFunc
```
In the example above the someFunc function uses the effect called SimpleEffect. In the example the effect appears in its signature but this effect declaration is fully optional, the list of used effects can be inferred by the compiler.

The ```with``` block introduces an effect handler, it specifies that the effectCall effect call must be handled by the call myEffectCall for everything inside the with block (including the someFunc call).

Effect handlers can be static or dynamic and effects can introduce types as well.