//! A small anime catalog crate — the running example this lesson built up.
//! `catalog` holds the data and a nested `pricing` submodule; the crate
//! root re-exports the one type callers actually need.

mod catalog {
    pub struct Anime {
        pub title: String,
        pub(crate) internal_rating: u8,
    }

    impl Anime {
        pub fn new(title: &str, raw_rating: u8) -> Self {
            Anime {
                title: title.to_string(),
                internal_rating: clamp_rating(raw_rating),
            }
        }

        /// Buckets the hidden `internal_rating` into a coarse public band,
        /// without ever exposing the raw number to callers outside this
        /// crate.
        ///
        /// Bands: `0..=3` is `"low"`, `4..=7` is `"medium"`, `8..=10` is
        /// `"high"`.
        pub fn public_rating_band(&self) -> &'static str {
            match self.internal_rating {
                0..=3 => "low",
                4..=7 => "medium",
                _ => "high",
            }
        }

        /// Rental price in cents: `base_price_cents` with
        /// `pricing::discount_percent(self.internal_rating)` taken off,
        /// rounded down to the nearest cent.
        ///
        /// `price = base_price_cents * (100 - discount_percent) / 100`
        pub fn rental_price_cents(&self, base_price_cents: u32) -> u32 {
            let discount = pricing::discount_percent(self.internal_rating) as u32;
            base_price_cents * (100 - discount) / 100
        }
    }

    // Private by default: an implementation detail of `Anime::new`, not
    // something any other module — inside this crate or outside it —
    // should ever need to call directly.
    fn clamp_rating(raw: u8) -> u8 {
        raw.min(10)
    }

    mod pricing {
        /// Crate-internal discount policy, as a whole-number percentage
        /// taken off the base price. `0..=3` is `0`, `4..=7` is `10`,
        /// `8..=10` is `25`.
        ///
        /// `pub(super)`, not `pub`: this is `catalog`'s business, not the
        /// whole crate's, and it is never part of the public API.
        pub(super) fn discount_percent(internal_rating: u8) -> u8 {
            match internal_rating {
                0..=3 => 0,
                4..=7 => 10,
                _ => 25,
            }
        }
    }
}

pub use catalog::Anime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_out_of_range_ratings() {
        let a = Anime::new("Frieren: Beyond Journey's End", 250);
        // `internal_rating` is `pub(crate)`, so this test — living inside
        // this same crate — can read it directly.
        assert_eq!(a.internal_rating, 10);
    }

    #[test]
    fn new_keeps_in_range_ratings_untouched() {
        let a = Anime::new("Mushishi", 7);
        assert_eq!(a.internal_rating, 7);
    }

    #[test]
    fn public_rating_band_buckets_low_medium_high() {
        assert_eq!(Anime::new("Show A", 2).public_rating_band(), "low");
        assert_eq!(Anime::new("Show B", 5).public_rating_band(), "medium");
        assert_eq!(Anime::new("Show C", 9).public_rating_band(), "high");
    }

    #[test]
    fn rental_price_cents_applies_the_matching_discount() {
        assert_eq!(Anime::new("Show A", 2).rental_price_cents(1000), 1000); // low: 0% off
        assert_eq!(Anime::new("Show B", 5).rental_price_cents(1000), 900); // medium: 10% off
        assert_eq!(Anime::new("Show C", 9).rental_price_cents(1000), 750); // high: 25% off
    }

    #[test]
    fn rental_price_cents_rounds_down() {
        // 999 * 75 / 100 = 749.25, and integer division rounds down.
        assert_eq!(Anime::new("Show C", 9).rental_price_cents(999), 749);
    }
}
