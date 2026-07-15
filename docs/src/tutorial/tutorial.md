title: Learn Siko in 21 Milliseconds
layout: reference

# Learn Siko in 21 Milliseconds

A dense, front-loaded guide for agents and fast readers. Every important concept is shown with a snippet. Skip nothing — the ordering is intentional.



## 1. The shape of a file

Every `.sk` file contains one or more named modules. The module boundary is the unit of visibility.

```
module My.App {

    fn main() {
        println("hello");
    }

}
```

Module names are dotted identifiers. Dots are just part of the name — they are not directory separators. Multiple modules can live in one file.



## 2. Types at a glance

| Category | Types |
|||
| Integers | `Int`, `I8`, `I16`, `I32`, `I64`, `U8`, `U16`, `U32`, `U64` |
| Floating point | `F32`, `F64` |
| Text | `String`, `U8` (char) |
| Boolean | `Bool` (`True` / `False`) |
| Containers | `Vec[T]`, `Map[K, V]`, `Set[T]`, `VecDeque[T]`, `Slice[T]` |
| Sum types | `Option[T]` (`Some(T)` / `None`), `Result[T, E]` (`Ok(T)` / `Err(E)`) |
| Tuples | `(A, B)`, `(A, B, C)`, … |
| Unit | `()` — the empty tuple |
| Functions | `fn(A, B) -> C` |

Most of these come from the standard library. `Option`, `Result`, `Vec`, `String`, `Bool`, and the numeric types are `@prelude` — available everywhere with no import. Floating-point literals default to `F64`; use a type annotation for `F32`. There are no implicit conversions between numeric types. `Map`, `Set`, `VecDeque`, and `Slice` are **not** prelude and require an explicit import:

```
import Std.Map
import Std.Set
```



## 3. Functions

```
fn add(a: Int, b: Int) -> Int {
    a + b
}
```

The last expression in a block is the return value — no `return` keyword needed for the happy path. The return type annotation is required unless the function returns `()`, in which case it can be omitted.

### Named arguments

```
fn make_point(x: Int, y: Int) -> Point {
    Point(x: x, y: y)
}

let p = make_point(x: 1, y: 2);
```

Named arguments are optional at the call site — you can always pass positionally.
Naming the arguments may make the call site clearer though, especially for boolean parameters or otherwise ambiguous.

### Generic functions

```
fn first[T](v: Vec[T]) -> Option[T] {
    v.iter().next()
}
```

Type parameters go in `[...]` after the function name. Bounds are written after a colon:

```
fn print_all[T: ToString[T]](items: Vec[T]) {
    for item in items {
        println(item.to_string());
    }
}
```

Multiple bounds are separated by commas: `[T: Foo[T], Bar[T]]`.
Multiple parameters with multiple bounds are written: `[A, B, C: Foo[A], Bar[B], Boo[B, C]]`. The `:` separates the type parameters from the list of bounds.



## 4. Structs

```
struct Point {
    pub x: Int,
    pub y: Int,

    pub fn distance(a: Point, b: Point) -> Int {
        let dx = a.x - b.x;
        let dy = a.y - b.y;
        dx * dx + dy * dy
    }
}
```

Methods are just functions inside the struct body. `pub` controls visibility — both fields and methods default to private. Construction uses the struct name as a function:

```
let p = Point(1, 2);          // positional
let p = Point(x: 1, y: 2);   // named
```

Generic structs:

```
struct Pair[A, B] {
    pub first:  A,
    pub second: B,
}
```



## 5. Enums

```
enum Shape {
    Circle(Int),        // radius
    Rectangle(Int, Int) // width, height
    Triangle(a: Int, b: Int, c: Int) // can use named arguments for variants with named items
}
```

Variants can carry data (any number of fields) or be plain:

```
enum Direction { North, South, East, West }
```

Constructing a variant:

```
let s = Shape.Circle(5);
let t = Shape.Triangle(a: 4, b: 5, 4);
let d = Direction.North;
```

The `Shape.` prefix is optional for unambiguous names.

Generic enums work the same way:

```
enum Tree[T] {
    Leaf(T),
    Node(Tree[T], Tree[T]),
}
```



## 6. Pattern matching

`match` is an expression. Arms are tried top-to-bottom; the first match wins.

```
match shape {
    Shape.Circle(r)      => println("circle r=${r}"),
    Shape.Rectangle(w, h) => println("rect ${w}x${h}"),
}
```

### Patterns you can use

