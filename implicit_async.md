# Implicit Async in Siko

> Design investigation. Goal: transparent sync/async I/O with **no `async`/`await`
> keywords**, built on the coroutine + effect primitives the language already has.
> The scheduler/executor lives entirely in userspace; the language only provides
> primitives.

This document assumes familiarity with two existing mechanisms and builds on them.
It is a research/design doc, not an implementation plan only — but it ends with a
staged plan.

---

## 0. The two primitives we already have

### 0.1 Coroutines (built, working)

`co f(args)` reifies a call to a coroutine function as a heap value; `resume`
drives it one step:

```
co_wrap_N        ::= struct { state: co_state_N }                    // the handle (reference type)
co_state_N       ::= enum { Variant0(co_frame_N_0), Variant1(...), ... }  // one variant per co-fn of signature (Y,R)
co_frame_N_v     ::= struct { state: co_fstate_N_v, _ctx, <args...>, <locals...> }
co_fstate_N_v    ::= enum { Start, AfterYield_0, ..., AfterYield_k, Completed }   // value type
resume(c: co_wrap_N) -> CoResult[Y,R]    where CoResult = { Yielded(Y), Returned(R), Completed }
```

The handler is a **state machine**: it `match`es `frame.state`, `goto`s the resume
point, runs a segment, then either `frame.state = AfterYield_i; return Yielded(v)`
or `frame.state = Completed; return Returned(v)`. **All locals live in the frame**,
so they survive across suspension points. The frame is a GC'd reference type, mutated
through `resume`.

Two facts that matter below:

- **The lowering is a tree walk** (`HandlerLower`, an `auto(Map)` pass): every
  `yield` *anywhere in the body* — including nested in `if`/`match`/loops — is
  rewritten in place into `set-state / return Yielded / resume-label`. The dispatch
  `goto`s into the middle of nested structures. So **any expression position can
  become a suspension point.** This is the hook the async transform needs.
- It runs **before `lower_tuples`**, so it sees pre-lowering, monomorphized types.

Today's coroutines are **asymmetric generators**: `yield` carries a value *up*;
`resume` carries *nothing back down*. Async will need to relax this (see §5).

### 0.2 Effects (working, mono-resolved)

An `effect` is a named set of operations. `with op = handler { ... }` binds a
handler for the dynamic extent of the block. The crucial implementation detail:

- **Effect operation calls are resolved during monomorphization.** `Monomorphizer/Expr.sk`
  carries a `current_effect_ctx` (operation → chosen handler). At an `EffectMethod`
  call it looks up the handler in that context and emits a direct call.
- Therefore **the same source function is specialized per effect context.** `do_work`
  called under `with log = console_log` and under `with log = file_log` becomes *two
  different monomorphic functions*, each with the handler baked in.
