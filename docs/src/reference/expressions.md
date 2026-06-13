title: Expressions & Statements
layout: reference

# Expressions & Statements

A quick tour of every expression and statement form in Siko. Each section shows the syntax and a small self-contained example.

## Literals

The three primitive literal forms.

```
let i = 42;          // integer
let s = "hello";     // string
let c = 'A';         // char (a single byte)
```

String literals support interpolation with `${}`:

```
let name = "world";
let msg = "hello ${name}!";  // "hello world!"
```

## Variables

Lowercase identifiers refer to local variables and function parameters.

```
let x = 10;
let y = x + 1;
```

## Enum Variants and Globals

Capitalized names refer to enum variants or module-level globals when used as values.

```
let b = True;
let o = None;
```

## Blocks

A block is a sequence of statements wrapped in `{}`. Its value is the last expression if it isn't followed by a semicolon, or `()` if there is none.

```
let result = {
    let a = 1;
    let b = 2;
    a + b      // the block evaluates to 3
};
```

## Arithmetic & Logic

Standard two-operand operators: `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&`, `|`, `^`, `<<`, `>>`.

`&&` and `||` are short-circuit — the right-hand side is only evaluated if needed.

## Function Calls

```
let n = add(1, 2);
```

Named arguments make calls with many parameters easier to read:

```
let p = Point(x: 1, y: 2);
```

Method call syntax works for methods defined inside a struct, enum, or trait:

```
let upper = s.to_upper();
let n = v.len();
```

## If / Else

The else branch is optional. `if` is an expression:

```
if x > 0 {
    println("positive");
} else {
    println("non-positive");
}

let label = if x > 0 { "pos" } else { "neg" };
```

## If Let

Match a single pattern and branch:

```
if let Some(v) = maybe_value {
    println("got ${v}");
} else {
    println("nothing");
}
```

## Match

Full pattern matching. Arms are tried top-to-bottom; the first match wins. `match` is an expression:

```
match color {
    Color.Red   => println("red"),
    Color.Green => println("green"),
    Color.Blue  => println("blue"),
}
```

Guards let you add extra conditions to an arm:

```
match x {
    n if n < 0 => println("negative"),
    n if n > 0 => println("positive"),
    _          => println("zero"),
}
```

## With

`with` binds **implicits** and **effect handlers**.

An `implicit` declares a module-level variable slot that functions can read and write without it appearing in their signatures. The actual instance of the implicit is provided by `with` and is propagated through function calls automatically. Functions can also override the implicit for a scope or provide a completely new instance of the same type.

```
implicit counter : Int

fn increment() {
    counter = counter + 1;
}

fn main() {
    let n = 0;
    with counter = n {
        increment();   // n becomes 1
        increment();   // n becomes 2
    }
    println(n.to_string());   // prints 2
}
```

An `effect` declares a set of operations whose implementation is pluggable:

```
effect Logger {
    fn log(msg: String)
}

fn console_log(msg: String) { println("console: ${msg}") }

fn main() {
    with log = console_log { Logger.log("started"); }
}
```

## Lambdas

Anonymous functions — argument types are inferred:

```
let double = |x| x * 2;
let doubled = [1, 2, 3].iter().map(|x| x * 2).collect();
```

## Statements

### let

Bind a value to a name. Type annotation is optional. Pattern destructuring works too:

```
let x = 42;
let s: String = "hello";
let (a, b) = (1, 2);
let Point(x: px, y: py) = point;
```

### for Loop

Iterates over anything that implements `IntoIterator`:

```
for item in collection {
    println("${item}");
}
```

### while Loop

```
while i < 10 {
    i += 1;
}
```

### loop

Infinite loop — use `break` to exit:

```
loop {
    let line = read_line();
    if line == "quit" { break; }
    process(line);
}
```
