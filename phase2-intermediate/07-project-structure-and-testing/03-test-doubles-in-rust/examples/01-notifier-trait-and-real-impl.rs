//! The dependency boundary: a trait, and the implementation production
//! code actually uses. Nothing about `EmailNotifier` is special yet — this
//! is just an ordinary trait and an ordinary `impl`, the same shape 2.3.1
//! taught.
//!
//!     cargo run -p p2-07-03-test-doubles-in-rust --example 01-notifier-trait-and-real-impl

trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

#[derive(Debug)]
struct NotifyError(String);

struct EmailNotifier;

impl Notifier for EmailNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        // Real code would call an email provider's API here. This lesson
        // only needs something observable, so it prints instead.
        println!("email to {to}: {message}");
        Ok(())
    }
}

fn main() {
    let mut notifier = EmailNotifier;
    match notifier.notify("ren@example.com", "your order has shipped") {
        Ok(()) => println!("sent"),
        Err(NotifyError(reason)) => println!("failed: {reason}"),
    }
}
