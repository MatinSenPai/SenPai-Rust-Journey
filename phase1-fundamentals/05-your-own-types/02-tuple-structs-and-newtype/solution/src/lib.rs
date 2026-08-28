//! Reference solution for 1.5.2 — Tuple structs and the newtype pattern.
//!
//! Three wrappers, each around a number you already know how to store. The
//! types were written for you; the constructors, accessors and the functions
//! that speak in those types were not.

/// An account number.
///
/// The wrapped `u64` is public: an account number has no invalid values, so
/// there is nothing for a constructor to defend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccountId(pub u64);

/// An amount of money, in rial — an integer in the smallest unit, never a
/// float.
///
/// The wrapped `i64` is private, so `Rial::new` is the only way in from
/// another module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rial(i64);

/// A whole-number percentage.
///
/// The wrapped `u8` is private, and `Percent::new` will not let a value above
/// 100 through.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Percent(u8);

impl Rial {
    /// A `Rial` holding exactly `amount`.
    ///
    /// No validation: a negative amount is a legitimate `Rial` (a refund, a
    /// debit) and passes through unchanged.
    ///
    /// # Examples
    ///
    /// `Rial::new(250_000).amount()` is `250_000`.
    /// `Rial::new(-50).amount()` is `-50`.
    pub fn new(amount: i64) -> Rial {
        Rial(amount)
    }

    /// The integer inside, in rial.
    ///
    /// # Examples
    ///
    /// `Rial::new(0).amount()` is `0`.
    pub fn amount(self) -> i64 {
        self.0
    }
}

impl Percent {
    /// A `Percent` holding `value`, clamped to the range 0 to 100.
    ///
    /// A `value` of 100 or below is stored as it is. Anything above 100 is
    /// stored as 100. (The honest answer to "this input is wrong" is a
    /// `Result`, which arrives in 1.6.3; clamping is what this lesson can
    /// build today.)
    ///
    /// # Examples
    ///
    /// `Percent::new(9).value()` is `9`.
    /// `Percent::new(100).value()` is `100`.
    /// `Percent::new(240).value()` is `100`.
    pub fn new(value: u8) -> Percent {
        if value > 100 {
            Percent(100)
        } else {
            Percent(value)
        }
    }

    /// The whole number inside, between 0 and 100.
    ///
    /// # Examples
    ///
    /// `Percent::new(0).value()` is `0`.
    pub fn value(self) -> u8 {
        self.0
    }

    /// This percentage of `amount`, as a `Rial`.
    ///
    /// The result is truncated toward zero, so 3% of 1050 rial is 31 rial and
    /// 3% of -1050 rial is -31 rial. Take the percentage of the whole amount:
    /// 10% of 1050 rial is 105 rial, not 100.
    ///
    /// # Examples
    ///
    /// `Percent::new(15).of(Rial::new(1_000))` is `Rial::new(150)`.
    /// `Percent::new(10).of(Rial::new(1_050))` is `Rial::new(105)`.
    /// `Percent::new(3).of(Rial::new(1_050))` is `Rial::new(31)`.
    /// `Percent::new(3).of(Rial::new(-1_050))` is `Rial::new(-31)`.
    /// `Percent::new(100).of(Rial::new(1_050))` is `Rial::new(1_050)`.
    /// `Percent::new(0).of(Rial::new(1_050))` is `Rial::new(0)`.
    pub fn of(self, amount: Rial) -> Rial {
        Rial(amount.0 * self.0 as i64 / 100)
    }
}

/// Everything in `amounts`, added up.
///
/// An empty slice totals `Rial::new(0)`.
///
/// # Examples
///
/// `total(&[Rial::new(10), Rial::new(32)])` is `Rial::new(42)`.
/// `total(&[])` is `Rial::new(0)`.
pub fn total(amounts: &[Rial]) -> Rial {
    let mut sum = 0;
    for amount in amounts {
        sum += amount.0;
    }
    Rial(sum)
}

/// A one-line record of a transfer, in exactly this shape:
///
/// ```text
/// 1001 -> 2002: 250000 rial
/// ```
///
/// The sending account's number, a space, an arrow, a space, the receiving
/// account's number, a colon, a space, the amount, a space, and the word
/// `rial`.
///
/// # Examples
///
/// `transfer(AccountId(1001), AccountId(2002), Rial::new(250_000))` is
/// `"1001 -> 2002: 250000 rial"`.
/// `transfer(AccountId(7), AccountId(8), Rial::new(-50))` is
/// `"7 -> 8: -50 rial"`.
pub fn transfer(from: AccountId, to: AccountId, amount: Rial) -> String {
    format!("{} -> {}: {} rial", from.0, to.0, amount.amount())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_rial_holds_what_it_was_given() {
        assert_eq!(Rial::new(250_000).amount(), 250_000);
        assert_eq!(Rial::new(0).amount(), 0);
        assert_eq!(Rial::new(-50).amount(), -50);
    }

    #[test]
    fn a_percent_cannot_be_built_above_a_hundred() {
        assert_eq!(Percent::new(9).value(), 9);
        assert_eq!(Percent::new(0).value(), 0);
        assert_eq!(Percent::new(100).value(), 100);
        assert_eq!(Percent::new(101).value(), 100);
        assert_eq!(Percent::new(240).value(), 100);
        assert_eq!(Percent::new(255).value(), 100);
    }

    #[test]
    fn amounts_add_up() {
        assert_eq!(total(&[Rial::new(10), Rial::new(32)]), Rial::new(42));
        assert_eq!(total(&[Rial::new(-5), Rial::new(5)]), Rial::new(0));
        assert_eq!(total(&[Rial::new(7)]), Rial::new(7));
        assert_eq!(total(&[]), Rial::new(0));
    }

    #[test]
    fn a_percentage_of_an_amount_truncates_toward_zero() {
        assert_eq!(Percent::new(15).of(Rial::new(1_000)), Rial::new(150));
        assert_eq!(Percent::new(10).of(Rial::new(1_050)), Rial::new(105));
        assert_eq!(Percent::new(3).of(Rial::new(1_050)), Rial::new(31));
        assert_eq!(Percent::new(3).of(Rial::new(-1_050)), Rial::new(-31));
        assert_eq!(Percent::new(0).of(Rial::new(1_050)), Rial::new(0));
        assert_eq!(Percent::new(100).of(Rial::new(1_050)), Rial::new(1_050));
    }

    #[test]
    fn a_transfer_reads_the_same_way_every_time() {
        assert_eq!(
            transfer(AccountId(1001), AccountId(2002), Rial::new(250_000)),
            "1001 -> 2002: 250000 rial"
        );
        assert_eq!(
            transfer(AccountId(7), AccountId(8), Rial::new(-50)),
            "7 -> 8: -50 rial"
        );
        assert_eq!(
            transfer(AccountId(0), AccountId(0), Rial::new(0)),
            "0 -> 0: 0 rial"
        );
    }
}
