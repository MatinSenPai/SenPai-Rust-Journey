# Solution

```rust
pub fn share_config(config: AppConfig) -> Rc<AppConfig> {
    Rc::new(config)
}

pub fn add_owner(shared: &Rc<AppConfig>) -> Rc<AppConfig> {
    Rc::clone(shared)
}

pub fn owner_count(shared: &Rc<AppConfig>) -> usize {
    Rc::strong_count(shared)
}

pub fn same_allocation(a: &Rc<AppConfig>, b: &Rc<AppConfig>) -> bool {
    Rc::ptr_eq(a, b)
}

pub fn share_config_across_threads(config: AppConfig) -> Arc<AppConfig> {
    Arc::new(config)
}
```

Five one-liners, and every one of them is just naming an `Rc`/`Arc` associated
function — the actual work already lives in the standard library. That is the
point: shared ownership is not a pattern you build, it is a type you reach
for.

## `add_owner` — the line worth staring at

`Rc::clone(shared)` does not touch the `AppConfig` on the heap at all — no new
`String` allocation for `app_name`, no copy of `max_connections`. All that
happens is the `Rc`'s internal count goes up by one, and you get back a
second pointer to the *same* allocation. That is the whole reason the
lesson insists on writing `Rc::clone(shared)` instead of `shared.clone()`
even though they compile to the exact same call: at the call site, the
explicit spelling says "cheap pointer clone," where the dot-call spelling
makes the reader go check what type `shared` actually is before they can
tell.

## `same_allocation` — the proof, not just the claim

`Rc::ptr_eq(a, b)` answers a genuinely different question than `*a == *b`
would. `*a == *b` asks "do these two `AppConfig`s hold equal fields?" —
true for any two configs built with the same `app_name` and
`max_connections`, even from two completely unrelated `Rc::new` calls.
`Rc::ptr_eq` asks "are these two handles pointing at the *same* heap
allocation?" — true only when one was produced by cloning the other's `Rc`,
never for two independently constructed ones, no matter how equal their data
looks. `same_allocation_is_false_for_two_equal_but_separate_rcs` exists
specifically to keep those two questions from blurring together: it builds
two `Rc<AppConfig>`s from identical data, confirms `*a == *b` (equal
values), and confirms `same_allocation` still says `false` (different
allocations).

## `owner_count` and the drop test

`owner_count` is just `Rc::strong_count` under a name that says what it
means for this lesson. The interesting part isn't the function — it's
`dropping_an_owner_decrements_the_count`, which relies on the same
automatic-cleanup rule Phase 1 already gave you: a value's `Drop` runs the
instant its owner's scope ends, with nothing left to a garbage collector's
schedule. `_second`'s scope is the inner `{ }` block; when that block
closes, `_second` is dropped, and `Rc`'s own `Drop` implementation is what
actually decrements the count (and, had it just reached zero, would have
freed the `AppConfig` too). You get this for free — it's the same
end-of-scope rule you already knew, just decrementing a counter this time
instead of immediately freeing memory, because the value going out of
scope is the `Rc` handle, not the data underneath it.

## `share_config_across_threads` — same shape, different counter

`Arc::new` reads exactly like `Rc::new` because the API was designed to.
Nothing in this function (or its test) actually crosses a thread — that
part of the story belongs to module 8. What matters today is recognizing
that `Arc<T>`'s count is incremented and decremented with atomic CPU
instructions instead of `Rc<T>`'s plain ones, which is the only reason
`Arc<T>` is safe to hand to a second thread at all: a plain integer
increment is a read-then-write, and two threads doing that at once to the
same non-atomic counter can lose an update, letting the count reach zero
— and the value get freed — while a handle to it still exists somewhere.
That extra safety is not free, which is exactly why the lesson does not
say "always use `Arc`": pay for the atomic counter only once a value
genuinely needs to leave one thread.
