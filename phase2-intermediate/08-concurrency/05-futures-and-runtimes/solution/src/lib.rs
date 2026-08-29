//! Exercises for 2.8.5 — Futures and runtimes.

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
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.remaining == 0 {
            Poll::Ready(self.total_polls)
        } else {
            self.remaining -= 1;
            Poll::Pending
        }
    }
}

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
