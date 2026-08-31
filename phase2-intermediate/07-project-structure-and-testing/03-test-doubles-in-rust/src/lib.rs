//! Exercises for 2.7.3 — test doubles in Rust.
//!
//! `Notifier`, `NotifyError`, `EmailNotifier`, and `ShipError` are provided
//! below, fully written — none of that is this lesson's subject. What you
//! write is: a hand-rolled test double for `Notifier` (inside
//! `#[cfg(test)]`, where it belongs), the one method on `ShippingService`
//! that actually uses the injected dependency, and a small function that
//! injects the same trait through a trait object instead of a generic
//! bound.

use std::collections::HashSet;
use std::fmt;

/// The dependency boundary: anything that can deliver a notification.
pub trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

/// An error from a `Notifier`, carrying a human-readable reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotifyError(pub String);

impl fmt::Display for NotifyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The production `Notifier`. Real code would call an email provider's
/// API here; this lesson only needs something observable, so it prints.
pub struct EmailNotifier;

impl Notifier for EmailNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        println!("email to {to}: {message}");
        Ok(())
    }
}

/// Why `ShippingService::ship_order` failed.
#[derive(Debug, PartialEq, Eq)]
pub enum ShipError {
    AlreadyShipped,
    NotifyFailed(NotifyError),
}

impl fmt::Display for ShipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ShipError::AlreadyShipped => write!(f, "already shipped"),
            ShipError::NotifyFailed(err) => write!(f, "notify failed: {err}"),
        }
    }
}

/// A tiny order-shipping service, generic over its `Notifier`. Production
/// wires up `ShippingService<EmailNotifier>`; a test wires up
/// `ShippingService<FakeNotifier>` — same struct, same methods.
pub struct ShippingService<N> {
    notifier: N,
    shipped: HashSet<String>,
}

impl<N: Notifier> ShippingService<N> {
    pub fn new(notifier: N) -> Self {
        Self {
            notifier,
            shipped: HashSet::new(),
        }
    }

    /// A reference to the injected notifier — how a test reaches back in
    /// to inspect a fake after `ship_order` has already moved it in.
    pub fn notifier(&self) -> &N {
        &self.notifier
    }

    /// Ships `order_id` to `customer_email`, unless `order_id` was already
    /// shipped successfully by this service.
    ///
    /// - If `order_id` is already recorded as shipped, returns
    ///   `Err(ShipError::AlreadyShipped)` and never calls the notifier.
    /// - Otherwise calls `self.notifier.notify(customer_email, &message)`,
    ///   where `message` is `format!("Your order {order_id} has shipped!")`.
    ///   - If that call returns `Err(e)`, `ship_order` returns
    ///     `Err(ShipError::NotifyFailed(e))` and does **not** record
    ///     `order_id` as shipped — a retry will call the notifier again.
    ///   - If it returns `Ok(())`, `ship_order` records `order_id` as
    ///     shipped and returns `Ok(())`.
    pub fn ship_order(&mut self, order_id: &str, customer_email: &str) -> Result<(), ShipError> {
        todo!(
            "check self.shipped for order_id, call self.notifier.notify with the exact \
             message format above, then update self.shipped only on success — see the \
             doc comment for the exact rules"
        )
    }
}

