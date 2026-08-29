//! Exercises for 2.8.5 — Futures and runtimes.
//!
//! `Countdown` is a toy `Future` that becomes `Ready` only after being
//! polled `total_polls` times — nothing here waits on real I/O or a timer,
//! it exists purely to make "a `Future` is a poll-able state machine"
//! concrete. `block_on` is the smallest possible executor: it drives any
//! `Future` to completion by polling it in a loop.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

/// A toy `Future` that returns `Pending` exactly `total_polls` times before
/// resolving to `Ready(total_polls)` — `total_polls + 1` calls to `poll` in
/// total.
pub struct Countdown {
    remaining: u32,
    total_polls: u32,
}

impl Countdown {
    pub fn new(total_polls: u32) -> Self {
        Countdown {
            remaining: total_polls,
            total_polls,
        }
    }
}

impl Future for Countdown {
    /// Resolves to the `total_polls` it was constructed with.
    type Output = u32;

    /// If `self.remaining` is `0`, resolve with `Poll::Ready(self.total_polls)`.
    /// Otherwise, subtract one from `self.remaining` and report
    /// `Poll::Pending`.
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!("if remaining is 0, resolve Ready with total_polls; otherwise decrement remaining and report Pending")
    }
}

/// The simplest possible executor: heap-pins `future` with `Box::pin` (so
/// it is safe to poll no matter whether `future` is `Unpin` — a real
/// `async fn`'s generated state machine never is), builds a `Context` from
/// a `Waker` that does nothing when woken (`Waker::noop()` — there is no
/// real I/O here to wake up from), then polls in a loop: return the value
/// the moment `poll` reports `Poll::Ready`, and poll again every time it
/// reports `Poll::Pending`.
pub fn block_on<F: Future>(future: F) -> F::Output {
    todo!(
        "heap-pin `future`, build a Context from a no-op Waker, then poll it in a loop until Ready"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn countdown_resolves_after_the_right_number_of_polls() {
        let result = block_on(Countdown::new(5));
        assert_eq!(result, 5);
    }

    #[test]
    fn countdown_of_zero_resolves_immediately() {
        let result = block_on(Countdown::new(0));
        assert_eq!(result, 0);
    }

    #[test]
    fn countdown_of_one() {
        let result = block_on(Countdown::new(1));
        assert_eq!(result, 1);
    }

    #[test]
    fn block_on_also_drives_a_real_async_fn() {
        async fn double(x: u32) -> u32 {
            x * 2
        }

        let result = block_on(double(21));
        assert_eq!(result, 42);
    }
}
