# Solution

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

Each call to `poll` either finishes the countdown (`remaining == 0`,
return `Ready` with the total) or ticks it down by one and reports
`Pending`. Note `self.remaining -= 1` works directly through `Pin<&mut
Self>` here without any special `Pin` gymnastics — `Countdown` is `Unpin`
(no self-referential data), so `Pin` is barely more than a regular `&mut`
reference for it in practice. `!Unpin` futures (the kind `async fn` bodies
usually produce, once they hold a reference across an `.await` point) are
where `Pin`'s real restrictions bite — a topic for further reading, not
this lesson's scope.

```rust
pub fn block_on<F: Future + Unpin>(mut future: F) -> F::Output {
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    loop {
        match Pin::new(&mut future).poll(&mut cx) {
            Poll::Ready(v) => return v,
            Poll::Pending => continue,
        }
    }
}
```

`Waker::noop()` gives a `Waker` that does nothing when woken — appropriate
here specifically because this `block_on` never actually goes to sleep
waiting for a wake-up; it just spins and re-polls immediately regardless.
A real executor passes a *real* `Waker` into `poll`, one wired back to
itself, so that a `Future` genuinely waiting on I/O can call
`waker.wake()` when data arrives — that's the signal the executor uses to
know "stop sleeping, go poll this one again," instead of guessing by
busy-looping. `Pin::new(&mut future)` is only valid because of the `F:
Unpin` bound on `block_on` itself — for a `!Unpin` future, you'd need
`std::pin::pin!` (stack-pinning) or `Box::pin` (heap-pinning) instead,
since `Pin::new` specifically requires the type to promise it's safe to
move (which is exactly what `Unpin` means).
