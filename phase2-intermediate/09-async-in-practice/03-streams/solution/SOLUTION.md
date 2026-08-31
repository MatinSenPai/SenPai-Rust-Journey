# Solution

```rust
pub async fn sum_stream(values: Vec<i32>) -> i32 {
    let mut stream = tokio_stream::iter(values);
    let mut total = 0;
    while let Some(value) = stream.next().await {
        total += value;
    }
    total
}

pub async fn first_n_even(values: Vec<i32>, limit: usize) -> Vec<i32> {
    let mut stream = tokio_stream::iter(values)
        .filter(|n| n % 2 == 0)
        .take(limit);

    let mut kept = Vec::new();
    while let Some(value) = stream.next().await {
        kept.push(value);
    }
    kept
}
```

Both functions follow exactly the idiom "The concept" showed several times: `tokio_stream::iter` turns a plain `Vec` into a `Stream`, and a `while let Some(...) = ... .next().await` loop is what actually drives it. The important detail in `first_n_even` is the chain order: `.filter()` comes before `.take()` — the other way around, `.take(limit)` would apply to all of `values` (not just the even ones) before filtering, which could return fewer than `limit` genuinely-even numbers even when enough existed in `values`. Because both adapters are lazy, this ordering costs nothing extra either — the stream pulls exactly as many elements out of `values` as it needs, never more.

## Build: values over real time

```rust
pub async fn ticks_after(delays_ms: Vec<u64>) -> Vec<u64> {
    let stream = tokio_stream::iter(delays_ms).then(|delay_ms| async move {
        tokio::time::sleep(Duration::from_millis(delay_ms)).await;
        delay_ms
    });
    tokio::pin!(stream);

    let mut ticks = Vec::new();
    while let Some(delay_ms) = stream.next().await {
        ticks.push(delay_ms);
    }
    ticks
}
```

The exact same pattern as `03-time-spaced-stream` in "The concept" — just keeping each value in a `Vec` instead of printing it. `ticks_after_actually_takes_real_time` proves with numbers exactly what "The concept" showed you by eye: five 20ms delays together take at least 90ms — not something near zero, meaning the sleeps genuinely happened rather than being faked instantly.

## Challenge: a hand-rolled `Stream`

```rust
pub struct CountdownStream {
    remaining: u32,
}

impl Stream for CountdownStream {
    type Item = u32;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<u32>> {
        if self.remaining == 0 {
            Poll::Ready(None)
        } else {
            self.remaining -= 1;
            Poll::Ready(Some(self.remaining))
        }
    }
}
```

No `tokio_stream::iter`, no adapters — this is exactly the `poll_next` shape "The concept" showed you, this time written by hand. Because `CountdownStream` holds only a `u32` and no internal references, it's automatically `Unpin` — the same reason [2.8.5](../../08-concurrency/05-futures-and-runtimes/README.md) gave for `FlipOnce`; that's why `stream.next().await` works on this type with no `Box::pin` or `tokio::pin!` at all. `CountdownStream::new(3)` yields `Some(2)`, `Some(1)`, `Some(0)` in turn, then `None` — because every `poll_next` checks whether `remaining` is zero *before* decrementing it.
