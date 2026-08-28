# 2.7.3 — Test doubles in Rust, and why you rarely need a mocking framework

## At a glance

After this lesson you can:

- Tell apart the four common shapes of test double — stub, fake, spy, mock — and use the right name for each.
- Write a trait for a real dependency (a `Notifier`, a clock, a data store), give it a real implementation and a hand-rolled fake, and choose between a generic bound and a trait object to inject either one.
- Explain why the compiler itself rejects a fake that has drifted from the trait's real signature — with no extra framework involved — and where this pattern stops being enough.

**Time:** ~55 minutes · **Prerequisites:**
[2.7.2 — Unit, integration, and doc tests](../02-unit-integration-doc-tests/README.md),
[2.3.7 — Static versus dynamic dispatch, and object safety](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md)

---

## Why this matters

2.7.2 taught you three kinds of test — unit, integration, doc — and all three were exercised against one perfectly pure function: `celsius_to_fahrenheit` never opened a file, never touched the network, never looked at the system clock. Testing it, no matter how many times you called it, was always the same story.

Real code isn't like that. An order-shipping service has to notify a customer; a handler has to read from a database; a job has to check a real timeout. If your tests are forced to go through those real dependencies, they either become slow and brittle (a real email, a real database connection), or you simply can't exercise the failure paths at all — how do you make a real email provider fail *right now, on demand*?

In Python or JavaScript, the common answer is a mocking framework: something that replaces a method's behavior at run time, on a real object. Rust has a different answer — one that comes from the same traits and the same static-versus-dynamic choice 2.3.7 already taught you, not from a separate library. This lesson shows you that answer: when it's enough, and where it genuinely runs out.

---

## The concept

### Four shapes of test double: stub, fake, spy, mock

When the code you're testing depends on something slow, unpredictable, or outside your control, you swap in a smaller, controllable stand-in instead of the real thing. The umbrella term for that stand-in is a **test double**, and underneath it are four shapes worth telling apart:

- **Stub** — returns a fixed, canned answer. No logic, nothing recorded.
- **Fake** — has real, working behavior, just simplified — an in-memory store standing in for a real database.
- **Spy** — remembers what was called, with what arguments, and how many times, so the test can assert on it afterward.
- **Mock** — pre-loaded with expectations — which calls, how many, in what order — and fails the test itself if those expectations aren't met.

