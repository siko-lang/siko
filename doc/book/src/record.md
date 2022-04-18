# Records

Records are data types with named fields and they are defined with the ```data``` keyword and ```{``` ```}``` characters.

```Haskell

data School = { name :: String,
                location :: String }

data Student = { name :: String,
                 age :: Int
                 school :: School }
```
