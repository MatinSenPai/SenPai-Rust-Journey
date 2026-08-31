//! Constructor injection through a generic bound. `ShippingService<N>`
//! never names `EmailNotifier` or `FakeNotifier` anywhere in its own body —
//! it only knows `N: Notifier`. Production wires up the real notifier; a
//! test wires up the fake. Same struct, same method, no branching on
//! "am I in a test."
//!
//!     cargo run -p p2-07-03-test-doubles-in-rust --example 03-generic-injection

use std::collections::HashSet;

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
}

impl Notifier for FakeNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        self.calls.push((to.to_string(), message.to_string()));
        Ok(())
    }
}

#[derive(Debug)]
enum ShipError {
    AlreadyShipped,
    NotifyFailed(NotifyError),
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

    fn ship_order(&mut self, order_id: &str, customer_email: &str) -> Result<(), ShipError> {
        if self.shipped.contains(order_id) {
            return Err(ShipError::AlreadyShipped);
        }
        let message = format!("Your order {order_id} has shipped!");
        self.notifier
            .notify(customer_email, &message)
            .map_err(ShipError::NotifyFailed)?;
        self.shipped.insert(order_id.to_string());
        Ok(())
    }
}

fn main() {
    // Production: `N = EmailNotifier`.
    let mut live = ShippingService::new(EmailNotifier);
    match live.ship_order("A1", "ren@example.com") {
        Ok(()) => println!("live.ship_order: sent"),
        Err(_) => println!("live.ship_order: failed"),
    }

    // Test: `N = FakeNotifier`. Same `ShippingService` code, same
    // `ship_order` method — only the type parameter changed.
    let mut test_service = ShippingService::new(FakeNotifier::default());
    test_service.ship_order("A1", "ren@example.com").unwrap();
    match test_service.ship_order("A1", "ren@example.com") {
        Ok(()) => println!("unexpected: shipped A1 twice"),
        Err(ShipError::AlreadyShipped) => println!("second ship_order: already shipped"),
        Err(ShipError::NotifyFailed(NotifyError(reason))) => {
            println!("second ship_order: notify failed: {reason}")
        }
    }
    println!("notifier.calls: {:?}", test_service.notifier.calls);
}