| Pattern | Example |
|||
| Wildcard | `_` |
| Binding | `x` |
| Literal | `42`, `"hello"`, `'A'` |
| Enum variant | `Some(v)`, `Ok(x)`, `None` |
| Struct/variant named fields | `Point(x: px, y: py)` |
| Tuple | `(a, b)` |
| Guard | `n if n > 0` |

### `if let`

Match a single pattern and branch:

```
if let Some(v) = maybe_value {
    println("got ${v}");
}
```

Chains work:

```
if let Some(v) = a {
    ...
} else if let Ok(w) = b {
    ...
} else {
    ...
}
```



## 7. Let, assignment, and scoping

```
let x = 42;
let s: String = "hello";          // optional type annotation
let (a, b) = (1, 2);              // tuple destructuring
let Point(x: px, y: py) = point;  // struct destructuring
```

Variables are mutable by default — `x = x + 1` is legal. Shadowing is also allowed: a new `let x` in the same scope replaces the old binding.

Block-local scope:

```
let result = {
    let tmp = compute();
    tmp * 2       // the block evaluates to this
};
```



## 8. Control flow

### if / else

`if` is an expression:

```
let label = if score > 0 { "positive" } else { "non-positive" };
```

### Loops

```
for item in collection { ... }          // IntoIterator
for i in 0..10 { ... }                 // range (exclusive)

while condition { ... }

loop {                                  // infinite — exit with break
    if done { break; }
}
```

### break, continue, return

All three are expressions of type `()`. They can appear anywhere an expression is valid:

```
loop {
    match next_event() {
        Event.Quit  => break,
        Event.Skip  => continue,
        Event.Data(d) => process(d),
    }
}
```

`return` exits the current function early:

```
fn find(items: Vec[Int], target: Int) -> Option[Int] {
    for item in items {
        if item == target { return Some(item); }
    }
    None
}
```

### Try (`?`)

Postfix `?` on a `Result[T, E]` either unwraps `Ok(v)` to `v`, or immediately returns `Err(e.into())` from the enclosing function. The enclosing function must return a compatible `Result`.

```
fn read_config(path: String) -> Result[Config, Error] {
    let text = read_file(path)?;
    let cfg  = parse(text)?;
    Ok(cfg)
}
```

Error types are converted automatically via the `Into` trait, so you can mix error types as long as `Into` instances exist.



## 9. Traits and instances

A trait declares an interface. There is no implicit `self` — you always name the receiver explicitly.

```
trait Describe[T] {
    fn describe(t: T) -> String
}
```

An instance provides the implementation:

```
instance Describe[Point] {
    fn describe(p: Point) -> String {
        "(${p.x}, ${p.y})"
    }
}
```

### Calling trait methods

Once the trait is in scope, call its methods like any other function — the compiler picks the right instance:

```
println(describe(p));   // or p.describe() with method syntax
```

### Canonical vs named instances

- **Canonical (unnamed)** — globally visible, no import needed. One per type per trait across the whole program.
- **Named** — only in scope where imported. Used for specialization: a named instance for a specific type takes priority over a canonical one for the same type.

```
// canonical — always in scope
instance ToString[Int] {
    fn to_string(n: Int) -> String { ... }
}

// named — must be imported; takes priority for Vec[MyFoo]
instance MyFooVec ToString[Vec[MyFoo]] {
    fn to_string(v: Vec[MyFoo]) -> String { ... }
}
```

### Generic instances

```
instance[T: ToString[T]] Describe[Vec[T]] {
    fn describe(v: Vec[T]) -> String {
        v.iter().map(|x| x.to_string()).collect()
    }
}
```

### Associated types

```
trait Iterator[T] {
    type Item
    fn next(i: T) -> Option[Item]
}

instance Iterator[RangeIterator] {
    type Item = Int
    fn next(r: RangeIterator) -> Option[Int] { ... }
}
```

### `@derive`

Common trait instances can be auto-generated:

```
@derive(Clone, PartialEq, Eq, PartialOrd, Ord)
struct Point { x: Int, y: Int }
```



## 10. Modules and imports

```
import Std.Map
import Siko.Interner as I    // alias — only I.intern, I.resolve, etc. are in scope
```

A plain import makes every `pub` name from the module available in three forms:
- `Std.Map.insert` — fully qualified
- `Map.insert` — type-prefixed
- `insert` — bare short name

All three work as long as the name is unambiguous. If two imported modules export the same short name, use the qualified or aliased form.

Importing a module also pulls in all of its instances.

