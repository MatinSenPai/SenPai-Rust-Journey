//! Implementing `Future` by hand for the simplest useful case: a value
//! that is `Pending` on its first poll and `Ready` on its second. No
//! executor yet — `poll` is called directly, by hand, to see exactly
//! what it returns.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

struct FlipOnce {
    polled_before: bool,
}

impl Future for FlipOnce {
    type Output = &'static str;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.polled_before {
            Poll::Ready("ready on the second poll")
        } else {
            self.polled_before = true;
            Poll::Pending
        }
    }
}

fn main() {
    let mut task = FlipOnce {
        polled_before: false,
    };
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);

    let first = Pin::new(&mut task).poll(&mut cx);
    println!("first poll:  {first:?}");

    let second = Pin::new(&mut task).poll(&mut cx);
    println!("second poll: {second:?}");
}
