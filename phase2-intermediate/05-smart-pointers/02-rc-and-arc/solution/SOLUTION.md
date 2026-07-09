# Solution

```rust
pub fn clone_handle(shared: &Rc<AppConfig>) -> Rc<AppConfig> {
    Rc::clone(shared)
}
```

This is the one line in the whole lesson worth staring at, because it's a
trap if you don't already know the difference: `Rc::clone(shared)` and
`shared.clone()` compile to the *exact same call* (`Rc::clone` is just the
explicit, "look, I mean the pointer clone" spelling of the same method
`.clone()` resolves to via the `Clone` trait). Neither one touches the
`AppConfig` on the heap — no new `String` allocation for `app_name`, no
copy of `max_connections`. All that happens is the `Rc`'s internal counter
goes up by one, and you get back a second pointer to the same allocation.
Compare that to `some_string.clone()`, which allocates a whole new buffer
and copies every byte. Same method-call syntax, opposite cost — which is
exactly why the convention is to write `Rc::clone(&x)` in real code: it
signals "cheap pointer clone" at the call site instead of making the
reader go check what type `x` is.

On checkpoint question 3: `dropping_a_clone_decrements_the_count` relies on
the same automatic-cleanup mechanic from Phase 1's ownership module —
every value's `Drop` runs the instant its owner goes out of scope, no
garbage collector deciding "later" is fine. `_handle`'s scope is the inner
`{ }` block; when that block ends, `_handle` is dropped, and `Rc`'s `Drop`
implementation is what actually decrements the strong count (and, if it
had just hit zero, would free the underlying `AppConfig` too). You get this
for free — it's the same "value's lifetime ends at the end of its scope"
rule you already know, just doing something slightly different (decrement
a counter instead of immediately freeing memory) because `Rc` is the thing
being dropped.

On checkpoint question 4: `Rc<T>`'s reference count is a plain `Cell<usize>`
internally — an ordinary, non-atomic integer. "Increment by one" on a plain
integer is not a single indivisible CPU operation; if two threads did it
"at the same time" you could lose an increment (both read the same old
value, both write back old+1, and the count ends up too low — a classic
data race, meaning the underlying value could be freed while something is
still using it). The Rust compiler prevents this entirely at compile time:
`Rc<T>` does not implement the `Send` marker trait, so attempting to move
one into a `thread::spawn` closure is a compile error, not a runtime bug
you'd have to debug. `Arc<T>` fixes it by using **atomic** integer
operations for its counter — CPU instructions specifically guaranteed to
be indivisible even when multiple cores touch the same memory location at
once. That safety is why `Arc` is implemented and does implement `Send`,
and also why it's measurably slower per-clone than `Rc` (atomic operations
have more overhead than a plain increment) — which is the whole rationale
for defaulting to `Rc` single-threaded and only paying for `Arc` when
threads are actually involved.
