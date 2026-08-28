//! DELIBERATELY BROKEN — expected: E0382
//!
//! In Python, a `Mock()` you pass into a function is still the same object
//! you kept a name for — asserting on it afterward just works. Rust has no
//! such shortcut: `ShippingService::new(fake)` takes ownership of `fake`,
//! so the local name is gone. `ship_order` still works fine; this example
//! is the tempting next line that doesn't.
//!
//!     cargo run -p p2-07-03-test-doubles-in-rust --example 07-use-after-move-broken --features broken

use std::collections::HashSet;

trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

#[derive(Debug)]
struct NotifyError(String);

#[derive(Default, Debug)]
struct FakeNotifier {
    calls: Vec<(String, String)>,
}

impl Notifier for FakeNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        self.calls.push((to.to_string(), message.to_string()));
        Ok(())
    }
}

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

    fn ship_order(&mut self, order_id: &str, customer_email: &str) -> Result<(), NotifyError> {
        let message = format!("Your order {order_id} has shipped!");
        self.notifier.notify(customer_email, &message)?;
        self.shipped.insert(order_id.to_string());
        Ok(())
    }
}

fn main() {
    let fake = FakeNotifier::default();
    let mut service = ShippingService::new(fake);
    service.ship_order("A1", "ren@example.com").unwrap();

    // `fake` was moved into `service` above — there is no separate handle
    // left to inspect.
    println!("calls recorded: {:?}", fake.calls);
}
