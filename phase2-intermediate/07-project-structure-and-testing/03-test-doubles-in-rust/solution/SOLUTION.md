# Solution — 2.7.3 Test doubles in Rust

```rust
impl<N: Notifier> ShippingService<N> {
    pub fn ship_order(&mut self, order_id: &str, customer_email: &str) -> Result<(), ShipError> {
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

impl Notifier for FakeNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        if let Some(reason) = &self.fail_with {
            return Err(NotifyError(reason.clone()));
        }
        self.calls.push((to.to_string(), message.to_string()));
        Ok(())
    }
}

pub fn notify_all(notifiers: &mut [Box<dyn Notifier>], to: &str, message: &str) -> usize {
    let mut succeeded = 0;
    for notifier in notifiers.iter_mut() {
        if notifier.notify(to, message).is_ok() {
            succeeded += 1;
        }
    }
    succeeded
}
```

## `ShippingService::ship_order` — the order of operations is what keeps idempotency correct

```rust
if self.shipped.contains(order_id) {
    return Err(ShipError::AlreadyShipped);
}
let message = format!("Your order {order_id} has shipped!");
self.notifier
    .notify(customer_email, &message)
    .map_err(ShipError::NotifyFailed)?;
self.shipped.insert(order_id.to_string());
```

Three lines, in exactly this order: first only *read* (`contains`), then call `notify`, and only insert `order_id` into `shipped` once that call has genuinely succeeded past the `?`. Flip the order — `insert` first, `notify` second — and a real notify failure would still record the order as "done," and no second attempt would ever happen. `ship_order_is_retried_after_a_previous_notify_failure` checks exactly this: calling `ship_order` twice in a row against an always-failing notifier must produce `NotifyFailed` **twice**, not `NotifyFailed` once followed by `AlreadyShipped`.

`.map_err(ShipError::NotifyFailed)` has a small trick of its own too: `ShipError::NotifyFailed` is an automatic constructor function — every tuple variant of an enum is also, for free, a function from its payload type to the enum (`NotifyError -> ShipError` here). Same `?` plus `.map_err(...)` combination 1.6.5 taught you, just with the enum variant itself as the constructor instead of a hand-written closure.

## `impl Notifier for FakeNotifier` — one `if let`, two roles

```rust
if let Some(reason) = &self.fail_with {
    return Err(NotifyError(reason.clone()));
}
self.calls.push((to.to_string(), message.to_string()));
Ok(())
```

This single condition decides whether `FakeNotifier` is acting as a stub or a spy this time — not two separate types, not an extra "test mode" flag. When `fail_with` holds nothing, the first branch is skipped and execution reaches the recording path; when it holds something, the function returns early and never touches `calls` at all. That exact ordering is what satisfies `fake_notifier_returns_canned_error_when_configured_to_fail`'s `assert!(fake.calls.is_empty())`.

## `notify_all` — the same call, behind a `dyn Notifier`

```rust
for notifier in notifiers.iter_mut() {
    if notifier.notify(to, message).is_ok() {
        succeeded += 1;
    }
}
```

The tempting shorter version was `.iter_mut().filter(|n| n.notify(...).is_ok()).count()` — the exact iterator-adapter shape 2.2 taught you. Try it, though, and it doesn't compile: `.filter()`'s closure always receives `&Item`, but here `Item` is already `&mut Box<dyn Notifier>` — that's what `.iter_mut()` itself yields — so the closure parameter is `n: &&mut Box<dyn Notifier>`, a shared reference sitting on top of the `&mut`. `.notify(...)` needs to reach a `&mut Box<dyn Notifier>`, and a `&` can't be reborrowed as `&mut` no matter how many layers you peel. A plain loop never runs into this at all, because it binds `notifier` directly to each `&mut Box<dyn Notifier>` that `notifiers.iter_mut()` yields, with no extra reference wrapped around it.

The real point is elsewhere: the loop's body is identical to what you'd write over a generic `N: Notifier` — the same `.notify(to, message)`. Only the type of `notifiers` changed: `&mut [Box<dyn Notifier>]` instead of one fixed `N`. That's exactly where dynamic dispatch charges its cost — one vtable hop per call — and exactly where it buys you something back: a `Vec` that genuinely holds an `EmailNotifier` and several `FakeNotifier`s side by side.

## What this lesson was really about

- **The substitution boundary was always the trait, never a separate layer.** Neither `ShippingService<N>` nor `notify_all` ever knew what `N` or a given `Box<dyn Notifier>` element actually was — only that it was a `Notifier`.
- **`FakeNotifier` has to follow the exact same compiler rule `EmailNotifier` follows.** That's exactly what "Why this replaces most of what a mocking framework does" claimed, and `E0050` in "Errors you will meet" proved it.
- **Ownership, not a testing convention, decides how you look at `FakeNotifier` after `ship_order` runs.** `ShippingService::new` takes ownership of the notifier, so `notifier()` — not the local variable you passed in — is the official way to reach it.
- **A generic bound and `dyn Trait` are two separate tools, not a matter of taste.** `ShippingService<N>` wanted one fixed type; `notify_all` needed a genuine mix. Each landed exactly where 2.3.7 said it would.
