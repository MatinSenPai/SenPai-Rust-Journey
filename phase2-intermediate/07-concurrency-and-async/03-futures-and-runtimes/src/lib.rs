use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

/// A toy `Future` that becomes `Ready` once it's been polled `total_polls`
/// times. Nothing here actually waits on real I/O or a timer — this
/// exists purely to make "a Future is a poll-able state machine" concrete.
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
    /// Resolves to how many times it was polled in total.
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!(
            "if self.remaining == 0, return Poll::Ready(self.total_polls); \
             otherwise self.remaining -= 1 and return Poll::Pending"
        )
    }
}

/// The simplest possible executor: polls `future` in a loop until it's
/// `Ready`, ignoring the `Waker` entirely (see the README for why real
/// executors can't get away with that at scale, but this one can — for a
/// `Future` that's always immediately ready to be polled again, like
/// `Countdown`, busy-polling is wasteful but not incorrect).
pub fn block_on<F: Future + Unpin>(mut future: F) -> F::Output {
    todo!(
        "let waker = Waker::noop(); let mut cx = Context::from_waker(waker); \
         loop {{ match Pin::new(&mut future).poll(&mut cx) {{ Poll::Ready(v) => return v, Poll::Pending => continue }} }}"
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
}
