//! Injection through a trait object instead of a generic bound — the
//! other half of 2.3.7's choice, applied to this lesson's dependency.
//! `notify_all` needs a genuine mix of concrete `Notifier` types in one
//! collection, which no `Vec<N>` with a single fixed `N` could ever hold.
//!
//!     cargo run -p p2-07-03-test-doubles-in-rust --example 04-trait-object-injection

trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

#[derive(Debug)]
struct NotifyError(String);

struct EmailNotifier;

impl Notifier for EmailNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        println!("email to {to}: {message}");
        Ok(())
    }
}

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

/// Calls `.notify(to, message)` on every notifier in the slice, in order,
/// even after one fails. Returns how many of them returned `Ok`.
fn notify_all(notifiers: &mut [Box<dyn Notifier>], to: &str, message: &str) -> usize {
    let mut succeeded = 0;
    for notifier in notifiers.iter_mut() {
        if notifier.notify(to, message).is_ok() {
            succeeded += 1;
        }
    }
    succeeded
}

fn main() {
    let mut always_fails = FakeNotifier {
        fail_with: Some("simulated outage".to_string()),
        ..Default::default()
    };
    match always_fails.notify("ren@example.com", "shipped") {
        Ok(()) => println!("unexpected success"),
        Err(NotifyError(reason)) => println!("standalone failure: {reason}"),
    }

    let mut fleet: Vec<Box<dyn Notifier>> = vec![
        Box::new(EmailNotifier),
        Box::new(FakeNotifier::default()),
        Box::new(always_fails),
    ];

    let succeeded = notify_all(&mut fleet, "ren@example.com", "shipped");
    println!("succeeded: {succeeded} of {}", fleet.len());
}
