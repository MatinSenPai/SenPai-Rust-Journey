//! DELIBERATELY BROKEN — expected: E0050
//!
//! `Notifier::notify` takes two `&str` arguments. `StaleFake` was written
//! against an older shape of the trait that only took one, and nobody
//! updated it when `message` was added. This is the exact drift a runtime
//! mock in a dynamic language cannot catch until the test actually runs —
//! here it never gets that far.
//!
//!     cargo run -p p2-07-03-test-doubles-in-rust --example 05-signature-drift-broken --features broken

trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

#[derive(Debug)]
struct NotifyError(String);

struct StaleFake;

impl Notifier for StaleFake {
    fn notify(&mut self, to: &str) -> Result<(), NotifyError> {
        println!("to: {to}");
        Ok(())
    }
}

fn main() {
    println!("compiled");
}
