//! DELIBERATELY BROKEN — expected: E0616
//! Run with `cargo run --example 05-private-field-is-inaccessible-broken
//! --features broken` and read the error.
//!
//! Same `Anime` as `01`, except `internal_rating` here has no modifier at
//! all — private by default — instead of `pub(crate)`. `main` is outside
//! `catalog` (a sibling, same as `front_desk` was in `01`), so it cannot
//! name the field.

mod catalog {
    pub struct Anime {
        pub title: String,
        internal_rating: u8,
    }

    impl Anime {
        pub fn new(title: &str, raw_rating: u8) -> Self {
            Anime {
                title: title.to_string(),
                internal_rating: raw_rating.min(10),
            }
        }
    }
}

fn main() {
    let a = catalog::Anime::new("Mushishi", 7);
    println!("{}", a.internal_rating);
}
