title: Modules & Imports
layout: reference

# Modules & Imports

How Siko organizes code into modules and how they talk to each other.

---

## Modules

A `.sk` file contains one or more modules, written one after another:

```siko
module My.Cool.Module {
    // ...
}

module My.Other.Module {
    // ...
}
```

Module names are dotted identifiers — the dots are just part of the name, not directory separators. How you group modules into files is entirely up to you.

### What can go inside a module

- `fn` — free functions
- `struct` / `enum` — data types (with their methods)
- `trait` / `instance` — type class definitions and implementations
- `effect` — effect interfaces
- `implicit` — module-level implicit variables
- `let` — module-level global constants
- `import` — imports from other modules

Everything except `import` can be prefixed with `pub` to make it visible outside the module. Without `pub` a name is private and only usable inside its own module.

---

## Imports

To use something from another module, import it:

```siko
import Std.Map
```

After this, every `pub` name exported by `Std.Map` becomes available in three forms. For a method like `insert` on the `Map` struct:

- `Std.Map.insert` — fully qualified
- `Map.insert` — type-prefixed
- `insert` — bare short name

All three work as long as the name isn't ambiguous with something else in scope. Short names are convenient; qualified names are always unambiguous.

Enum variants are available with the following name forms:

- `My.Module.Enum.Variant` — fully qualified
- `My.Module.Variant` — type-prefixed
- `Variant` — short name

### Aliases

Give a module a local prefix with `as`:

```siko
import Siko.Interner as I
```

When aliased, the module's names are **only** available through the alias — `I.intern`, `I.resolve`, etc. The bare short names and the original qualified names are not in scope.

This is the main point of aliases: if two modules export the same short name, import at least one with an alias to eliminate the ambiguity. Each name then lives behind its own prefix and there's no clash.

---

## Name resolution order

When a name is used inside a module, the resolver looks it up in this order:

1. **Local items** — defined in the current module itself
2. **Imported items** — brought in via `import`
3. **Implicit items** — automatically available (see Preludes below)

Local always wins. If the same short name appears in more than one imported module the compiler reports an ambiguous name error — resolve the error by using the qualified (or aliased) form instead.

---

## Preludes

Modules annotated with `@prelude` are automatically imported into every other module in the program — you never have to write those imports yourself. The standard library uses this to make core types like `String`, `Vec`, `Option`, `Result`, `Bool`, and the numeric types available everywhere.

If you're writing a library or application and want some module to be universally available, annotate it:

```siko
@prelude
module My.Core {
    // ...
}
```

Prelude imports are implicit and never produce unused-import warnings.

---

## Visibility

The `pub` keyword controls what a module exports:

```siko
module Geometry {
    pub struct Point {
        pub x: Int,
        pub y: Int,
    }

    pub fn distance(a: Point, b: Point) -> Int { ... }

    fn helper() -> Int { ... }   // private — not importable
}
```

Fields of structs have their own `pub` — mark only the fields you want to expose.

Methods on structs and enums follow the same rule: `pub fn` is exported, `fn` is module-private.

---

## Instances and imports

`instance` declarations are special: importing any name from a module pulls in all of that module's instances too.

