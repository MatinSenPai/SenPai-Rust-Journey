# Checkpoint

1. `Rc::clone(&a)` and `a.clone()` on an `Rc<AppConfig>` do the exact same
   thing, but `Rc::clone(&a)` is the preferred style in real Rust code.
   Why — what does it communicate to a reader that `a.clone()` doesn't?
2. Contrast `Rc::clone(&some_rc)` with calling `.clone()` on a `String` or
   `Vec`. What actually happens on the heap in each case?
3. `dropping_a_clone_decrements_the_count` puts `_handle` inside an inner
   `{ }` block. Why does the reference count go back down to 1 after that
   block ends, without any explicit `drop(_handle)` call?
4. Why won't the following compile: spawning a `std::thread::spawn` closure
   that captures and uses an `Rc<T>`? What specifically about `Rc` makes it
   unsafe to share across threads, and what does `Arc` do differently at
   the machine-instruction level to fix it?
5. The README says "Rc/Arc alone only let you share a value — every owner
   gets read-only access." If you had an `Rc<AppConfig>` and genuinely
   needed one of the owners to *mutate* `max_connections`, would that
   compile as written? What do you think you'd need to add to make it
   possible? (You don't need the answer yet — next lesson covers it.)
