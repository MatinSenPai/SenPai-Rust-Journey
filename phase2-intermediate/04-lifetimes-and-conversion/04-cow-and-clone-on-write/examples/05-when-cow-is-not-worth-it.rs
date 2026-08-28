//! `Cow` earns its keep only when the no-allocation path is real. This
//! function always allocates — wrapping every input in brackets can never
//! reuse the input's own bytes — so returning `Cow` here buys nothing.

use std::borrow::Cow;

fn tag_always(input: &str) -> Cow<'_, str> {
    Cow::Owned(format!("[{input}]"))
}

fn tag_always_plain(input: &str) -> String {
    format!("[{input}]")
}

fn main() {
    for input in ["Trigun", "Blame!", ""] {
        let wrapped = tag_always(input);
        println!(
            "{input:?} -> {wrapped:?} (borrowed? {})",
            matches!(wrapped, Cow::Borrowed(_))
        );
    }
    // `tag_always_plain` does exactly the same work, with a plain `String`
    // return type the caller never has to match on or deref through.
    println!("plain version: {}", tag_always_plain("Trigun"));
}
