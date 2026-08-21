//! Four ways to cut text that cannot panic, and the helper you actually ship.
//!
//!     cargo run -p p1-04-04-slicing-text-safely --example 03-safe-cuts

/// The first `max_chars` characters of `text`, counted as characters.
fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}

/// `truncate_to_chars`, plus a `…` when — and only when — something was cut.
fn with_ellipsis(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let kept = truncate_to_chars(text, max_chars);
    format!("{kept}…")
}

fn main() {
    let title = "برنامه‌نویسی سیستمی با Rust";

    // 1. Ask instead of demanding: `.get()` hands back `None` for a cut that
    //    `&title[..]` would have panicked on.
    println!("get(0..20) = {:?}", title.get(0..20));
    println!("get(0..19) = {:?}", title.get(0..19));

    // 2. Take the answer, or a stand-in, without unwrapping anything.
    println!("or empty   = {:?}", title.get(0..20).unwrap_or(""));

    // 3. Snap a byte budget down to the last legal cut below it.
    let snapped = title.floor_char_boundary(20);
    println!("floor(20)  = {snapped} -> {:?}", &title[..snapped]);

    // 4. `split_at` panics on a bad boundary; `split_at_checked` does not.
    println!("split_at_checked(20) = {:?}", title.split_at_checked(20));
    println!("split_at_checked(19) = {:?}", title.split_at_checked(19));

    // And the two helpers, on four kinds of input.
    println!();
    let samples = ["برنامه‌نویسی سیستمی", "Rust systems programming", "سلام", ""];
    for sample in samples {
        println!(
            "{:>4} chars, {:>2} bytes -> {:?}",
            sample.chars().count(),
            sample.len(),
            with_ellipsis(sample, 8)
        );
    }
}
