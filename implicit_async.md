# Implicit Async in Siko

> Design investigation. Goal: transparent sync/async I/O with **no `async`/`await`
> keywords and no changes to the coroutine protocol**, built on the coroutine and
> effect primitives the language already has. The scheduler/executor lives entirely in
> userspace; the language only supplies primitives.

The model is essentially Rust's `Future`/`poll`, but with the two things Rust spells
explicitly — `.await` and the `Context`/`Waker` parameter — supplied implicitly by
features Siko already has (monomorphization-resolved effects, and implicits).

---

## 0. The two primitives we build on

### 0.1 Coroutines

`co f(args)` reifies a call to a coroutine function as a heap value; `resume` drives
it one step:

```
co_wrap_N      ::= struct { state: co_state_N }                           // the handle (reference type)
co_state_N     ::= enum { Variant0(co_frame_N_0), ... }                   // one variant per co-fn of signature (Y,R)
co_frame_N_v   ::= struct { state: co_fstate_N_v, _ctx, <args...>, <locals...> }
co_fstate_N_v  ::= enum { Start, AfterYield_0, ..., AfterYield_k, Completed }
resume(c: co_wrap_N) -> CoResult[Y,R]    where CoResult = { Yielded(Y), Returned(R), Completed }
```

The handler is a **state machine**: it `match`es `frame.state`, `goto`s the current
suspension point, runs a segment, then either `frame.state = AfterYield_i; return
Yielded(v)` or `frame.state = Completed; return Returned(v)`. **All locals live in the
frame**, so they survive across suspensions. The frame is a GC'd reference type, mutated
through `resume`.

This is exactly Rust's `poll`: **`resume` ≙ `poll`**, **`Yielded` ≙ `Pending`**,
**`Returned(v)` ≙ `Ready(v)`**. We use it as-is; nothing about the protocol changes.

Two facts that matter below:

- **The lowering is a tree walk** (`HandlerLower`, an `auto(Map)` pass): every suspension
  point — anywhere in the body, nested in `if`/`match`/loops — is rewritten in place into
  `set-state / return / resume-label`, and the dispatch `goto`s into the middle of nested
  structures. The dispatch jumps **straight to the current suspension point**, never
  re-running completed segments.
- It runs after monomorphization **and after closure lowering** — closures must
  already be gone, so the transform never meets a `CreateClosure` (an indirect call it
  could neither color nor rewrite, §9.2). It works on the per-context monomorphic bodies.

### 0.2 Effects

An `effect` declares operations whose implementation is chosen at the call site with
`with op = handler { ... }`. The detail that matters:

- **Effect operations are resolved during monomorphization.** The monomorphizer carries
  an effect context (operation → chosen handler); at an effect call it emits a direct
  call to that handler.
- So **the same source function is specialized per effect context.** `do_work` under
  `with read = blocking_read` and under `with read = async_read` becomes two distinct
  monomorphic functions.
- **Handlers are ordinary functions that run at the effect call site**, threaded like
  implicits. Effects are not stack-unwinding continuations.

A handler is therefore normal code executing where the I/O conceptually happens — at the
bottom of the call chain. If that code suspends, it suspends exactly there.

---

## 1. The idea

```
effect Net { fn read(s: Socket, buf: Slice) -> Int ; fn accept(l: Listener) -> Socket ; ... }

fn handle(req)      { ... Net.read(sock, buf) ... }     // no async anything
fn serve(listener)  { let s = Net.accept(listener); handle(read_request(s)) }
fn main()           { serve(open(8080)) }
```

Run the *same source* two ways:

- **Sync:** `with Net = blocking_net { main() }` — `Net.read` is a blocking syscall that
  returns directly. Nothing is a coroutine. Zero overhead.
- **Async:** `Executor.run(|| with Net = async_net { main() })` — `Net.read` registers
  interest with a reactor and suspends; one thread multiplexes many such tasks.

No keyword distinguishes them. The difference is **which handler is in scope**, resolved
by monomorphization. The mechanism is three parts:

1. **Coloring** (§2, §3) — a monomorphic function that might suspend is compiled as a
   coroutine.
2. **The call-site transform** (§4) — *calling* a coroutine-colored function is the
   suspension point; the compiler injects the drive logic. (This is what Rust writes as
   `.await`; here it is implicit — an ordinary call.)
