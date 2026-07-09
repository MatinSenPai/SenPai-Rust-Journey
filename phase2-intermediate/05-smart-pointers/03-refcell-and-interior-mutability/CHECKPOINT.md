# Checkpoint

1. Run the `a_second_borrow_mut_panics_while_the_first_is_still_alive` test
   and read the panic message. What specific `RefCell` invariant is being
   violated, in your own words?
2. `increment` and `get` both take `&self`, not `&mut self`. If `SharedCounter`
   didn't use `RefCell` at all — just a plain `inner: Rc<i32>` — could you
   write an `increment(&self)` that actually mutates the count? Why not?
3. `SharedCounter` derives `Clone`. What gets cloned when you call
   `.clone()` on it — a fresh `i32`, or something else? How does that
   answer explain why `clones_share_the_same_underlying_counter` passes?
4. Name a concrete data structure or scenario (not from this lesson) where
   you'd reach for `Rc<RefCell<T>>`, and explain why a single owner with
   `&mut` wouldn't work there.