The line between these four isn't always sharp — most hand-rolled doubles you'll actually write play more than one of these roles at once (you'll build exactly one of those today). What matters isn't pinning the single most precise label on a given double; it's knowing exactly what a colleague means when they say "I mocked the `Notifier` for this test."

### The `Notifier` trait: the boundary you swap out in tests

To make this concrete, one running example carries the whole lesson: an order-shipping service that has to notify a customer once an order ships. That "notifying" is exactly the part you don't want real in a test — a real email is neither fast, nor free, nor something you can reliably force to fail on demand.

Rust's answer isn't to bring in a framework that rewrites a function's behavior at run time — exactly as 2.3.7 showed you, the boundary between "which implementation" and "what it does" is a trait:

```rust
trait Notifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
}

struct EmailNotifier;

impl Notifier for EmailNotifier {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        println!("email to {to}: {message}");
        Ok(())
    }
}
```

Nothing new here — exactly the trait and `impl` shape 2.3.1 gave you. `EmailNotifier` is what production actually runs; instead of a real API call, it just prints, so its behavior stays observable.

```rust
let mut notifier = EmailNotifier;
match notifier.notify("ren@example.com", "your order has shipped") {
    Ok(()) => println!("sent"),
    Err(NotifyError(reason)) => println!("failed: {reason}"),
}
```

```text
email to ren@example.com: your order has shipped
sent
```

### `FakeNotifier`: a second, hand-rolled implementation, test-only

`Notifier` is just a trait — any type with this one method is a valid `Notifier`. So write a second type, by hand, test-only, that never sends a real email:

```rust
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
```

This one type plays both roles above. When `fail_with` is nothing, `FakeNotifier` is a **spy** — it records every call, exactly as it arrived, in `calls`:

```rust
let mut spy = FakeNotifier::default();
spy.notify("ren@example.com", "shipped").unwrap();
spy.notify("aoi@example.com", "shipped").unwrap();
println!("spy.calls: {:?}", spy.calls);
```

```text
spy.calls: [("ren@example.com", "shipped"), ("aoi@example.com", "shipped")]
```

And when `fail_with` holds a value, the same type becomes a **stub** — it returns a pre-decided failure, exactly the kind of thing that's hard or impossible to trigger on demand with a real `EmailNotifier`:

```rust
let mut stub = FakeNotifier {
    fail_with: Some("simulated provider outage".to_string()),
    ..Default::default()
};
match stub.notify("ren@example.com", "shipped") {
    Ok(()) => println!("unexpected success"),
    Err(NotifyError(reason)) => println!("stub failed with: {reason}"),
}
```

```text
stub failed with: simulated provider outage
```

One organizational note: `FakeNotifier` here is an ordinary, unrestricted type, so these examples can run it directly and show you its output. In a real crate, where only that crate's own tests ever construct it, its natural home is entirely inside the same `#[cfg(test)] mod tests { ... }` block your assertions already live in — exactly where today's exercise puts it. It then never compiles into the release binary at all, and never needs to be `pub`.

### Injection through a generic bound: one `ShippingService`, two `Notifier`s

Now build something that actually uses `Notifier` — an order-shipping service that notifies the customer when an order ships, and refuses to ship the same order twice:

```rust
enum ShipError {
    AlreadyShipped,
    NotifyFailed(NotifyError),
}

struct ShippingService<N> {
    notifier: N,
    shipped: HashSet<String>,
}
```

`ShippingService` is generic over its notifier's type — that one detail is the whole lesson. Now its methods:

```rust
impl<N: Notifier> ShippingService<N> {
    fn new(notifier: N) -> Self {
        Self {
            notifier,
            shipped: HashSet::new(),
        }
    }
}
```

```rust
impl<N: Notifier> ShippingService<N> {
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
```

Neither `ShippingService` nor `ship_order` ever names `EmailNotifier` or `FakeNotifier` anywhere in its own body — they only know `N: Notifier`. **Injection** happens right here: not in a config file, not in a DI framework, but at the exact moment you call `::new(...)` and decide which concrete type to hand over.

```rust
let mut live = ShippingService::new(EmailNotifier);
match live.ship_order("A1", "ren@example.com") {
    Ok(()) => println!("live.ship_order: sent"),
    Err(_) => println!("live.ship_order: failed"),
}
```

```text
email to ren@example.com: Your order A1 has shipped!
live.ship_order: sent
```

Same `ShippingService`, same `ship_order` — this time with `N = FakeNotifier`:

```rust
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
```

```text
second ship_order: already shipped
notifier.calls: [("ren@example.com", "Your order A1 has shipped!")]
```

The second `ship_order` came back with `AlreadyShipped` before it ever reached `notifier` — and `notifier.calls` proves exactly that: only one call was ever recorded, not two.

```senpai-visual
{"kind":"concept","labels":["ShippingService::new(notifier)","production: N = EmailNotifier","test: N = FakeNotifier","ship_order() — one method, two instantiations","the compiler picks the impl, not a runtime flag"]}
```

### Injection through a trait object: when you need heterogeneity

The generic bound above is enough when one `ShippingService` has exactly one fixed notifier type. But sometimes you need several *different* `Notifier`s in one collection at once — exactly the heterogeneity problem 2.3.7 showed you with `Vec<Box<dyn Summarize>>`. The same move works here too:

```rust
fn notify_all(notifiers: &mut [Box<dyn Notifier>], to: &str, message: &str) -> usize {
    let mut succeeded = 0;
    for notifier in notifiers.iter_mut() {
        if notifier.notify(to, message).is_ok() {
            succeeded += 1;
        }
    }
    succeeded
}
```

```rust
let mut always_fails = FakeNotifier {
    fail_with: Some("simulated outage".to_string()),
    ..Default::default()
};
match always_fails.notify("ren@example.com", "shipped") {
    Ok(()) => println!("unexpected success"),
    Err(NotifyError(reason)) => println!("standalone failure: {reason}"),
}
```

```text
standalone failure: simulated outage
```

Now put that same `always_fails` alongside a real `EmailNotifier` and a healthy `FakeNotifier`, in one `Vec`:

```rust
let mut fleet: Vec<Box<dyn Notifier>> = vec![
    Box::new(EmailNotifier),
    Box::new(FakeNotifier::default()),
    Box::new(always_fails),
];

let succeeded = notify_all(&mut fleet, "ren@example.com", "shipped");
println!("succeeded: {succeeded} of {}", fleet.len());
```

```text
email to ren@example.com: shipped
succeeded: 2 of 3
```

One `Vec`, three genuinely different types underneath — something no generic `ShippingService<N>` with a single fixed `N` could ever hold. The rule is the same one 2.3.7 taught: default to a generic bound; reach for `dyn Trait` when you genuinely need a mix of types in one variable or collection.

### Why this replaces most of what a mocking framework does

Nothing in the code above was a separate library, a decorator, or a "now patch this method" step. Two plain things happened:

1. **Which one runs was decided by writing ordinary code.** `ShippingService::new(EmailNotifier)` versus `ShippingService::new(FakeNotifier::default())` — that's it. No environment variable, no global registry, nothing hidden.
2. **The compiler, not a runtime interception layer, decides which `notify` actually gets called.** `FakeNotifier` has to implement the exact same trait `EmailNotifier` implements — same signature, same argument count, same return type.

That second point gives you something a runtime mock in a dynamic language cannot promise: if `Notifier::notify`'s signature ever changes — a parameter gets added, a return type changes — and you don't update `FakeNotifier`, your code simply does not compile. Not an under-covered test that never runs that path, not a surprise failure in production six months later — right now, at `cargo build`. You'll see the real proof of this in "Errors you will meet."

### An honest note on the limits

This pattern works great for a handful of swappable dependencies — a `Notifier`, maybe a clock, maybe a data store. It is not a replacement for *everything* a mocking framework does in other languages:

- **Verifying exact call order or count across several unrelated dependencies gets manual.** `FakeNotifier.calls` easily answers "I was called exactly once, with these exact arguments" — exactly what today's exercises check with `assert_eq!` on that same `Vec`. But "the notifier must be called *after* the order is saved to the database" is an ordering claim across two separate doubles, and Rust hands you no ready-made tool for that — you'd have to build a shared event log yourself. Crates like `mockall` exist for exactly this heavier need; this lesson doesn't go there, because most code you write never needs that level.
- **For something simple enough to actually run for real, a genuine integration test (2.7.2) often beats an elaborate fake.** A fake is always a simplified model of reality, and it can drift from what the real thing actually does; a local database or an in-memory queue you genuinely run sidesteps that risk entirely.

And two small pointers to where this keeps going: if a double ever has to be shared across threads, that's where `Mutex`/`Arc` comes in. And if the real `Notifier` genuinely has to be `async` (a real network call), an `async` method inside a trait raises exactly the same object-safety wall 2.3.7 named.

---

## Hands on

```sh
cargo run -p p2-07-03-test-doubles-in-rust --example 01-notifier-trait-and-real-impl
cargo run -p p2-07-03-test-doubles-in-rust --example 02-fake-notifier-spy-and-stub
cargo run -p p2-07-03-test-doubles-in-rust --example 03-generic-injection
cargo run -p p2-07-03-test-doubles-in-rust --example 04-trait-object-injection
```

Then the three broken ones:

```sh
cargo run -p p2-07-03-test-doubles-in-rust --example 05-signature-drift-broken --features broken
cargo run -p p2-07-03-test-doubles-in-rust --example 06-missing-trait-impl-broken --features broken
cargo run -p p2-07-03-test-doubles-in-rust --example 07-use-after-move-broken --features broken
```

Then try these:

1. In `02-fake-notifier-spy-and-stub.rs`, call `notify` a third time on `spy`. How many entries does `calls` have?
2. In `03-generic-injection.rs`, call `ship_order` for a different `order_id` (say, `"A2"`) too. What does `notifier.calls` show now?
3. In `04-trait-object-injection.rs`, add another healthy `FakeNotifier` to `fleet`. What is `succeeded` now?

---

## Errors you will meet

### `E0050` — a fake left behind by the trait's real signature

```text
error[E0050]: method `notify` has 2 parameters but the declaration in trait `Notifier::notify` has 3
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\05-signature-drift-broken.rs:21:15
   |
12 |     fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError>;
   |               ---------------------------------- trait requires 3 parameters
...
21 |     fn notify(&mut self, to: &str) -> Result<(), NotifyError> {
   |               ^^^^^^^^^^^^^^^^^^^ expected 3 parameters, found 2

For more information about this error, try `rustc --explain E0050`.
```

**What the compiler is objecting to:** `Notifier::notify` needs three parameters (`self`, `to`, `message`). `StaleFake` has an older signature — as if it were written before `message` was added to the trait, and nobody updated it.

**The fix:** add the missing parameter so the signature matches the trait exactly:

```rust
impl Notifier for StaleFake {
    fn notify(&mut self, to: &str, message: &str) -> Result<(), NotifyError> {
        println!("to: {to}, message: {message}");
        Ok(())
    }
}
```

**Why this is the fix:** this is exactly what "Why this replaces most of what a mocking framework does" claimed: `StaleFake` is a real implementation of the trait, not a freestanding runtime object — the compiler applies the exact same rule to it that it applied to `EmailNotifier`. If this fake were a Python mock instead, this drift would stay hidden until the first test that happened to call this exact method with these exact arguments.

### `E0277` — injecting a type that isn't a `Notifier` at all

```text
error[E0277]: the trait bound `SilentLogger: Notifier` is not satisfied
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\06-missing-trait-impl-broken.rs:35:41
   |
35 |     let _service = ShippingService::new(SilentLogger);
   |                    -------------------- ^^^^^^^^^^^^ unsatisfied trait bound
   |                    |
   |                    required by a bound introduced by this call
   |
help: the trait `Notifier` is not implemented for `SilentLogger`
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\06-missing-trait-impl-broken.rs:32:1
   |
32 | struct SilentLogger;
   | ^^^^^^^^^^^^^^^^^^^
help: this trait has no implementations, consider adding one
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\06-missing-trait-impl-broken.rs:11:1
   |
11 | trait Notifier {
   | ^^^^^^^^^^^^^^
note: required by a bound in `ShippingService::<N>::new`
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\06-missing-trait-impl-broken.rs:23:9
   |
23 | impl<N: Notifier> ShippingService<N> {
   |         ^^^^^^^^ required by this bound in `ShippingService::<N>::new`
24 |     fn new(notifier: N) -> Self {
   |        --- required by a bound in this associated function

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** `ShippingService::new` accepts any `N: Notifier`. `SilentLogger` never got an `impl Notifier for SilentLogger` — maybe because its shape merely looked like a notifier, but nobody actually wrote the `impl`.

**The fix:** either write `impl Notifier for SilentLogger`, or hand `ShippingService::new` a type that genuinely implements `Notifier` — `EmailNotifier` or `FakeNotifier`.

**Why this is the fix:** no substitution happens here at all, because none makes sense to the compiler — `SilentLogger` is exactly as unrelated to `Notifier`, type-wise, as a `bool` is to a `String`. Unlike a runtime mock, which says nothing until the exact moment a missing method is actually called, not a single line of code here even ran.

### `E0382` — using a fake after its ownership is gone

```text
error[E0382]: borrow of moved value: `fake`
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\07-use-after-move-broken.rs:60:38
   |
54 |     let fake = FakeNotifier::default();
   |         ---- move occurs because `fake` has type `FakeNotifier`, which does not implement the `Copy` trait
55 |     let mut service = ShippingService::new(fake);
   |                                            ---- value moved here
...
60 |     println!("calls recorded: {:?}", fake.calls);
   |                                      ^^^^^^^^^^ value borrowed here after move
   |
note: consider changing this parameter type in method `new` to borrow instead if owning the value isn't necessary
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\07-use-after-move-broken.rs:38:22
   |
38 |     fn new(notifier: N) -> Self {
   |        ---           ^ this parameter takes ownership of the value
   |        |
   |        in this method
note: if `FakeNotifier` implemented `Clone`, you could clone the value
  --> phase2-intermediate\07-project-structure-and-testing\03-test-doubles-in-rust\examples\07-use-after-move-broken.rs:21:1
   |
21 | struct FakeNotifier {
   | ^^^^^^^^^^^^^^^^^^^ consider implementing `Clone` for this type
...
55 |     let mut service = ShippingService::new(fake);
   |                                            ---- you could clone this value

For more information about this error, try `rustc --explain E0382`.
```

**What the compiler is objecting to:** `ShippingService::new(fake)` took ownership of `fake`. The local variable `fake` is no longer valid after that line — exactly the same move rule Phase 1.2 taught you, with no exception here for "but this is being used for testing."

**The fix:** instead of holding onto `fake`, use the way `ShippingService` itself provides for this — the `notifier()` method:

```rust
println!("calls recorded: {:?}", service.notifier().calls);
```

**Why this is the fix:** in Python, a `Mock()` you pass into a function is still the exact same object you kept a name for — checking it afterward is free, because everything works by reference. Rust has no such shortcut: `ShippingService::new` takes ownership of `notifier`, so your local name is gone. `notifier()` exists for exactly this moment — an official way to ask the service itself "what's inside you right now" instead of relying on a separate handle.

---

## Exercises

### Warm up

<details>
<summary>After this code, what does <code>fake.calls</code> hold?</summary>

```rust
let mut fake = FakeNotifier::default();
fake.notify("ren@example.com", "shipped").unwrap();
fake.notify("aoi@example.com", "shipped").unwrap();
```

</details>

<details>
<summary>Answer</summary>

```text
[("ren@example.com", "shipped"), ("aoi@example.com", "shipped")]
```

Neither call failed (`fail_with` is unset), so both were recorded in `calls`, in the order they arrived.

</details>

<details>
<summary>What does this <code>notify</code> return?</summary>

```rust
let mut fake = FakeNotifier {
    fail_with: Some("outage".to_string()),
    ..Default::default()
};
fake.notify("ren@example.com", "shipped")
```

</details>

<details>
<summary>Answer</summary>

```text
Err(NotifyError("outage".to_string()))
```

`fail_with` holds a value, so `notify` returns that same reason inside a `NotifyError` without adding anything to `calls`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
struct SilentLogger;

let service = ShippingService::new(SilentLogger);
```

</details>

<details>
<summary>Answer</summary>

No — `E0277`. `SilentLogger` never got `impl Notifier for SilentLogger`, and every `N` `ShippingService::new` accepts must be a `Notifier`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let fake = FakeNotifier::default();
let mut service = ShippingService::new(fake);
println!("{:?}", fake.calls);
```

</details>

<details>
<summary>Answer</summary>

No — `E0382`. `ShippingService::new(fake)` took ownership of `fake`; the next line tries to use a moved variable.

</details>

<details>
<summary>If the first notifier inside <code>notifiers</code> fails, does <code>notify_all</code> still call the rest?</summary>

</details>

<details>
<summary>Answer</summary>

Yes. `notify_all` walks every element and only counts the successes; one failed call doesn't stop the walk.

</details>

### Repair

1. Fix `examples/05-signature-drift-broken.rs` so `StaleFake::notify`'s signature matches the trait exactly.
2. Fix `examples/06-missing-trait-impl-broken.rs` so `SilentLogger` is a genuine `Notifier` — either write `impl Notifier for SilentLogger`, or hand `ShippingService::new` a different type.
3. Fix `examples/07-use-after-move-broken.rs` by using the `notifier()` method instead of holding onto the `fake` variable.

### Implement

Three pieces in `src/lib.rs`:

```sh
cargo test -p p2-07-03-test-doubles-in-rust
```

- `impl Notifier for FakeNotifier` (inside `mod tests`) — the double itself.
- `ShippingService::ship_order` — the idempotency and injection logic.
- `notify_all` — injection through a trait object.

The exact specification for each is in the doc comment right above the function.

### Build

Pick a different dependency of your own — a `Clock` (`fn now(&self) -> u64`, or whatever), a simple in-memory `Repository`, anything you like. Write a trait for it, a "real" implementation, and a hand-rolled fake that either records something or returns a canned answer. Write a small function that injects this dependency — through a generic bound or a trait object, your choice — and a test for it. In a comment, say why you picked generic or `dyn`.

### Challenge (optional)

"An honest note on the limits" said verifying order across separate doubles gets manual. Prove it: build two `FakeNotifier`s that share one `Rc<Cell<usize>>` for numbering — every call records the counter's current value alongside `to`/`message` in `calls`, then bumps the counter. Build a `ShippingService` and a separate call using these two notifiers, and use an `assert!` to prove which one was actually called first. There is no ready-made test for this part — you write it yourself.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Test double | a stand-in for a real dependency in a test, through the same trait boundary | anywhere a dependency is slow, unpredictable, or outside your control |
| Stub | returns a fixed, canned answer | exercising error paths that are hard to trigger with the real thing |
| Fake | real, working behavior, just simplified | an in-memory store standing in for a real database |
| Spy | records what was called, with what arguments | assertions after the fact |
| Mock | pre-loaded with expectations, verifies them itself | heavier call-order/count needs |

### What you now know

- Four common shapes of test double — stub, fake, spy, mock — and that most real doubles play more than one role at once.
- How to write a trait for a dependency, a real implementation, and a hand-rolled fake for it, and where to keep that fake (`#[cfg(test)]`).
- How to choose between injection through a generic bound and injection through a trait object — the same trade-off 2.3.7 taught, this time applied to a real dependency.
- Why the compiler rejects a fake that has drifted from the trait's real signature, with no extra framework involved.
- Where this pattern runs out — verifying order across several doubles, and where a genuine integration test beats an elaborate fake.

### What comes back later

- **Property testing with `proptest`, snapshot testing with `insta`** — [2.7.4](../04-property-and-snapshot-testing/README.md)
- **Threads, Mutex, Arc — sharing a double across multiple threads** — [2.8.1](../../08-concurrency/01-threads-mutex-arc/README.md)
- **`async` methods inside a trait** — [2.9.4](../../09-async-in-practice/04-async-traits-and-blocking/README.md)

### Can you explain?

- Tell apart stub, fake, spy, and mock in your own words, each with a different example from this lesson.
- Why is `FakeNotifier` both a stub and a spy? Which field decides that?
- Why does `ShippingService::new(fake)` make holding onto the `fake` variable impossible, and how does `notifier()` work around that?
- When do you pick a generic bound versus a trait object for injecting a `Notifier`?
- Give a real example where a hand-rolled fake isn't enough and you'd genuinely reach for a mocking framework or an integration test.

---

## Going further

- [Martin Fowler — Mocks Aren't Stubs](https://martinfowler.com/articles/mocksArentStubs.html) — the essay this taxonomy comes from.
- [`mockall` on docs.rs](https://docs.rs/mockall) — a real mocking framework for Rust, for when this pattern genuinely runs out.
- [The Rust Book — Traits: Defining Shared Behavior](https://doc.rust-lang.org/book/ch10-02-traits.html) — a refresher on the trait mechanism this whole lesson rides on.
