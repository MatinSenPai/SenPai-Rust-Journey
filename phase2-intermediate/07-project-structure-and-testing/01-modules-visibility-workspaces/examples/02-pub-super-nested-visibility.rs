//! `pub(super)` widens visibility by exactly one level: from a module to
//! its immediate parent, and no further.
//!
//! `series` is nested inside `catalog`, and `series` itself is `pub` — so
//! the path `catalog::series` is nameable from anywhere `catalog` is
//! nameable, which (since `catalog` is private-by-default at the crate
//! root) is this whole crate. But `shelf_code` inside it is `pub(super)`,
//! reaching exactly one level up to `catalog` and no further. `catalog`
//! (its direct parent) can call it — which is how `catalog::shelf_label`
//! below reaches it. Crate-root code, one level further out than `catalog`
//! itself, still cannot name `series::shelf_code` — try uncommenting the
//! line in `main` marked below.

mod catalog {
    pub struct Anime {
        pub title: String,
    }

    pub mod series {
        use super::Anime;

        pub(super) fn shelf_code(a: &Anime) -> String {
            format!("SR-{}", a.title.len())
        }
    }

    pub fn shelf_label(a: &Anime) -> String {
        format!("{}: {}", a.title, series::shelf_code(a))
    }
}

fn main() {
    let a = catalog::Anime {
        title: "Mushishi".to_string(),
    };
    println!("{}", catalog::shelf_label(&a));

    // Uncomment to see `pub(super)` stop exactly one level too early for
    // crate-root code. `series` itself is `pub`, so the path resolves —
    // but `shelf_code` only reaches as far up as `catalog`, and `main`
    // here is one level further out than that:
    // catalog::series::shelf_code(&a);
}
