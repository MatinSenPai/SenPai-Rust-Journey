//! Trimming, splitting, replacing, case — and which of them allocate.
//!
//!     cargo run -p p1-04-03-building-and-transforming-strings --example 04-trim-split-replace-case

fn main() {
    // trim borrows: same buffer, narrower view, no allocation at all.
    let raw = "  نام: ماتین  ";
    let clean = raw.trim();
    let tail_only = raw.trim_end();
    println!("[{clean}]");
    println!(
        "trim_end starts at the same byte: {}",
        tail_only.as_ptr() == raw.as_ptr()
    );

    // split borrows too. Every piece is a view into `line`.
    let line = "ماتین,رشت,۲۶";
    for field in line.split(',') {
        println!(
            "field [{field}] — {} bytes, {} chars",
            field.len(),
            field.chars().count()
        );
    }

    // Empty pieces are kept, which is nearly always what you want.
    for field in "a,,b".split(',') {
        print!("[{field}]");
    }
    println!();

    // replace always allocates — even when it replaces nothing.
    let untouched = "abc".replace('z', "!");
    println!("replaced nothing: {untouched}");
    println!(
        "new buffer anyway: {}",
        untouched.as_ptr() != "abc".as_ptr()
    );

    // Case conversion allocates because the answer can be a different length.
    println!("{} -> {}", "rust", "rust".to_uppercase());
    println!("{} -> {}", "straße", "straße".to_uppercase());
    println!(
        "{} chars -> {} chars",
        "straße".chars().count(),
        "straße".to_uppercase().chars().count()
    );

    // And Persian has no upper case, so nothing happens at all.
    let fa = "سلام دنیا";
    println!("{} -> {}", fa, fa.to_uppercase());
    println!("unchanged: {}", fa.to_uppercase() == fa);
}
