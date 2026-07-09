//! Each function below is a *fixed* version of a classic borrow-checker
//! trap. Filling in the `todo!()`s correctly means writing code that
//! satisfies the borrow checker on the first try — read each doc comment
//! carefully, it explains exactly which trap this function's shape avoids.

/// Returns the length of `s`'s first whitespace-separated word, then clears
/// `s` entirely, then returns the length. The trap this avoids: computing
/// the length into a plain `usize` (which is `Copy` — an owned value, not a
/// borrow) *before* calling `.clear()`, so there's no live reference into
/// `s` at the moment you mutate it.
pub fn first_word_len_then_clear(s: &mut String) -> usize {
    todo!(
        "compute the first word's length into an owned usize FIRST, then s.clear(), then return it"
    )
}

/// Builds and returns an owned greeting — contrast this with the classic
/// "dangling reference" trap (a function that tries to return `&String`
/// pointing at a local variable that's about to be dropped). Returning an
/// **owned** `String` sidesteps the problem entirely: ownership moves to
/// the caller, nothing dangles.
pub fn make_greeting(name: &str) -> String {
    todo!("format! an owned String, e.g. \"Hello, {{name}}!\"")
}

/// Builds a description of `s` (using only shared, read-only access), then
/// separately mutates `s` by appending " (grown)". The trap this avoids:
/// if the read-only access were still "alive" (e.g. a `&str` slice held
/// across the mutation), the later `s.push_str(...)` wouldn't compile.
/// Scoping the read-only work inside its own `{ ... }` block ends that
/// borrow before the mutation begins.
pub fn describe_and_grow(s: &mut String) -> String {
    todo!("build `description` inside a `{{ }}` block using only shared access to s, then s.push_str(\" (grown)\"), then return description")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measures_then_clears() {
        let mut s = String::from("hello world");
        assert_eq!(first_word_len_then_clear(&mut s), 5);
        assert_eq!(s, "");
    }

    #[test]
    fn greets() {
        assert_eq!(make_greeting("Matin"), "Hello, Matin!");
    }

    #[test]
    fn describes_then_grows() {
        let mut s = String::from("rust");
        let description = describe_and_grow(&mut s);
        assert_eq!(description, "4 chars, starts with 'r'");
        assert_eq!(s, "rust (grown)");
    }
}

// UNCOMMENT ME (then run `cargo check -p p1-03-02-borrow-checker-rules`):
//
// fn conflicting_borrows_demo() {
//     let mut s = String::from("hello");
//     let r1 = &s;
//     let r2 = &mut s; // <- the problem. Read the error carefully.
//     println!("{r1} {r2}");
// }