**Circular imports are fine.** Modules may import each other freely — there is no restriction on import cycles.

### Visibility

```
pub struct Foo { pub x: Int }    // pub on struct AND field
pub fn helper() -> Int { ... }
fn private() -> Int { ... }     // not importable outside
```

### Preludes

Modules annotated with `@prelude` are automatically imported everywhere. The standard library marks `Std.Option`, `Std.Result`, `Std.Vec`, `Std.String`, etc. as preludes — that is why you never import them.

```
@prelude
module My.Core { ... }
```



## 11. Closures

Anonymous closures — argument types are inferred:

```
let double = |x| x * 2;
let evens: Vec[_] = (0..10).into_iter().filter(|x| x % 2 == 0).collect();
```

Closures can capture variables from the enclosing scope. A closure has type `fn(A) -> B`.



## 12. The standard iterator chain

Iterators are lazy. Common adapters:

```
let result: Vec[String] =
    items
        .iter()
        .filter(|x| x.is_valid())
        .map(|x| x.name)
        .collect();
```

Key methods on `Iterator[T]`: `map`, `filter`, `filter_map`, `flat_map`, `enumerate`, `zip`, `chain`, `find`, `find_map`, `any`, `all`, `collect`.

`for item in x` is syntactic sugar for calling `x.into_iter()` and then `next()` in a loop.



## 13. Option and Result idioms

```
// Option
let v: Option[Int] = some_map.get(key);
let n = v.unwrap_or(0);
let s = v.map(|x| x.to_string()).unwrap_or("none");

// Result
let r: Result[Config, Error] = load();
let cfg = r.unwrap();           // panics on Err
let cfg = r.expect("load failed");
match r {
    Ok(c)  => use(c),
    Err(e) => handle(e),
}
```



## 14. Implicits

An implicit is a module-level variable slot that any function in the call tree can read or write without it appearing in any signature.

```
implicit counter : Int

fn increment() {
    counter = counter + 1;
}

fn main() {
    let n = 0;
    with counter = n {
        increment();
        increment();
    }
    println(n.to_string());   // 2
}
```

`with` binds the implicit to a local variable for its scope and all the functions that are called in that scope. Mutations are visible in the bound variable. Multiple implicits can be bound in one `with`:

```
with x = a, y = b {
    swap();
}
```

Inner `with` blocks shadow the outer binding for their scope only.



## 15. Effects

An effect declares a set of operations whose handler is pluggable at the call site.

```
effect Logger {
    fn log(msg: String)
}

fn console_log(msg: String) { println("console: ${msg}") }

fn do_work() {
    Logger.log("started");
}

fn main() {
    with log = console_log {
        do_work();
    }
}
```

Handlers must be named functions — lambdas cannot be used as effect handlers.

Like implicits, the handler is visible to the entire call chain inside the `with` block. You can have multiple effects and override just one in an inner scope:

```
fn foo() {
    with log = file_log {    // override Logger for this scope
        do_work();
    }
}

fn main() {
    with log = console_log, alert = system_alert {
        foo();   // foo overrides log; alert still uses system_alert
    }
}
```



## 16. Safety

By default every function is safe. Mark it `@unsafe` to use raw pointers, `extern` calls, `sizeof`, `transmute`, address-of (`&`), pointer dereference (`*`), or raw function pointer calls. Safe code can never call an unsafe function without also being marked `@unsafe` or `@safe`.

```
@unsafe
fn raw_copy(dst: Ptr[U8], src: Ptr[U8], n: Int) {
    // pointer arithmetic here
}
```

Trait instance methods and effect handlers must be either fully safe or explicitly marked `@safe` — you cannot smuggle unsafe behavior through a generic safe abstraction.



## 17. Annotations

Annotations are `@name` or `@name(args)` placed before a declaration.

| Annotation | Effect |
|||
| `@prelude` | Module is auto-imported everywhere |
| `@derive(...)` | Auto-generate trait instances |
| `@unsafe` | Mark function as unsafe |
| `@safe` | Mark function as safe |
| `@extern("C")` | Declare a C-compatible extern function |
| `@extern("C", "foo.h")` | Declare a C-compatible extern function and signals that foo.h needs to be imported |



## 18. Key things that differ from Rust/Haskell

