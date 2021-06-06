# Modules

A Siko program is simply a collection of modules and modules are a collection of other definitions.
Every data type/function/typeclass/effect belongs to a single module. Modules are named and their names can contain the '.' character but they are not hierarchial. All modules simply exist at the same level.
The definiton of a module start wit the ```module``` keyword and ends at the end of the file or at the start of the next module.

```Haskell

module My.First.Module where

foo = 0

module This.Is.A.New.Module.Not.Included.In.My.First.Module where

bar = 0

```