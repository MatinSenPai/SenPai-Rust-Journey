# 05.2 — `Rc` and `Arc`

## Breaking "exactly one owner," on purpose

Phase 1 drilled in one rule: every value has exactly one owner. That rule
is what makes Rust's memory management deterministic and predictable — but
sometimes it doesn't map cleanly onto the problem you're solving. What if
several parts of a program all genuinely need to hold onto the *same*
piece of data, for as long as any of them are still using it, and there's
no obviously "correct" single owner? Example: a shared application config
loaded once at startup, read by a web handler, a background job, and a
logger — none of those three should be "the" owner responsible for
dropping it out from under the other two.

`Rc<T>` ("**r**eference **c**ounted") is Rust's answer: multiple owners,
sharing the same heap-allocated value, with the value only freed once the
*last* owner drops it.

```rust
use std::rc::Rc;

let a = Rc::new(String::from("shared"));
let b = Rc::clone(&a); // a second owner of the SAME String — not a copy
let c = a.clone();     // identical to Rc::clone(&a); just different syntax
```

## `Rc::clone` is cheap — it does not copy your data

This is the single most important thing to internalize about `Rc`, because
the syntax is a trap: `a.clone()` on an `Rc<String>` and `a.clone()` on a
plain `String` **look identical** but do completely different things.

- `some_string.clone()` deep-copies the string's heap data — a whole new
  allocation, byte-for-byte identical contents.
- `Rc::clone(&some_rc)` (equivalently, `some_rc.clone()`) does **not** touch
  the data it points to at all. It just increments an internal counter and
  hands back a new `Rc` pointing at the *same* allocation. It's a few
  machine instructions, regardless of whether the `T` inside is a `u8` or a
  10 MB struct.

Preferring `Rc::clone(&a)` over `a.clone()` in your own code (even though
they compile to the same thing) is a real Rust convention — it makes "this
is a cheap pointer clone, not a data copy" visible at the call site instead
of relying on the reader to already know what type `a` is.

## Watching the count

```rust
use std::rc::Rc;

let a = Rc::new(5);
println!("{}", Rc::strong_count(&a)); // 1
let b = Rc::clone(&a);
println!("{}", Rc::strong_count(&a)); // 2
drop(b);
println!("{}", Rc::strong_count(&a)); // back to 1
```

Every `Rc::clone` increments the count; every time an `Rc` is dropped, the
count decrements. The underlying `T` is only actually deallocated when the
count hits zero — i.e. when the *last* handle to it goes away. None of the
owners is special; there's no "original" vs. "copies" once you're holding
an `Rc` — they're all equally valid owners.

## `Rc` doesn't work across threads — that's what `Arc` is for

`Rc`'s counter is a plain, non-atomic integer. Incrementing it from two
threads at once is a data race, so Rust simply won't let you send an `Rc`
across a `std::thread::spawn` boundary — it doesn't implement the `Send`
marker trait, and trying anyway is a compile error, not a runtime bug.

`Arc<T>` ("**a**tomically **r**eference **c**ounted") is the thread-safe
version — same API (`Arc::new`, `Arc::clone`, `Arc::strong_count`), but the
counter is incremented/decremented using atomic CPU instructions, which
are safe to touch from multiple threads simultaneously at the cost of
being slightly slower than `Rc`'s plain increments. Rule of thumb: use
`Rc` in single-threaded code, switch to `Arc` the moment a value needs to
be shared with a spawned thread (module 07 goes deep on threads — for now,
just know `Arc` is the tool that makes "share this across threads"
possible at all).

## When to actually reach for these

Default to normal ownership and borrowing (`&T`/`&mut T`) — it's simpler,
fully checked at compile time, and has zero runtime cost. Reach for
`Rc`/`Arc` only when you've genuinely got multiple long-lived owners and
restructuring the code around a single clear owner isn't practical (a
shared config, a node in a graph/tree with multiple parents, a cache
shared across independent subsystems). Rc/Arc alone only let you *share*
a value — every owner gets read-only access via `&T`. If you also need to
*mutate* the shared value, you need to combine it with something that
provides interior mutability, which is exactly next lesson's topic.

## Your task

Implement the four functions in `src/lib.rs`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
