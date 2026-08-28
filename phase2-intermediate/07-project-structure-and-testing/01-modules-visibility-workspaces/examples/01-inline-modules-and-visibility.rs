//! `mod` builds a tree, and visibility controls who can see into it.
//!
//! `catalog` and `front_desk` are siblings — neither is nested inside the
//! other. `front_desk` can read `title` because it is `pub`. It can also
//! read `internal_rating`, even though `front_desk` is not a descendant of
//! `catalog` at all — because `pub(crate)` means "anywhere in this crate,"
//! not "this module and its children."

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
    }

    // No modifier — private by default. Visible only inside `catalog`
    // itself (and any module nested inside it). `front_desk`, a sibling,
    // cannot name this function at all.
    fn clamp_rating(raw: u8) -> u8 {
        raw.min(10)
    }
}

mod front_desk {
    use crate::catalog::Anime;

    pub fn describe(a: &Anime) -> String {
        format!("{} (internal rating {}/10)", a.title, a.internal_rating)
    }
}

fn main() {
    let a = catalog::Anime::new("Frieren: Beyond Journey's End", 250);
    println!("{}", front_desk::describe(&a));
}
