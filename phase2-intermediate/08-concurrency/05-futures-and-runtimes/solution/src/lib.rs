use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

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
