//! DELIBERATELY BROKEN — expected: E0277
//!
//! `ShippingService::new` accepts any `N: Notifier`. `SilentLogger` never
//! implemented `Notifier` at all — an easy slip when it superficially looks
//! like it could be one. There is no `impl`, so there is no substitution.
//!
//!     cargo run -p p2-07-03-test-doubles-in-rust --example 06-missing-trait-impl-broken --features broken

use std::collections::HashSet;

trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

#[derive(Debug)]
struct NotifyError(String);

struct ShippingService<N> {
    notifier: N,
    shipped: HashSet<String>,
}

impl<N: Notifier> ShippingService<N> {
    fn new(notifier: N) -> Self {
        Self {
            notifier,
            shipped: HashSet::new(),
        }
    }
}

struct SilentLogger;

fn main() {
    let _service = ShippingService::new(SilentLogger);
    println!("compiled");
}