- **Handlers are plain functions that execute at the effect call site** (in the
  callee's activation), threaded like implicits. Effects are *not* stack-unwinding
  delimited continuations.

This last point is the whole game: **a handler is ordinary code running at the
operation site. If that code suspends, the suspension happens exactly where the I/O
conceptually happens — at the bottom of the call chain.**

---

## 1. The vision

```
effect Net { fn read(s: Socket, buf: Slice) -> Int  ;  fn accept(l: Listener) -> Socket ; ... }

fn handle(req)            { ... Net.read(sock, buf) ... }    // no async anything
fn serve(listener)        { let s = Net.accept(listener); handle(read_request(s)) }
fn main()                 { serve(open(8080)) }
```

Run it two ways, **same source**:

- **Sync:** `with Net = blocking_net { main() }` — `Net.read` is a blocking
  syscall, returns directly. Nothing is a coroutine. Zero overhead.
- **Async:** `Scheduler.run(|| with Net = async_net { main() })` — `Net.read`
  registers interest with a reactor and *suspends*; the scheduler multiplexes many
  such `main()`-like tasks over one thread.

No keyword distinguishes the two. The difference is **which handler is in scope**,
and that is resolved by monomorphization.

The three ingredients:

1. **Auto-coroutine coloring** — a function that (transitively, *in this mono
   context*) might suspend is compiled as a coroutine; a call to such a function
   becomes a suspension point in the caller. (§3, §4)
2. **Effect-injected yields** — the async I/O handler is the *leaf* that actually
   suspends; it registers with the reactor and yields. (§2, §6)
3. **A userspace scheduler** — `co`'s the top, drives the leaf directly on wakeup,
   walks completions up the chain. (§6, §7)

---

## 2. Why mono-resolved effects make this work (the key insight)

"Is this function async?" is normally undecidable without an `async` annotation,
because it depends on what the callees *do*, which depends on runtime handlers. In
Siko it **is** decidable — *after monomorphization* — because effect handlers are
chosen per mono context.

Define a monomorphic function as **suspending** if:

- it performs a `yield` (explicit generator), **or**
- it calls an effect operation whose *chosen handler in this mono context* is
  suspending, **or**
- it calls another monomorphic function that is suspending.

This is a least-fixed-point over the **post-mono call graph** (handlers included as
ordinary edges). Compute it once after mono; the "color" of every monomorphic
function is known. `handle` is suspending under `async_net` and *not* suspending
under `blocking_net` — and those are genuinely different mono instances, so there is
no conflict and no coloring lost to dynamic dispatch (modulo §8.1 indirection).

> This is the property that buys us keyword-free async: **the async/sync split is
> just the suspending/non-suspending partition of the monomorphized program, and
> monomorphization already split the program by handler.**

The coroutine lowering pass already runs right after mono/implicit-propagation, with
exactly the per-context monomorphic bodies in hand. The coloring analysis slots in
just before it.

---

## 3. Coloring: who becomes a coroutine

After mono + the suspending-analysis:

- A **suspending** monomorphic function is lowered as a coroutine (gets a frame,
  handler state machine), even if it contains no explicit `yield`. Its suspension
  points are: explicit `yield`s **and** every call to a suspending callee (§4).
- A **non-suspending** function is left exactly as today — a plain C function, no
  frame, no overhead.

Note the signature consequence: a suspending function `f : A -> R` is *invoked* as a
coroutine `co f(a) : co(Suspend) -> R` rather than called directly. The compiler
rewrites call sites (§4); the user never writes `co` for these (that stays the
explicit-generator surface syntax).

The "yield type" of an implicitly-async coroutine is **not** the user's choice; it's
the scheduler's suspension protocol type (§5.3), fixed by the runtime library.

---

## 4. The call-site transform ("yield at the call site")

This is the heart of "every function that calls a coroutine becomes a coroutine and
yields at the call site." In a suspending function, a call `r = g(args)` to a
suspending callee `g` is rewritten to a drive-once-then-maybe-suspend sequence:

```
let child = co g(args);              // reify callee as a coroutine frame, parent-linked to self
match resume(child) {
    Returned(r)  => r                // g completed synchronously (never actually suspended): continue inline
    Yielded(sig) => {
        // g suspended somewhere below. Record child as our active child,
        // save our state, and propagate the suspension upward.
        self.frame.awaiting = child;
        self.frame.state    = AfterCall_i;       // resume point is *right here*
        return Yielded(sig);                     // bubble up to our resumer
        // ---- resume point AfterCall_i ----
        // We are re-entered only when `child` has fully RETURNED.
        // Its return value is delivered to us (see §5):
        deliver_child_result()                   // == r
    }
    Completed => unreachable
}
```

Observations:

- This is **structurally identical to the `yield` expansion** I already implemented
  in `HandlerLower` (set state, return, drop a resume label) — just with the extra
  "reify child + resume once + on-return read child result" wrapping. The existing
  tree-walk handles it in any expression position, including nested in `if`/loops.
- The **fast path is fast**: a suspending function whose callee *doesn't actually
  suspend this time* (`Returned` on the first `resume`) pays one frame alloc + one
  `resume` and continues inline. (Optimizable further — §8.6.)
- The resume point `AfterCall_i` is a real state in the caller's `co_fstate`. So a
  co-call site is an await point, exactly like a `yield`.

The effect operation `Net.read(...)` is, after mono, a call to the chosen handler
function. Under `async_net` that handler is suspending, so the call to it is colored
and transformed exactly as above. **The user's `Net.read(...)` becomes an await
point with zero syntax.**

---

## 5. The missing primitive: passing values *down* on resume

Today `resume` returns a value (`Yielded`/`Returned`) but accepts none. Async needs
to deliver two kinds of values *into* a suspended frame:

1. the **I/O result** when the reactor wakes the leaf (`read` returns N bytes), and
2. the **child's return value** when a parent is resumed after its child completed
   (§4's `deliver_child_result`).

Two ways to provide this:

### 5.1 Option A — bidirectional `resume` (symmetric-ish coroutines)

Change the ABI to `resume(c, v: Resume) -> CoResult[Yield, Return]`. The value `v`
becomes the result of the suspension expression in the resumed frame. Signature
grows from `co(Y) -> R` to `co(Yield, Resume) -> R`. Clean and general; this is the
"real" coroutine. Cost: ABI change, an extra type parameter, every yield/await site
must type the down-value.

### 5.2 Option B — side-channel slot in the frame

Keep `resume` value-less. Before resuming a frame, the scheduler writes the incoming
value into a well-known frame slot (`frame.resume_slot`); the resumed code reads it.
This is just Option A implemented by hand, and it keeps the generator ABI intact.
Likely the pragmatic first step.

Either way, the **return value of a completed child** must be retrievable. Simplest:
when a coroutine hits `Completed`, its `Returned(r)` was the *previous* `resume`
result; the scheduler caches `r` and hands it to the parent (slot or down-value).
(Today `resume` yields `Returned(r)` once then `Completed`; we either keep `r` in the
frame after return, or the scheduler stashes it on the transition.)

### 5.3 The up-value: the suspension signal

What does an implicit-async `yield` carry *up*? Minimal design: a single
`Pending` marker (unit-like). The *actual* wait description (which fd, readable vs
writable) is **registered as a side effect** by the leaf handler before it yields —
"the bottom coroutine registers itself." So the signal up the chain carries no
payload; it only means "a leaf below has parked itself; return control."

This keeps the propagated type trivial and uniform across the whole chain (every
frame yields the same `Pending`), which matters because the chain is heterogeneous
(different `R` per frame, but the same suspension protocol).

If we later want richer cooperative scheduling (priorities, yield-to-scheduler
without I/O, cancellation tokens) the up-value can grow into a small enum; start with
`Pending`.

---

## 6. Direct leaf wakeup — the property you actually care about

The requirement: **on an I/O event, wake only the bottom coroutine; leave the rest
of the chain idle. Wake the next frame up only when its callee returns.**

### 6.1 The shape of a suspended task

A suspended task is a **linked stack of frames** (a "cactus stack" if tasks fork):

```
top(main)  ──child──▶ serve ──child──▶ handle ──child──▶ async_net.read   ← LEAF (registered with reactor)
   ▲ parent             ▲ parent          ▲ parent
```

Each frame stores a **parent link** (set when the parent reified it in §4) and its
own saved state. Exactly one frame per task is the **active leaf**.

The scheduler holds:

- a **ready queue** of leaves to run,
- the **reactor** map `fd → leaf` (populated by leaf handlers at registration),
- (implicitly, via parent links) the chain above each leaf.

### 6.2 First descent (establishes the suspension — O(depth), once)

`Scheduler.run`: `let t = co main(); resume(t)`. Control flows *down* through the
auto-coroutine call-site transforms until `async_net.read` runs at the bottom. The
leaf handler:

```
fn async_read(s, buf) -> Int {
    reactor.register(s.fd, READABLE, current_coroutine());   // register SELF directly
    yield Pending;                                            // suspend
    // resumed here when readable:
    nonblocking_read(s.fd, buf)
}
```

`yield Pending` returns control to the leaf's resumer (its parent's §4 drive code),
which sees `Yielded(Pending)`, records the child and **re-yields `Pending`**. This
bubbles up the chain once and returns control to `Scheduler.run`, which parks the
task and goes back to its event loop. The whole chain is now suspended in memory; the
reactor holds `fd → leaf`.

`current_coroutine()` is a new runtime primitive: the innermost frame currently being
resumed. `resume` sets a thread-local "current frame" around the dispatch so any code
(including handlers) can obtain its own handle. (Implementation: push/pop on the
resume boundary.)

### 6.3 Wakeup (the fast path — does NOT walk the chain)

Reactor reports `fd` readable → look up `leaf` → push to ready queue → scheduler
`resume(leaf, io_ready)`. The leaf resumes **directly** at its post-`yield` point,
does the nonblocking read, and:

- **yields again** (needs more I/O): re-register, stays the leaf. The chain above is
  never touched.
- **returns `r`**: the leaf is done. Now — and only now — the scheduler resumes the
  **parent** (`leaf.parent`), delivering `r` (§5). The parent re-enters at its
  `AfterCall_i`, continues, and either:
  - calls another coroutine → pushes a new child → new leaf descends, or
  - hits its own I/O → becomes the new registered leaf, or
  - returns → scheduler pops to *its* parent, delivering *its* value, … and so on up.

So a single I/O event resumes exactly one frame. Returns walk up **one frame per
actual completion**, not per I/O event. A task blocked deep in a loop doing thousands
of reads touches only its leaf frame thousands of times; the frames above it are
inert until the leaf's function finally returns. **This is the idle-chain property.**

Contrast with the naive "re-drive from the top" model (scheduler always resumes
`main`, which re-drives down to the leaf): that is O(depth) per I/O event and keeps
re-walking dormant frames. We explicitly avoid it via the `fd → leaf` registration +
parent links.

### 6.4 Why parent links and not a re-walk

The parent link is what lets the scheduler resume the *parent* when a child returns
without re-running the grandparents. It's set for free during §4: when a frame reifies
its child (`co g(args)`), it records `child.parent = self` (and `self.awaiting =
child`). The scheduler never needs the full chain at once — only "who do I resume when
*this* frame returns," which is one hop.

---

## 7. The top and the scheduler

- The runtime entry does `let root = co user_main()` and runs the loop. `co
  user_main()` type-checks because `user_main` is colored suspending in the async
  handler context (§2).
- The **reactor** (epoll/kqueue/io_uring wrapper) is exposed to leaf handlers as an
  effect or implicit (`reactor.register(fd, interest, leaf)`), made available by the
  scheduler's `with`.
- The scheduler is **plain Siko**: a ready queue, the reactor, and the resume/return
  bookkeeping of §6. Spawning another task is `let t = co some_fn(args);
  scheduler.spawn(t)` — i.e. `co` of any suspending function yields a schedulable
  object, as you wanted. Multiple independent chains = multiple cactus stacks the
  scheduler multiplexes.

Everything above the `co`/`resume`/`current_coroutine`/`resume-with-value`
primitives is library code.

---

## 8. Hard problems & open questions

### 8.1 Indirection erases color (the real hazard)
`fn(A) -> R` pointers, closures, trait/dynamic dispatch, and effect *handlers passed
as runtime values* break static coloring: at an indirect call the compiler can't know
if the target is suspending. Options:
- **Color the function type.** A pointer to a suspending fn has type `co-fn`, and an
  indirect call through it is always an await point (drive code emitted
  unconditionally). Non-suspending pointers stay plain. This splits fn-pointer types
  by color (a form of the "what color is your function" tax, but inferred).
- **Uniform suspending ABI for all indirect calls** — every indirect call site emits
  drive code; the callee, if non-suspending, returns `Returned` on the first
  `resume`. Simpler, costs a frame per indirect call.
- **Forbid** taking a coroutine fn as a plain pointer (diagnostic). Least pleasant.
Closures capturing across suspension already work (frames are heap), but the *call*
through the closure has the same coloring question.

### 8.2 `resume`-with-value ABI (§5)
Decide A vs B. Recommendation: ship B (frame slot) first to avoid an ABI/type-param
change, migrate to A if symmetric coroutines prove generally useful.

### 8.3 Cancellation + resource cleanup (no destructors)
Siko is GC'd with no dtors. Dropping a task = dropping its frame tree (GC reclaims
memory). But a **registered fd must be deregistered** from the reactor, and any
half-done I/O resource released. Without dtors there is no automatic unwinding of a
suspended chain. Options: an explicit `cancel(task)` that the scheduler routes to the
leaf to run cleanup (requires a cleanup/`defer`-like effect, or a "cancel" resume
mode that drives finalizers); or model resources via an effect whose handler the
scheduler can finalize. **This is the biggest open design question.**