- **No implicit self.** Trait methods always take the receiver as an explicit first parameter with any name you choose.
- **Traits have one type parameter** — the type the trait is for — not a self type.
- **No lifetimes.** Siko uses a garbage collector. Raw pointers exist but are unsafe and FFI-only.
- **Enum variants share the type namespace but not the value namespace.** A struct `Q` and a variant `Q` in the same module do not collide.
- **`with` is not a monad.** Implicits and effects are compiler-threaded context, not CPS transforms. There is no `bind`/`>>=`.
- **String interpolation is `${expr}`.** Not a macro — any expression works.
- **Tuple indexing is `.0`, `.1`, etc.** Like Rust.
- **`loop {}` is the only infinite loop.** `while` and `for` desugar through it.
- **All three of `break`, `continue`, `return` are expressions** of type `()`.



## 19. Complete worked example

```
module Main {

import Std.Map

@derive(Clone)
struct Student {
    pub name:   String,
    pub score:  Int,
}

fn grade(score: Int) -> String {
    match score {
        s if s >= 90 => "A",
        s if s >= 80 => "B",
        s if s >= 70 => "C",
        _            => "F",
    }
}

fn top_students(students: Vec[Student], threshold: Int) -> Vec[String] {
    students
        .iter()
        .filter(|s| s.score >= threshold)
        .map(|s| "${s.name}: ${grade(s.score)}")
        .collect()
}

fn main() {
    let students = [
        Student("Alice",   95),
        Student("Bob",     72),
        Student("Charlie", 88),
        Student("Diana",   61),
    ];

    let top = top_students(students, 80);
    for line in top {
        println(line);
    }
    // Alice: A
    // Charlie: B
}

}
```



## 20. Common standard library surface

### Vec[T]

`Vec.new()`, `.push(v)`, `.pop()`, `.len()`, `.get(i)`, `.iter()`, `.into_iter()`, `.contains(v)`, `.last()`, `.first()`, `.sort()`, `.reverse()`, index with `v[i]`, slice with `v[a..b]`.

### Map[K, V]  *(requires `import Std.Map`; `K: Ord[K]`)*

`Map.new()`, `.insert(k, v) -> Option[V]`, `.get(k) -> Option[V]`, `.remove(k)`, `.contains_key(k)`, `.len()`, `.iter()`.

### Set[T]  *(requires `import Std.Set`; `T: Ord[T]`)*

`Set.new()`, `.insert(v)`, `.contains(v)`, `.remove(v)`, `.len()`, `.iter()`.

### String

`.len()`, `.chars()`, `.split(sep)`, `.trim()`, `.starts_with(p)`, `.ends_with(p)`, `.contains(p)`, `.to_upper()`, `.to_lower()`, string interpolation with `"${expr}"`.

### Option[T]

`.is_some()`, `.is_none()`, `.unwrap()`, `.expect(msg)`, `.unwrap_or(default)`, `.unwrap_or_else(f)`, `.map(f)`, `.map_or(default, f)`, `.and_then(f)`, `.or(other)`, `.iter()`.

### Result[T, E]

`.is_ok()`, `.is_err()`, `.unwrap()`, `.expect(msg)`, `Result.collect(vec)` (transpose `Vec[Result]` → `Result[Vec]`).



## 21. Cheat sheet

```
// module
module Foo.Bar { ... }
import Foo.Bar
import Foo.Bar as FB

// visibility
pub struct S { pub field: Int }
pub fn f() -> Int { ... }

// functions & generics
fn id[T](x: T) -> T { x }
fn bounded[T: ToString[T]](x: T) -> String { x.to_string() }

// data
struct Point { x: Int, y: Int }
enum Tree[T] { Leaf(T), Node(Tree[T], Tree[T]) }

// let & patterns
let x = 42;
let (a, b) = (1, 2);
let Point(x: px) = p;

// control
if c { ... } else { ... }
for item in col { ... }
while cond { ... }
loop { if done { break; } }

// match
match val {
    Pattern(x) if x > 0 => expr,
    _ => default,
}
if let Some(v) = opt { ... }

// traits
trait Foo[T] { fn foo(t: T) -> Int }
instance Foo[MyType] { fn foo(t: MyType) -> Int { 0 } }
@derive(Clone, PartialEq)
struct S { x: Int }

// implicits
implicit ctx : Config
with ctx = my_config { use_ctx() }

// effects
effect Log { fn log(msg: String) }
fn console_log(msg: String) { println(msg) }
with log = console_log { do_work() }

// error handling
fn f() -> Result[Int, Error] {
    let v = might_fail()?;
    Ok(v + 1)
}

// unsafe
@unsafe fn dangerous() { ... }
```