/// Calls `.notify(to, message)` on every notifier in `notifiers`, in
/// order, even after one of them fails. Returns how many returned `Ok`.
pub fn notify_all(notifiers: &mut [Box<dyn Notifier>], to: &str, message: &str) -> usize {
    todo!("call notify on every notifier in order, and count + return how many returned Ok")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A hand-rolled test double for `Notifier`. In a real crate this is
    /// exactly the kind of type that lives only here, inside
    /// `#[cfg(test)]`: it never compiles into the release binary and never
    /// needs to be `pub`.
    #[derive(Default)]
    struct FakeNotifier {
        calls: Vec<(String, String)>,
        fail_with: Option<String>,
    }

    impl Notifier for FakeNotifier {
        /// When `self.fail_with` is `None`, appends `(to.to_string(),
        /// message.to_string())` to `self.calls` and returns `Ok(())`.
        /// When `self.fail_with` is `Some(reason)`, returns
        /// `Err(NotifyError(reason.clone()))` instead, and does not touch
        /// `self.calls`.
        fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
            todo!(
                "check self.fail_with first and return its Err if set; otherwise push \
                 (to, message) onto self.calls and return Ok(())"
            )
        }
    }

    #[test]
    fn fake_notifier_records_calls_when_not_configured_to_fail() {
        let mut fake = FakeNotifier::default();
        fake.notify("a@example.com", "hi").unwrap();
        fake.notify("b@example.com", "bye").unwrap();
        assert_eq!(
            fake.calls,
            vec![
                ("a@example.com".to_string(), "hi".to_string()),
                ("b@example.com".to_string(), "bye".to_string()),
            ]
        );
    }

    #[test]
    fn fake_notifier_returns_canned_error_when_configured_to_fail() {
        let mut fake = FakeNotifier {
            fail_with: Some("outage".to_string()),
            ..Default::default()
        };
        let result = fake.notify("a@example.com", "hi");
        assert_eq!(result, Err(NotifyError("outage".to_string())));
        assert!(fake.calls.is_empty());
    }

    #[test]
    fn ship_order_calls_notifier_with_expected_message() {
        let mut service = ShippingService::new(FakeNotifier::default());
        service.ship_order("A1", "a@example.com").unwrap();
        assert_eq!(
            service.notifier().calls,
            vec![(
                "a@example.com".to_string(),
                "Your order A1 has shipped!".to_string()
            )]
        );
    }

    #[test]
    fn ship_order_twice_for_the_same_id_is_rejected_without_a_second_notify() {
        let mut service = ShippingService::new(FakeNotifier::default());
        service.ship_order("A1", "a@example.com").unwrap();
        let second = service.ship_order("A1", "a@example.com");
        assert_eq!(second, Err(ShipError::AlreadyShipped));
        assert_eq!(service.notifier().calls.len(), 1);
    }

    #[test]
    fn ship_order_propagates_a_notify_failure() {
        let fake = FakeNotifier {
            fail_with: Some("outage".to_string()),
            ..Default::default()
        };
        let mut service = ShippingService::new(fake);
        let result = service.ship_order("A1", "a@example.com");
        assert_eq!(
            result,
            Err(ShipError::NotifyFailed(NotifyError("outage".to_string())))
        );
    }

    #[test]
    fn ship_order_is_retried_after_a_previous_notify_failure() {
        let fake = FakeNotifier {
            fail_with: Some("outage".to_string()),
            ..Default::default()
        };
        let mut service = ShippingService::new(fake);
        let first = service.ship_order("A1", "a@example.com");
        let second = service.ship_order("A1", "a@example.com");
        // If a failed attempt had wrongly been recorded as shipped, the
        // second call would answer `AlreadyShipped` instead of trying
        // (and failing) again.
        assert_eq!(
            first,
            Err(ShipError::NotifyFailed(NotifyError("outage".to_string())))
        );
        assert_eq!(
            second,
            Err(ShipError::NotifyFailed(NotifyError("outage".to_string())))
        );
    }

    #[test]
    fn notify_all_counts_successes_across_a_heterogeneous_list() {
        let mut notifiers: Vec<Box<dyn Notifier>> = vec![
            Box::new(EmailNotifier),
            Box::new(FakeNotifier::default()),
            Box::new(FakeNotifier {
                fail_with: Some("outage".to_string()),
                ..Default::default()
            }),
        ];
        let succeeded = notify_all(&mut notifiers, "a@example.com", "hi");
        assert_eq!(succeeded, 2);
    }

    #[test]
    fn notify_all_continues_past_an_early_failure() {
        let mut notifiers: Vec<Box<dyn Notifier>> = vec![
            Box::new(FakeNotifier {
                fail_with: Some("outage".to_string()),
                ..Default::default()
            }),
            Box::new(FakeNotifier::default()),
            Box::new(FakeNotifier::default()),
        ];
        let succeeded = notify_all(&mut notifiers, "a@example.com", "hi");
        assert_eq!(succeeded, 2);
    }
}