### 8.4 Errors / panics across frames
A panic in a leaf must propagate to the awaiting parent and up. With value-passing
resume, errors are just `Result` returns and flow naturally through §4's
`deliver_child_result`. Hard panics (abort) bypass the chain — probably fine
(process dies), but a recoverable-error effect is cleaner. Decide whether the chain
participates in `try`/`?`.

### 8.5 Structured concurrency: spawn / join / select
`co f()` gives one schedulable chain. Real async needs: spawn N, await any/all,
timeouts, channels. These are **library** constructs over the primitive: a `join`
parks the current leaf and registers it to be woken when child tasks complete
(children get their own reactor-less completion notifications). `select` registers
interest in several leaves/fds. Worth prototyping early to validate the primitive set
(esp. whether the up-signal must carry more than `Pending`).

### 8.6 Cost of the synchronous fast path
Every suspending→suspending call allocates a child frame even when it completes
without ever suspending. Mitigations: (a) only color a call site if the callee can
*actually* suspend in this context (the FLP already gives this — a callee that's
non-suspending in this mono context is called directly, no frame); (b) inline
trivial suspending callees; (c) arena/pool frames per task; (d) a "may suspend but
usually doesn't" fast-return path that avoids heap alloc until the first real
suspend. (a) is the big one and the analysis already provides it.

