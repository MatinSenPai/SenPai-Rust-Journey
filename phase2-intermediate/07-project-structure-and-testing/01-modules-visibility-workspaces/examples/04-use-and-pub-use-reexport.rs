//! `use` shortens a path for the code that writes it. `pub use` does that
//! *and* re-exports the item at the new, shallower path for everyone else —
//! without moving where it's actually defined. Both paths below name the
//! exact same `Anime` type; there is only ever one definition, three
//! doors into it.

mod catalog {
    pub mod series {
        pub struct Anime {
            pub title: String,
        }
    }
}

// Re-exports `Anime` at the crate root: callers write `Anime`, or (from
// another crate depending on this one) `this_crate::Anime` — never the
// three-module-deep internal path.
pub use catalog::series::Anime;

mod front_desk {
    // `use` here only saves retyping; it re-exports nothing.
    use crate::catalog::series::Anime;

    pub fn label(a: &Anime) -> String {
        format!("now showing: {}", a.title)
    }
}

fn main() {
    // Built through the flat, re-exported path...
    let a = Anime {
        title: "Frieren".to_string(),
    };
    // ...and handed to a function that imported the full internal path.
    // Same type either way — `pub use` never creates a second one.
    println!("{}", front_desk::label(&a));
}
