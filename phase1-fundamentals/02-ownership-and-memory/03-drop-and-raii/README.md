# 02.3 — Drop and RAII

## What actually happens when a value's owner goes out of scope

```rust
{
    let s = String::from("hello");
    // ... use s ...
} // s's scope ends HERE — Rust calls `drop(s)` automatically, right here,
  // deterministically. No garbage collector decides "later, whenever."
```

This pattern — "acquire a resource when a value is created, release it
automatically the instant its owner's scope ends" — has a name:
**RAII** (Resource Acquisition Is Initialization), and it's the
organizing idea behind almost all resource management in Rust: memory,
yes, but also file handles, network sockets, database connections, mutex
locks — anything that needs cleanup gets it automatically and
*deterministically*, tied to scope, not to whenever a GC happens to run.

Contrast with Python: `f = open("file.txt")` doesn't guarantee `f` is
closed at any particular point — you need `with open(...) as f:` (a context
manager) to get scope-tied cleanup, and it's opt-in, per-resource, decided
by whoever wrote that class. In Rust, **every** type gets this for free via
the `Drop` trait — there's no separate "remembered to close it" discipline
to maintain.

## The `Drop` trait

```rust
struct LoudResource {
    name: String,
}

impl Drop for LoudResource {
    fn drop(&mut self) {
        println!("Cleaning up: {}", self.name);
    }
}
```

Implementing `Drop` for a type lets you run custom cleanup logic the moment
a value goes out of scope. You'll use this later for things like "close
this database connection" or "release this lock" — for this lesson, you'll
use it to make cleanup *visible*, so you can watch ownership rules from the
previous two lessons play out in practice.

Two rules worth knowing now:
- You can't call `.drop()` directly (it's not a normal method call) —
  instead, if you need to drop something early, call `std::mem::drop(value)`
  (a free function that just takes ownership and immediately lets it go out
  of scope).
- Order matters: values drop in the **reverse** order they were declared,
  and moving a value (previous lessons) also moves *when* it gets dropped —
  the drop happens wherever the value's final owner's scope ends, not where
  it was originally created.

## Your task

`src/lib.rs` has a `Tracker` type that implements `Drop` by pushing a
message into a shared log (so tests can observe drop order without relying
on `println!` output). Implement the functions that exercise it.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`. That's ownership & memory
done — next up: borrowing.
