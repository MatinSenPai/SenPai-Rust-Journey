//! Reference solution for 2.3.6 — supertraits, blanket impls, the orphan
//! rule, and the newtype escape hatch.
//!
//! `Discounted` is a supertrait of `Priced`. `Cart` is a newtype around
//! `Vec<Pen>`, written specifically so a foreign trait (`Display`) can be
//! implemented for it — `impl Display for Vec<Pen>` directly would be
//! `E0117`, because both `Display` and `Vec` are foreign.

/// Anything that can report a price, in whole cents.
pub trait Priced {
    fn price_cents(&self) -> u32;
}

pub struct Book {
    pub price_cents: u32,
}

impl Priced for Book {
    fn price_cents(&self) -> u32 {
        self.price_cents
    }
}

pub struct Pen {
    pub price_cents: u32,
}

impl Priced for Pen {
    fn price_cents(&self) -> u32 {
        self.price_cents
    }
}

/// Anything `Priced` can also report a discounted price. Requires `Priced`
/// as a supertrait, so `discounted_cents` can call `price_cents`.
pub trait Discounted: Priced {
    /// 90% of `price_cents()`, using integer math: `price_cents() * 90 / 100`.
    /// For example, 1000 cents discounts to 900; 101 cents discounts to 90
    /// (integer division truncates 90.9 down to 90).
    fn discounted_cents(&self) -> u32 {
        self.price_cents() * 90 / 100
    }
}

impl Discounted for Book {}
impl Discounted for Pen {}

/// The amount saved by buying at the discounted price instead of full price:
/// `item.price_cents()` minus `item.discounted_cents()`.
///
/// `T: Discounted` is enough to call both methods here — `Discounted`'s
/// supertrait bound already guarantees `T: Priced` too.
pub fn amount_saved<T: Discounted>(item: &T) -> u32 {
    item.price_cents() - item.discounted_cents()
}

/// A shopping cart: an ordered list of pens. A newtype around `Vec<Pen>` so
/// a foreign trait (`Display`, below) can be implemented for it.
pub struct Cart(pub Vec<Pen>);

impl Cart {
    /// Sum of `price_cents()` for every pen in the cart. An empty cart is `0`.
    pub fn total_cents(&self) -> u32 {
        self.0.iter().map(Pen::price_cents).sum()
    }
}

impl std::fmt::Display for Cart {
    /// Formats as `"{count} pen(s), {total} cents total"` — for example, a
    /// cart holding pens priced 100 and 250 cents formats as
    /// `"2 pen(s), 350 cents total"`. An empty cart formats as
    /// `"0 pen(s), 0 cents total"`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} pen(s), {} cents total",
            self.0.len(),
            self.total_cents()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discount_is_ninety_percent_with_integer_truncation() {
        assert_eq!(Book { price_cents: 1000 }.discounted_cents(), 900);
        assert_eq!(Pen { price_cents: 250 }.discounted_cents(), 225);
        assert_eq!(Book { price_cents: 101 }.discounted_cents(), 90);
    }

    #[test]
    fn amount_saved_is_price_minus_discounted() {
        assert_eq!(amount_saved(&Book { price_cents: 1000 }), 100);
        assert_eq!(amount_saved(&Pen { price_cents: 250 }), 25);
    }

    #[test]
    fn cart_total_sums_every_pen() {
        let cart = Cart(vec![Pen { price_cents: 100 }, Pen { price_cents: 250 }]);
        assert_eq!(cart.total_cents(), 350);
    }

    #[test]
    fn empty_cart_totals_zero() {
        let cart = Cart(vec![]);
        assert_eq!(cart.total_cents(), 0);
    }

    #[test]
    fn cart_display_matches_total_cents() {
        // If `fmt` computed its own sum instead of calling `total_cents()`,
        // the two could drift apart silently.
        let cart = Cart(vec![Pen { price_cents: 100 }, Pen { price_cents: 250 }]);
        assert_eq!(
            format!("{cart}"),
            format!("2 pen(s), {} cents total", cart.total_cents())
        );
    }

    #[test]
    fn cart_display_exact_text() {
        let cart = Cart(vec![Pen { price_cents: 100 }, Pen { price_cents: 250 }]);
        assert_eq!(format!("{cart}"), "2 pen(s), 350 cents total");
    }

    #[test]
    fn empty_cart_display() {
        let cart = Cart(vec![]);
        assert_eq!(format!("{cart}"), "0 pen(s), 0 cents total");
    }
}
