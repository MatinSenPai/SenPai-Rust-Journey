# 07.2 — Channels and message passing

Last lesson's `Arc<Mutex<T>>` shares *state* across threads — every thread
reaches into the same box and takes turns touching it. Channels are a
different model entirely: instead of sharing memory, threads **send
values** to each other. Rust's standard library slogan for this, borrowed
from Go: "Do not communicate by sharing memory; instead, share memory by
communicating."

## `mpsc`: multi-producer, single-consumer

```rust
use std::sync::mpsc;
use std::thread;

let (tx, rx) = mpsc::channel();

thread::spawn(move || {
    tx.send(42).unwrap();
}); // tx is moved into the closure, dropped when the thread ends

let received = rx.recv().unwrap(); // blocks until a value arrives
assert_eq!(received, 42);
```

`mpsc::channel()` returns a `(Sender<T>, Receiver<T>)` pair. `tx.send(v)`
never blocks (values queue up); `rx.recv()` blocks until a value is
available, returning `Err` only once every `Sender` has been dropped and
nothing more will ever arrive. `Sender<T>` implements `Clone` — that's the
"multi-producer" part: clone it once per thread that needs to send, and
every clone feeds the *same* receiver.

## `rx` as an iterator, and why dropping every `Sender` matters

```rust
for value in rx {
    println!("{value}");
} // loop ends once every Sender (including all clones) has been dropped
```

`Receiver<T>` implements `IntoIterator`, yielding values as they arrive and
ending automatically once every corresponding `Sender` clone is gone —
this is why, when you spawn several producer threads each with their own
`tx.clone()`, you must also `drop(tx)` (the *original* sender you kept in
the parent thread) once every clone has been handed off — otherwise the
iterator waits forever for a `Sender` that's just sitting there unused,
since it doesn't know you'll never call `.send()` on it.

## Channels vs. `Mutex`: which one, when

- Need one thread to hand a **result** back to another, once, when it's
  done? A channel (or, as you've already used, a plain `JoinHandle`'s
  return value) is usually simpler than shared state.
  `compute_async_sum` below does this.
- Need multiple threads to keep independently **feeding data** to one
  place, over time, without a shared collection everyone locks? A channel.
  `collect_from_producers` below does this.
- Need multiple threads to read *and* write the *same* live piece of
  state, with each read seeing the latest writes? `Arc<Mutex<T>>` — a
  channel doesn't naturally model "shared, continuously-updated state,"
  only "hand this value over."

## Your task

Implement both functions in `src/lib.rs`.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