### 8.7 Generators + async together
Explicit `yield` (generator, user-chosen `Y`) and implicit async (`Pending`) share
the yield channel. A function that is both an async task and a value-generator needs
a combined up-type, e.g. `Yield = enum { Value(Y), Pending }`, with the scheduler
ignoring `Value` (or it being illegal at the top). Cleanest first cut: **disallow
explicit `yield` in implicitly-async functions**; revisit if needed.

### 8.8 Blocking calls inside async (the footgun)
If async code calls a genuinely blocking syscall (a non-effect one, or an effect
under the blocking handler), it stalls the scheduler thread. No keyword means no
compiler warning by default. Possible mitigation: a purity/effect annotation marking
"blocking" so the scheduler context can flag it. Ties into §8.1's effect-as-value
question.

### 8.9 Multithreading / work-stealing
Frames are heap objects; a task chain can in principle migrate threads between
suspensions if the scheduler is multi-threaded. Requires frames to be `Send`-safe and
the reactor to be shared/sharded. Out of scope for v1 (single-threaded loop), but the
frame model doesn't preclude it.

### 8.10 Debuggability
A logical stack trace is now a parent-link chain of frames, not a C stack. The
runtime can walk parent links to reconstruct an async backtrace — worth designing the
frame to keep enough info (the source location / state tag is already there via
`co_fstate`).

