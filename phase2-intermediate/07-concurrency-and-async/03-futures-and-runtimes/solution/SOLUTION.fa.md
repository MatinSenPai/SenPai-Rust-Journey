# راه‌حل

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

هر poll countdown را کم می‌کند یا `Ready` می‌دهد. `Countdown: Unpin` است؛ Future خودارجاع پس از `.await` ممکن است `!Unpin` باشد.

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

`Waker::noop()` اینجا کافی است چون executor نمی‌خوابد. executor واقعی Waker متصل به scheduler می‌دهد تا completion I/O، task را دوباره runnable کند. شرط `Unpin` امکان `Pin::new` را می‌دهد؛ برای `!Unpin` باید `std::pin::pin!` یا `Box::pin` به‌کار رود.

```senpai-visual
{"kind":"async","labels":["executor","poll task","wake later"]}
```
