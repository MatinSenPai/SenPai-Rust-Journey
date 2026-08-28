//! `.to_mut()` is the literal mechanism behind "clone on write": it clones
//! only at the moment it is actually called, and only if the `Cow` was still
//! `Borrowed` at that moment. Call it again on an already-`Owned` `Cow` and
//! nothing is cloned a second time.

use std::borrow::Cow;

fn main() {
    let source = String::from("Trigun");
    let mut value: Cow<str> = Cow::Borrowed(&source);
    println!(
        "before .to_mut(): borrowed? {}",
        matches!(value, Cow::Borrowed(_))
    );
    println!("points at `source`? {}", value.as_ptr() == source.as_ptr());

    value.to_mut().push_str(" - 26 episodes");
    println!(
        "after .to_mut():  borrowed? {}",
        matches!(value, Cow::Borrowed(_))
    );
    println!("points at `source`? {}", value.as_ptr() == source.as_ptr());
    println!("value:  {value:?}");
    println!("source: {source:?} (never touched)");

    let cloned_address = value.as_ptr();
    let _ = value.to_mut(); // already `Owned` — nothing left to clone
    println!(
        "a second .to_mut() moved the data? {}",
        value.as_ptr() != cloned_address
    );
}
