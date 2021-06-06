# Algebraic data types

Algebraic data types are defined using the ```data``` keyword.

```Haskell

data Month = January
           | February
           | March
           | TheOthers

data Expr = Lit Int
          | Sum Expr Expr
          | Mul Expr Expr

```

Types can be generic:

```Haskell

data Option a = Some a | None

```

GADTs are not supported (yet).