# 05.3 — `RefCell` and interior mutability

## The rule, moved from compile time to run time

Since Phase 1's borrowing module, you've relied on "aliasing XOR
mutability": at any point, you can have many `&T` (shared, read-only
borrows) **or** exactly one `&mut T` (exclusive, mutable borrow) — never
both at once — and the compiler checks this *statically*, before your
program ever runs. That's `RefCell<T>`'s whole reason for existing: it
enforces the **exact same rule**, just at *run time* instead.

```rust
use std::cell::RefCell;

let cell = RefCell::new(5);

let a = cell.borrow();      // like &T — shared, read-only
println!("{}", *a);
drop(a);                    // must end before borrow_mut, or this panics

let mut b = cell.borrow_mut(); // like &mut T — exclusive, mutable
*b += 1;
```

`.borrow()` and `.borrow_mut()` work through a plain shared reference
(`&RefCell<T>`) — no `&mut` needed to call `.borrow_mut()`. `RefCell`
tracks, internally, how many shared borrows and how many mutable borrows
are currently "live" (not yet dropped). If you try to take a `borrow_mut()`
while another borrow (shared or mutable) is still alive, it doesn't refuse
to compile — it **panics at run time**, with a message like `already
borrowed: BorrowMutError`.

## Why trade a compile-time guarantee for a run-time one?

Because sometimes the *compiler's* borrow checker is stricter than what's
actually safe, particularly once you're several layers of abstraction
deep — e.g. mutating one logical part of a shared structure while reading
another part through code the checker can't see all the way through (a
callback, a trait method, a tree with shared child nodes). `RefCell` gives
you an escape hatch: the compiler steps back, and in exchange you take on
the responsibility of not violating the rule — with a loud panic, not
silent memory corruption, if you get it wrong. This is *not* `unsafe` —
`RefCell` is 100% safe Rust, it just checks its invariant later than usual.

## The big combo: `Rc<RefCell<T>>`

Last lesson: `Rc<T>` gives you multiple owners, but every owner only gets
`&T` — read-only access to the shared value. Wrap the `T` in a `RefCell`
and every owner can call `.borrow_mut()` through their own `Rc` clone and
actually mutate the shared data:

```rust
use std::cell::RefCell;
use std::rc::Rc;

let counter = Rc::new(RefCell::new(0));
let handle = Rc::clone(&counter);

*handle.borrow_mut() += 1;
*counter.borrow_mut() += 1;

assert_eq!(*counter.borrow(), 2); // both handles mutated the SAME value
```

This combination — multiple owners (`Rc`), each able to mutate the shared
value (`RefCell`) — is how you get genuinely shared, mutable state in
single-threaded Rust. (The thread-safe version of this combo is
`Arc<Mutex<T>>`, which you've already glimpsed in the glossary and will
build hands-on in module 07 — same idea, `Mutex` does at run time, across
threads, what `RefCell` does at run time, within one thread.)

**This should be a deliberate, occasional tool — not a reflex.** If you can
restructure your code so there's one clear owner and you pass `&mut`
around instead, do that: it's simpler, and the compiler checks it for you
at compile time with zero runtime cost. Reach for `Rc<RefCell<T>>`
specifically when you have multiple long-lived owners that all genuinely
need to mutate shared state and there's no way to designate a single
owner — a tree/graph where child nodes are shared between parents, an
observer pattern where several listeners need to update shared state, a
cache shared across independent subsystems.

## Your task

Implement `SharedCounter` in `src/lib.rs`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
