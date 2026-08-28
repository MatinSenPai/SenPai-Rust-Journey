//! `Cow<'a, B>` is not a new, unrelated mechanism — it is 2.4.3's `Deref` and
//! `ToOwned` put to work together. This shows both connections directly.

use std::borrow::Cow;

// Takes a plain `&str`. A `&Cow<'_, str>` reaches it through the exact same
// deref coercion 2.4.3 covered for `&String` — `Cow` gets no special favor.
fn shout(input: &str) -> String {
    format!("{}!", input.to_uppercase())
}

fn main() {
    let value: Cow<str> = Cow::Borrowed("trigun");

    // `Cow<'_, str>: Deref<Target = str>` — this call never mentions `Cow`.
    println!("{}", shout(&value));

    // `Cow`'s `Owned` variant holds exactly `<str as ToOwned>::Owned`, which
    // is `String` — the same type 2.4.3's `.to_owned()` produces. This is
    // the only shape that fits; a bare `&str` will not.
    let built_with_to_owned: Cow<str> = Cow::Owned("trigun".to_owned());
    let built_with_string_from: Cow<str> = Cow::Owned(String::from("trigun"));
    println!(
        "same value either way: {}",
        built_with_to_owned == built_with_string_from
    );
}
