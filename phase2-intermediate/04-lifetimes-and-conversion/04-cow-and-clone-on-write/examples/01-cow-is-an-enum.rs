//! `Cow<'a, B>` is a plain enum with two states. This shows both, built by
//! hand and matched by hand, and then read through `Deref` — the way you
//! will actually use it almost everywhere else in this lesson.

use std::borrow::Cow;

// clippy's `ptr_arg` lint wants `&str` here — good advice when a function
// only reads text, wrong when the whole point is to inspect which variant
// the `Cow` is, which a plain `&str` could never answer.
#[allow(clippy::ptr_arg)]
fn describe(value: &Cow<'_, str>) -> &'static str {
    match value {
        Cow::Borrowed(_) => "borrowed",
        Cow::Owned(_) => "owned",
    }
}

fn main() {
    let borrowed: Cow<str> = Cow::Borrowed("Matin");
    let owned: Cow<str> = Cow::Owned(String::from("Matin"));

    println!("borrowed: {borrowed:?} -> {}", describe(&borrowed));
    println!("owned:    {owned:?} -> {}", describe(&owned));

    // Deref means you almost never have to match by hand: both values act
    // like a `&str` directly, because `Cow<'_, str>: Deref<Target = str>`.
    println!("borrowed.len(): {}", borrowed.len());
    println!("owned.len():    {}", owned.len());
    println!("borrowed == owned: {}", borrowed == owned);
}
