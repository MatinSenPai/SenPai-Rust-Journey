# Solution

```rust
pub fn increment(&self) {
    *self.inner.borrow_mut() += 1;
}

pub fn get(&self) -> i32 {
    *self.inner.borrow()
}
```

`self.inner` is `Rc<RefCell<i32>>` — `.borrow_mut()` works through the
`Rc`'s shared reference to the `RefCell` (no `&mut self` needed anywhere,
since `RefCell` moves the mutability check to run time), returning a
`RefMut<i32>` which `*` dereferences to read/write the actual `i32`. The
`RefMut`/`Ref` guards are dropped at the end of each statement here (since
they're temporaries, not bound to a variable), which is exactly why calling
`increment()` twice in a row never panics — each call's borrow is fully
released before the next one starts.

On checkpoint question 2: without `RefCell`, `inner: Rc<i32>` gives every
clone only `&i32` access — `Rc` itself doesn't allow mutation through a
shared reference (that's exactly the "aliasing XOR mutability" rule from
Phase 1, enforced by `Rc`'s API not exposing any way to mutate what it
points to). You'd be stuck with `&self` methods that can only read, never
write — `RefCell` is specifically what reintroduces mutation, by moving the
"only one mutable access at a time" check from compile time to run time.
