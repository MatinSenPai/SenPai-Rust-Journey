//! The newtype pattern: the same `u64`, wrapped so the compiler knows what it
//! means.
//!
//! Run `cargo run -p p1-05-02-tuple-structs-and-newtype --example
//! 02-newtype-in-action`.

/// An account number. Nothing but a `u64` — and not interchangeable with one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AccountId(u64);

/// Money, as an integer in the smallest unit (1.1.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rial(i64);

impl Rial {
    fn new(amount: i64) -> Rial {
        Rial(amount)
    }

    fn amount(self) -> i64 {
        self.0
    }
}

/// Three arguments, one type. Every wrong call compiles.
fn transfer_untyped(from: u64, to: u64, rial: u64) -> String {
    format!("{rial} rial: {from} -> {to}")
}

/// Three arguments, three types. The amount can no longer land in an account
/// slot, and neither can an account number land in the amount slot.
fn transfer(from: AccountId, to: AccountId, amount: Rial) -> String {
    format!("{} rial: {} -> {}", amount.amount(), from.0, to.0)
}

/// A percentage that cannot be built out of range: the constructor is the only
/// way in, and it clamps.
mod rates {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Percent(u8);

    impl Percent {
        pub fn new(value: u8) -> Percent {
            if value > 100 {
                Percent(100)
            } else {
                Percent(value)
            }
        }

        pub fn value(self) -> u8 {
            self.0
        }
    }
}

fn main() {
    // The amount went into the `to` slot. The compiler had nothing to say.
    println!("wrong:  {}", transfer_untyped(1001, 250_000, 2002));
    println!("right:  {}", transfer_untyped(1001, 2002, 250_000));

    let alice = AccountId(1001);
    let bob = AccountId(2002);

    println!("{}", transfer(alice, bob, Rial::new(250_000)));
    println!("{}", transfer(bob, alice, Rial::new(1)));

    println!("debug:  {:?}", Rial::new(250_000));
    println!("equal:  {}", Rial::new(5) == Rial::new(5));

    let fee = rates::Percent::new(9);
    let silly = rates::Percent::new(240);
    println!("fee:    {}%", fee.value());
    println!("clamped: {}% (asked for 240)", silly.value());
    println!("debug:  {silly:?}");
}
