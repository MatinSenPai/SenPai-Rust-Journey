//! Width, alignment and precision — and what the number in `{:>8}` counts.
//!
//!     cargo run -p p1-04-03-building-and-transforming-strings --example 02-width-and-precision

fn main() {
    // Width pads to a minimum. Alignment says which side the padding goes.
    println!("[{:>8}]", "hello");
    println!("[{:<8}]", "hello");
    println!("[{:^8}]", "hello");
    println!("[{:*>8}]", "hello");

    // Text defaults to the left, numbers to the right. Say it anyway.
    println!("[{:8}]", "42");
    println!("[{:8}]", 42);

    // Precision on a number rounds it.
    println!("[{:.2}]", 1.0_f64 / 3.0);
    println!("[{:>8.2}]", 1.0_f64 / 3.0);

    // Precision on text truncates it — and counts characters, not bytes.
    println!("[{:.3}]", "hello");
    println!("[{:.2}]", "سلام");

    // Both numbers can come from variables, with a `$` after the name.
    let width = 10;
    let places = 3;
    println!("[{:>width$.places$}]", 2.0_f64.sqrt());

    // Now the honest part: `8` counts `char`s. Nothing else.
    let english = "hello";
    let persian = "سلام";
    let joined = "می‌شود";
    println!();
    println!(
        "{english}: {} chars, {} bytes",
        english.chars().count(),
        english.len()
    );
    println!(
        "{persian}: {} chars, {} bytes",
        persian.chars().count(),
        persian.len()
    );
    println!(
        "{joined}: {} chars, {} bytes",
        joined.chars().count(),
        joined.len()
    );
    println!("[{english:>8}]");
    println!("[{persian:>8}]");
    println!("[{joined:>8}]");
}
