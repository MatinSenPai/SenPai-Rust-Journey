// This crate simulates a tiny anime catalog to demonstrate `mod`,
// visibility (`pub`, `pub(crate)`, private-by-default), and re-exporting
// with `pub use`. Read the module tree top to bottom.

mod catalog {
    /// A single anime series in the catalog.
    pub struct Anime {
        /// Public field — visible to anyone who can see `Anime` at all,
        /// including external crates that depend on this one.
        pub title: String,

        /// Crate-internal detail. `pub(crate)` means "visible anywhere in
        /// this crate (including this crate's own unit tests below), but
        /// NOT part of the public API." An external crate that added this
        /// one as a dependency could construct an `Anime` and read
        /// `title`, but could not even name `internal_rating` — the
        /// compiler would report it as a private field.
        pub(crate) internal_rating: u8,
    }

    impl Anime {
        /// Constructs a new `Anime`. `raw_rating` is clamped into the
        /// `0..=10` range (via the crate-internal `normalize_rating`
        /// helper) before being stored.
        pub fn new(title: &str, raw_rating: u8) -> Self {
            Anime {
                title: title.to_string(),
                internal_rating: normalize_rating(raw_rating),
            }
        }

        /// Buckets the hidden `internal_rating` into a coarse public band,
        /// without ever exposing the raw number to callers outside this
        /// crate. This is the whole point of `pub(crate)`: callers get a
        /// stable, deliberately-vague public API (`"low"` / `"medium"` /
        /// `"high"`), while we stay free to change the underlying 0..=10
        /// scale (or replace it with something else entirely) without
        /// breaking anyone who depends on us.
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
    }

    /// Crate-internal helper: clamps a raw `u8` rating into `0..=10`.
    /// `pub(crate)` (not `pub`) because this is an implementation detail of
    /// how `Anime::new` normalizes input — not something an external crate
    /// should be calling directly.
    pub(crate) fn normalize_rating(raw: u8) -> u8 {
        raw.min(10)
    }
}

// Re-exporting `Anime` at the crate root means callers write
// `p2_06_01_modules_visibility_workspaces::Anime` (or, from inside this
// crate, `crate::Anime`) instead of the deeper
// `crate::catalog::Anime` — a flat, ergonomic public path on top of a
// nested internal module structure.
pub use catalog::Anime;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_clamps_out_of_range_ratings() {
        let a = Anime::new("Frieren: Beyond Journey's End", 250);
        // `internal_rating` is `pub(crate)`, so this test — living inside
        // this same crate — can read it directly. An external crate could
        // not write `a.internal_rating` at all; it would fail to compile.
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
    fn normalize_rating_is_reachable_from_anywhere_in_the_crate() {
        // `catalog` itself has no `pub` in front of `mod catalog`, so it's
        // private to the crate root and its descendants — this test module
        // is one such descendant, so the path resolves. `normalize_rating`
        // being `pub(crate)` (not `pub`) is what lets us call it directly
        // here, even though it will never appear in this crate's public
        // docs or be callable from an external crate.
        assert_eq!(catalog::normalize_rating(15), 10);
        assert_eq!(catalog::normalize_rating(3), 3);
        assert_eq!(catalog::normalize_rating(0), 0);
    }
}
