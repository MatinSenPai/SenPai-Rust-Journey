//! Retrying `PUT` leaves a resource in the same state no matter how many
//! times it happens; retrying `POST` keeps happening, and keeps changing
//! things. This is what "idempotent" buys you when a request might have to
//! be retried after a dropped connection or a timeout — you can safely
//! resend a `PUT` without knowing whether the first one arrived; resending
//! a `POST` is a real risk of doing it twice.
//!
//!     cargo run -p p3-01-03-http-semantics-you-must-know --example 01-idempotency-in-practice

/// A toy "server": a single stored balance, plus a count of every deposit
/// ever applied.
struct Account {
    balance_cents: i64,
    deposits_applied: u32,
}

impl Account {
    /// `PUT /balance` — idempotent: sets the balance outright, however many
    /// times you call it with the same value.
    fn put_balance(&mut self, cents: i64) {
        self.balance_cents = cents;
    }

    /// `POST /deposit` — not idempotent: every call adds another deposit.
    fn post_deposit(&mut self, cents: i64) {
        self.balance_cents += cents;
        self.deposits_applied += 1;
    }
}

/// A client resending a request it never got a reply for — the exact
/// situation a timed-out request leaves you in: you genuinely don't know
/// whether the server applied it.
fn retry_three_times(mut call: impl FnMut()) {
    call();
    call();
    call();
}

fn main() {
    let mut put_target = Account {
        balance_cents: 0,
        deposits_applied: 0,
    };
    retry_three_times(|| put_target.put_balance(500));
    println!(
        "after 3x PUT /balance 500:   balance = {}, deposits_applied = {}",
        put_target.balance_cents, put_target.deposits_applied
    );

    let mut post_target = Account {
        balance_cents: 0,
        deposits_applied: 0,
    };
    retry_three_times(|| post_target.post_deposit(500));
    println!(
        "after 3x POST /deposit 500:  balance = {}, deposits_applied = {}",
        post_target.balance_cents, post_target.deposits_applied
    );
}