3. **A userspace executor** (§5–§7) — `co`s the top of a task, re-polls it on its
   wakeup, and re-descends cheaply to wherever it left off.

---

## 2. Why mono-resolved effects make this work

"Is this function async?" is normally undecidable without an `async` annotation, because
it depends on what callees do, which depends on runtime handlers. In Siko it is decidable
**after monomorphization**, because handlers are chosen per mono context.

Call a monomorphic function **suspending** if it `yield`s, or calls an effect operation
whose chosen handler (in this mono context) is suspending, or calls another suspending
monomorphic function. This is a least-fixed-point over the **post-mono call graph**, with
handlers as ordinary edges. Compute it once; every monomorphic function's color is known.

`handle` is suspending under `async_net` and non-suspending under `blocking_net` — and
those are genuinely different monomorphic functions, so nothing conflicts and no color is
lost to dynamic dispatch (modulo indirection, §9.2).

> The async/sync split is just the **suspending partition of the monomorphized program**,
> and monomorphization already split the program by handler. That is what buys
> keyword-free async with zero-cost sync.

---

## 3. Coloring: who becomes a coroutine

After mono + the suspending analysis:

- A **suspending** monomorphic function is lowered as a coroutine, even with no explicit
  `yield`. Its suspension points are explicit `yield`s and every call to a suspending
  callee (§4).
- A **non-suspending** function is left exactly as today — a plain C function, no frame,
  no overhead.

An async coroutine yields `()` (a "pending" signal with no payload — *what* it waits on is
registered as a side effect, §6) and returns its real `R`. So its type is `co(()) -> R`,
and a whole task — entered at `main : () -> ()` — is `co(()) -> ()` (§7).

A suspending `f : A -> R` is therefore invoked as `co f(a)` and driven; the compiler
rewrites the call sites. The user never writes `co` for this — that stays the surface
syntax for explicit generators.

---

## 4. The call-site transform (no `await` — it's just a call)

There is no `await`. A plain call `r = g(args)` to a coroutine-colored `g` is the
suspension point; the compiler turns it into "reify the callee, drive it, and if it's
pending, suspend here and propagate":

```
// state AwaitG, entered on the first poll that reaches the call AND on every re-poll:
let child = co g(args);            // (first time) reify callee; saved in the frame thereafter
match resume(child) {
    Returned(r) => r               // Ready: continue inline with r
    Yielded(()) => { yield () }    // Pending: suspend here; re-polled later, re-enters AwaitG
    Completed   => unreachable
}
```

This is structurally the same as the explicit-`yield` expansion already in `HandlerLower`
(set state, return, drop a resume label) — and the existing tree-walk handles it in any
expression position, for multiple calls and early returns, for free. On re-poll the
dispatch jumps straight back to `AwaitG` and re-`resume`s the saved child; the child's own
dispatch jumps to *its* current point; and so on down to the leaf. Completed segments are
never re-run.

`Net.read` under `async_net` is, after mono, a call to a suspending handler — so the
user's plain `Net.read(...)` *is* one of these suspension points, with zero syntax.

The value flows **up** through `resume`'s `Returned(v)` (= `Ready(v)`); there is no
down-channel and no "future" object for results.

---

## 5. Wakeup: re-poll the task from the top

We wake at **task** granularity, like Rust — not by poking individual frames.

A task is the chain of coroutine frames rooted at one `co main()`-style top. The executor
holds **only the top handle** per task. On a wakeup it does `resume(top)`, which
re-descends through the §4 call sites (each `resume(child)` jumps to that child's current
suspension point) until it reaches the frame that was blocked. That frame re-attempts its
operation; if it's now ready it returns, and values cascade back up through `Returned`
until the task suspends again or the top returns.

Re-descent is O(depth), but each step is a state-machine dispatch + a `resume` that jumps
to the saved point — no completed work is re-run, no sibling is touched. The "idle chain"
between the top and the blocked frame costs ~nothing to re-enter. This is exactly Rust's
re-poll, and it is why none of the frame-addressing machinery (per-frame handles,
continuation links, wrappers, type-erasure of inner frames) is needed: the executor never
addresses anything but the top.

The only thing woken on an event is the *one task* whose handle the reactor holds; every
other task is untouched.

---

## 6. The wakeup capability is an ordinary implicit

