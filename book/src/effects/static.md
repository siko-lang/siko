# Static Effects

Static effects are effects with a statically dispatched handler,, i.e. the compiler can compile time decide which function will be called as the effect handler.

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

In the above example, the effectCall is defined to be handled by myEffectCall which is a statically known function's name without any argument, so the effect handler does not have any associated environment. Thus, this call can be statically dispatched, i.e. the effectCall call in someFunc can be simply replaced by a myEffectCall in the generated code. This type of effect call has zero runtime overhead, it is essentially a type safe search and replace.