title: Effects & Implicits
layout: reference
priority: 90

# Effects & Implicits

Two related features for threading context through a call chain without explicit parameter passing.

---

## Implicits

An implicit is a module-level variable slot that any function in the call chain can read or write — without it appearing in any function signature. You declare it at module scope with `implicit`, and bind it to a local variable with `with`. This feels like a scoped global variable.

```siko
implicit counter : Int

fn increment() {
    counter = counter + 1;
}

fn main() {
    let n = 0;
    with counter = n {
        increment();
    }
    println(n.to_string());   // 1
}
```

The compiler passes the bound value as a hidden context parameter throughout the call tree — every function that uses the implicit receives it automatically, without it appearing in any written signature. Mutations made anywhere in the call tree are visible in the bound variable.

### The call chain sees it automatically

Every function called inside the `with` block — and every function those call — automatically has access to the implicit:

```siko
implicit config : Config

fn baz() { println(config.name) }
fn bar() { baz() }
fn foo() { bar() }

fn main() {
    let c = Config("prod");
    with config = c {
        foo();   // baz() deep inside the chain reads config
    }
}
```

### Multiple implicits at once

Bind several implicits in one `with`:

```siko
implicit x : Int
implicit y : Int

fn swap() {
    let tmp = x;
    x = y;
    y = tmp;
}

fn main() {
    let a = 1;
    let b = 2;
    with x = a, y = b {
        swap();
    }
    // a == 2, b == 1
}
```

Or nest `with` blocks:

```siko
with x = a {
    with y = b {
        sum();
    }
}
```

### Shadowing

An inner `with` block can rebind an implicit for its scope. The outer binding is unaffected:

```siko
let n = 0;
with counter = n {
    increment();          // n = 1
    let local = 0;
    with counter = local {
        increment();      // local = 1, n stays 1
    }
    increment();          // n = 2
}
```

### Implicits and closures

A closure created inside a `with` block captures the implicit binding. When the closure is later called, it still uses the correct context:

```siko
implicit config : Config

fn foo() { println(config.name) }

fn main() {
    let c = Config("myconfig");
    let f: fn() -> () = with config = c {
        || { foo() }
    };
    f();   // prints "myconfig"
}
```

---

## Effects

An effect declares a named interface of operations whose implementation is pluggable at the call site. You can use `with` to provide the concrete handler.

```siko
effect Logger {
    fn log(msg: String)
}

fn console_log(msg: String) { println("console: ${msg}") }
fn file_log(msg: String)    { println("file: ${msg}") }

fn do_work() {
    Logger.log("started");
}

fn main() {
    with log = console_log { do_work() }   // calls console_log
    with log = file_log    { do_work() }   // calls file_log
}
```

`Logger.log(...)` inside `do_work` is a hole — it dispatches to whatever handler is in scope at the call site. The same function can run under completely different implementations without any changes to its code.

### Effects thread through the call chain

Like implicits, effect handlers are visible to the entire call chain inside the `with` block:

```siko
fn baz() { Logger.log("deep") }
fn bar() { baz() }
fn foo() { bar() }

fn main() {
    with log = console_log {
        foo()   // baz() deep down still dispatches to console_log
    }
}
```

### Multiple effects

Bind several effects at once or in separate `with` blocks. Inner blocks can override a specific effect while leaving others unchanged:

```siko
effect Logger { fn log(msg: String) }
effect Alarm  { fn alert(msg: String) }

fn bar() {
    Logger.log("hello");
    Alarm.alert("world");
}

fn foo() {
    with alert = another_alert {   // override just Alarm inside foo
        bar()
    }
}

fn main() {
    with log = console_log, alert = system_alert {
        foo()   // foo overrides alert; Logger still uses console_log
    }
}
```

### A handler can call other effects

Handlers are plain functions, so they can call other effects. Those are resolved against whatever handlers are in scope at the point the handler runs:

```siko
fn my_log(msg: String) {
    Alarm.alert(msg)   // delegates to the Alarm handler in scope
}

fn main() {
    with log = my_log, alert = system_alert {
        Logger.log("hello")   // → my_log → system_alert
    }
}
```