To re-poll a task, the blocked leaf must register "wake *this task*" with the reactor.
That capability — call it `current_task` — is **constant for the whole task** (it always
means `resume(top)`), so it is a textbook implicit: bound once at the top, threaded down
the chain, read at the bottom. (This is Rust's `Waker`; Rust threads it as an explicit
`Context` parameter through every `poll`, but "context threaded down a call chain" is what
implicits *are*, so we get it for free.)

The executor binds it; registration is an ordinary library function reading it:

```
with current_task = task {        // ordinary implicit, set once at the top
    resume(top)
}

fn park_on(fd, interest) {        // library; itself suspending, hence a coroutine
    loop {
        if reactor.ready(fd, interest) { return; }      // re-checked on every (re-)poll
        reactor.register(fd, interest, current_task);   // "re-poll this task" when ready
        yield ();                                        // Pending
    }
}

fn async_read(s, buf) -> Int {
    park_on(s.fd, READABLE);
    nonblocking_read(s.fd, buf)
}
```

Because implicits are captured into each frame's `_ctx` at `co`-creation, every frame in
the task — down to the deepest leaf — sees the same `current_task` the executor bound, and
sees it again on re-poll. It is a normal implicit: bound by a `with`, lowered by the
existing implicit-propagation pass, no per-frame variation, no ordering problem, no
thread-local.

Setup wrinkle (solved the normal way): `current_task` wants the top handle, created in the
same breath, so it closes over a small mutable `Task` cell whose `.top` is filled right
after `co main()`. Ordinary value, ordinary implicit.

---

## 7. The schedulable type, the executor, the reactor

- **Schedulable = `co(()) -> ()`, enforced by the ordinary typechecker.** `Executor.spawn`
  and `Executor.run` take `co(()) -> ()`. `co main()` (where `main : () -> ()`) is one.
  Intermediate frames keep their real `co(()) -> R_i` types and are driven by §4; the
  executor never holds them. Trying to schedule something that returns a value is a *type
  error* — results leave a task through channels/join-handles (§9.5), not the executor.
  "What is schedulable" is answered entirely by a type, with no special compiler support.
- **Executor** (plain Siko): a ready queue of tasks. `run` resumes a ready task's top;
  `Yielded(())` → leave it parked (it has registered itself with the reactor); `Returned`
  → the task is done. A wakeup pushes a task back onto the ready queue.
- **Reactor** (epoll/kqueue/io_uring wrapper): `fd → current_task`. On an event it enqueues
  the task. Handed to suspending handlers via the scheduler's `with`.

Everything above `co` / `resume` / coloring / the §4 transform is library code.

---

## 8. Worked example (trace)

```
Executor.run(|| with Net = async_net { serve(listener) })
```

1. The executor `co`s the task top and `resume`s it under `with current_task = task`.
2. `serve` calls `accept` (suspending) → §4 reifies the child and `resume`s it; down to
   `async_accept` → `park_on(listener.fd, READABLE)` registers `current_task`, `yield ()`.
   `Yielded` cascades up the §4 sites to the top; the executor parks the task and loops.
3. Connection arrives → reactor enqueues the task → executor `resume(top)`. The dispatch
   re-descends to `park_on`, which now sees `ready`, returns; `async_accept` does the
   nonblocking `accept(2)` and `Returned(socket)`; that cascades up through the §4 sites,
   `serve` continues with `socket`, calls `handle(...)`, descends to `async_read`, parks
   on `socket.fd`, `Yielded` to the top, parked again.
4. Data arrives → re-poll → re-descend to the read → `Returned(bytes)` up the chain → …

Each event wakes exactly one task; within it, re-descent only ever dispatches to the saved
suspension point at each level.

---

## 9. Open questions

### 9.1 Cancellation + resource cleanup (no destructors) — the big one
Siko is GC'd with no destructors. Dropping a task frees its frame tree via GC, but a
registered fd must be **deregistered** from the reactor and half-open resources released,
and there is no automatic unwinding of a parked chain. Needs an explicit `cancel(task)`
routed so the parked frames run cleanup (a `defer`/cleanup effect, or a "cancel" resume
mode that runs finalizers) — or resources modeled as an effect the executor can finalize.
No clean answer yet.

