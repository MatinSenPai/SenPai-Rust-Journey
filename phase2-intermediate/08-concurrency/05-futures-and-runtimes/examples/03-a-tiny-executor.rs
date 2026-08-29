//! The smallest possible executor: `block_on` polls a `Future` in a loop
//! until it is `Ready`. `YieldOnce` is a hand-rolled `Future` — `Pending`
//! on its first poll, `Ready` on its second — awaited from inside a real
//! `async fn`, so you can watch a `Pending` from OUR OWN future bubble up
//! through `.await` and make the executor poll a second time.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};

struct YieldOnce {
    done: bool,
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<()> {
        if self.done {
            Poll::Ready(())
        } else {
            self.done = true;
            Poll::Pending
        }
    }
}

fn block_on<F: Future>(future: F) -> F::Output {
    let mut future = Box::pin(future);
    let waker = Waker::noop();
    let mut cx = Context::from_waker(waker);
    let mut attempt = 1;
    loop {
        println!("executor: poll attempt {attempt}");
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(value) => return value,
            Poll::Pending => attempt += 1,
        }
    }
}

async fn run() -> u32 {
    println!("  run: about to yield once");
    YieldOnce { done: false }.await;
    println!("  run: resumed after yielding");
    42
}

fn main() {
    let total = block_on(run());
    println!("total = {total}");
}
