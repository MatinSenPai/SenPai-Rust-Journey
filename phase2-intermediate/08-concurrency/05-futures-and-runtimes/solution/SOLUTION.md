# Solution

`Countdown::poll` is the same two-way branch every hand-rolled `Future` in this lesson used — finished, or one step closer:

```rust
fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
    if self.remaining == 0 {
        Poll::Ready(self.total_polls)
    } else {
        self.remaining -= 1;
        Poll::Pending
    }
}
```

`self.remaining -= 1` writes straight through `Pin<&mut Self>` with no special `Pin` gymnastics at all — `Countdown` holds no reference into its own fields, so Rust gives it `Unpin` automatically, and for an `Unpin` type `Pin<&mut Self>` is barely more than a plain `&mut Self` in practice. The `mut` on `self` is required, not decoration: writing through `Pin`'s `DerefMut` needs a mutable binding of the `Pin` itself, not just of what it points at.

`block_on` is the general-purpose executor from "The concept," unchanged:

```rust
pub fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => continue,
        }
    }
}
```

`Box::pin(future)` is what makes the single type parameter `F: Future` — with no `Unpin` bound anywhere — actually work. A real `async fn`'s state machine is always `!Unpin` (see "Errors you will meet" — `E0277` for exactly what happens if you try `Pin::new` on one instead), so a `block_on` that only accepted `Unpin` futures would work for `Countdown` and fail on every genuine `async fn`. Putting the future on the heap once, behind `Box::pin`, sidesteps the question entirely: it never moves again after that, so it's always safe to poll, `Unpin` or not. That's exactly what the fourth test, `block_on_also_drives_a_real_async_fn`, is checking — the same `block_on` driving a plain `async fn` to completion, no different from driving `Countdown`.

`Waker::noop()` hands back a `Waker` that does nothing when woken — fine here specifically because this `block_on` never actually sleeps waiting for a wake-up; it just spins and re-polls immediately regardless, which is correct (if wasteful) for futures like `Countdown` that always have a real answer ready on the very next poll. A real executor passes a genuine `Waker`, wired back to itself, precisely so a future that's actually waiting on I/O can call `.wake()` when data arrives — the signal a real executor uses to know when it's worth polling again, instead of busy-looping the way this one does.