### 9.2 Indirection erases color
`fn`-pointers, closures, trait/dynamic dispatch, and effect handlers passed as runtime
values defeat static coloring: at an indirect call the compiler can't tell if the target
suspends, so it can't know whether to inject the §4 drive code. Options: color the
function *type* (a pointer to a suspending fn is a distinct type; calls through it always
drive); or a uniform "drive every indirect call" ABI (a non-suspending target just
`Returned`s on the first `resume`); or forbid taking a coroutine fn as a plain pointer.

### 9.3 Blocking calls inside async (footgun)
Async code that calls a genuinely blocking syscall (or an effect under the blocking
handler) stalls the executor thread, with no keyword to warn. Mitigation: a "blocking"
annotation the executor context can flag.

### 9.4 Generators + async share the yield channel
Explicit `yield` (generator, user `Y`) and async (`Y = ()`, pending) share `resume`'s
yield channel. A function that is both needs a combined up-type. First cut: forbid explicit
`yield` in implicitly-async functions.

### 9.5 Structured concurrency
A task is `co(()) -> ()`, so results and coordination (spawn-N, join, select, timeouts,
channels) are **library** constructs over the same primitives: a parked task registers on a
channel/join the way the leaf registers on an fd; completing the producer enqueues the
waiter. Worth prototyping early to confirm the primitive set is sufficient.

### 9.6 Cost, and the zero-cost version
Re-poll is O(depth) cheap dispatches per event — the same trade Rust makes. Rust's truly
zero-cost form comes from *nesting* each child future inline into its parent (one flat
struct, inlined polls) instead of heap-linking frames. Siko heap-links frames today; the
nesting optimization is a separate, much larger change and explicitly not now.

### 9.7 Multithreading
Heap frames could migrate threads between suspensions if the executor is multi-threaded
(work-stealing); needs `Send`-safe frames and a shared/sharded reactor. Out of scope for a
single-threaded v1, but the model doesn't preclude it.

### 9.8 Debuggability
The logical stack is the re-poll path, not the C stack. The runtime can reconstruct an
async backtrace from the chain of saved `co_fstate` tags (each records its suspension
point).

### 9.9 Implicits across suspension
Implicits are threaded as a hidden `_ctx` and already captured into the coroutine frame, so
a `with`-bound implicit (including `current_task` and the reactor) survives suspension and
re-poll. This already works in the current lowering; the design relies on it.

---

## 10. What the language must add

1. **Coloring analysis** — least-fixed-point "suspending" over the post-mono call graph
   (handlers as edges). New pass, just before coroutine lowering.
2. **The call-site transform** — in suspending functions, rewrite calls to suspending
   callees into the §4 drive sequence. Extends `HandlerLower` (the same tree-walk that
   expands `yield`).

Unchanged: `co`, `resume`, `yield`, the `CoResult` protocol, the implicit mechanism.
Library: the `Task`/`current_task` capability, the reactor, the executor, async I/O effect
handlers, channels/join/select.

---

## 11. Suggested staging

1. **Manual proof-of-runtime, zero compiler changes.** With today's coroutines, hand-write
   an `async_read`/`park_on` using `current_task` (a normal implicit) and a trivial reactor
   (`select(2)`), and an executor that `resume`s a `co(()) -> ()` top and re-polls on
   wakeup. Manually coroutine-ize a small chain by writing the §4 drive code by hand. This
   validates the entire runtime shape — re-poll-from-top, the implicit waker, the reactor —
   before any compiler work.
2. **Coloring analysis**, read-only (compute + print), validated against real programs
   (effects and indirection, §9.2).
3. **The call-site transform** behind the analysis; run the http server async with the same
   source as sync.
4. **Cancellation** (§9.1) and **structured concurrency** (§9.5) once the core loop is
   solid.

---

### One-line summary

`resume`/`Yielded`/`Returned` already *are* `poll`/`Pending`/`Ready`; calling a
coroutine-colored function (no keyword) is the suspension point; the executor wakes a task
and re-polls it from the top, re-descending cheaply via the dispatch that jumps straight to
each saved point; and the only "context" needed — "wake this task" — is an ordinary
implicit, the thing Rust bolts onto its ABI as `Waker` and Siko already has. Coloring falls
out of monomorphization, so it's transparent sync/async with no keywords, no protocol
change, and no per-frame scheduling machinery.
