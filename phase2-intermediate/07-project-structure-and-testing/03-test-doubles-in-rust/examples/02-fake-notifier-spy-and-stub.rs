//! A hand-rolled test double: the same `Notifier` trait, a second
//! implementation that never sends anything real. `FakeNotifier` plays two
//! roles at once — it records every call (a spy), and `fail_with` lets a
//! test make it return a canned error on demand (a stub).
//!
//!     cargo run -p p2-07-03-test-doubles-in-rust --example 02-fake-notifier-spy-and-stub

trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

#[derive(Debug)]
struct NotifyError(String);

#[derive(Default)]
struct FakeNotifier {
    calls: Vec<(String, String)>,
    fail_with: Option<String>,
}

impl Notifier for FakeNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        if let Some(reason) = &self.fail_with {
            return Err(NotifyError(reason.clone()));
        }
        self.calls.push((to.to_string(), message.to_string()));
        Ok(())
    }
}

fn main() {
    // Used as a spy: nothing configured to fail, so every call succeeds
    // and is recorded for the test to inspect afterward.
    let mut spy = FakeNotifier::default();
    spy.notify("ren@example.com", "shipped").unwrap();
    spy.notify("aoi@example.com", "shipped").unwrap();
    println!("spy.calls: {:?}", spy.calls);

    // Used as a stub: configured up front to fail, so a test can exercise
    // an error path the real EmailNotifier has no easy way to trigger.
    let mut stub = FakeNotifier {
        fail_with: Some("simulated provider outage".to_string()),
        ..Default::default()
    };
    match stub.notify("ren@example.com", "shipped") {
        Ok(()) => println!("unexpected success"),
        Err(NotifyError(reason)) => println!("stub failed with: {reason}"),
    }
}