### 8.11 Interaction with implicits across suspension
Implicits are threaded as a hidden `_ctx` parameter and **already captured into the
coroutine frame** (`co_frame._ctx`). So an implicit bound with `with` outside an
await is correctly preserved across suspension — this already works in the current
lowering. Good. But the *scheduler's* implicits (reactor) must be in scope at the
leaf; verify the `with` discipline composes (the reactor `with` wraps `co main()`).

---

## 9. Worked example (trace)

```
Scheduler.run(|| with Net = async_net { serve(listener) })
```

1. `co serve` reified; `resume(serve)`.
2. `serve` calls `accept` (suspending) → reifies child `accept`, `resume`s it.
3. `async_accept` registers `listener.fd` READABLE with `current_coroutine()`,
   `yield Pending`. Bubbles up: `serve` records child, `yield Pending`. Scheduler
   parks task, loops.
4. Connection arrives → reactor: `fd → accept-leaf` → `resume(accept-leaf,
   ready)`. `accept` does nonblocking `accept(2)`, **returns** `socket`.
5. Leaf returned → scheduler resumes `serve` (parent) delivering `socket`. `serve`
   continues, calls `handle(...)` → new child → descends → `async_read` registers
   `socket.fd`, `yield Pending` → bubbles to `serve` → parks.
6. Data arrives → `resume(read-leaf, ready)` directly; read returns bytes; leaf
   returns; parent (`handle`/`read_request`) resumed with bytes; … and so on.

At no point between steps 4 and 6 is `serve` or any frame above the active leaf
touched until the leaf's function actually returns.

---

## 10. Primitive surface (what the language must add)

1. **Coloring analysis** — least-fixed-point "suspending" over the post-mono call
   graph (handlers as edges). New pass, just before coroutine lowering.
2. **Call-site transform** — in suspending functions, rewrite calls to suspending
   callees into the §4 drive sequence. Extends `HandlerLower` (same tree-walk that
   already expands `yield`).
3. **`resume` with a down-value** (§5) — ABI change or frame slot.
4. **`current_coroutine()`** — handle to the innermost resuming frame.
5. **Parent link + `awaiting` slot** in frames; scheduler-visible.
6. Everything else — reactor, scheduler, async I/O effect handlers, spawn/join/
   select — is **library code** in std.

## 11. Suggested staging

1. **Bidirectional resume / down-value** (§5 B) on the existing generator coroutines.
   Unblocks everything; testable in isolation.
2. **`current_coroutine()` + a trivial reactor** (single fd, `select(2)`), and a
   hand-written async read that registers + `yield`s. Prove the leaf-direct-wakeup
   loop with a *manually* coroutine-ized chain (no auto-coloring yet).
3. **Coloring analysis** (read-only: just compute + print colors) to validate the FLP
   against real programs (esp. effect handlers and indirection §8.1).
4. **Call-site transform** behind the analysis; run the http server async with the
   same source as sync.
5. **Cancellation** (§8.3) and **structured concurrency** (§8.5) once the core loop
   is solid.

---

### One-line summary

Because effects are resolved by monomorphization, "async" is just the suspending
partition of the monomorphized program; coroutines already give us per-frame
suspension with frame-saved locals and tree-walk suspension points; add a down-value
to `resume`, a `current_coroutine()` handle, and parent links, and a *userspace*
scheduler can drive the bottom frame directly on I/O and walk completions up one hop
at a time — transparent sync/async with no keywords.
