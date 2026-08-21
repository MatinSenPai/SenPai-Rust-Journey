//! `.len()` counts bytes. `.chars().count()` counts scalars. They agree for
//! English and disagree for everything else you write.
//!
//!     cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 02-three-ways-to-count

fn main() {
    println!("bytes  chars   text");
    report("hello");
    report("Rust");
    report("سلام");
    report("متین");
    report("سلام، من متین هستم.");
    report("Rust برای بک‌اند");
    report("🌸");
    report("سلام 🌸");
    report("");

    // The rule a signup form really wants to state.
    let name = "محمدمتین";
    println!();
    println!("name: {name}");
    println!("  len()            = {}", name.len());
    println!("  chars().count()  = {}", name.chars().count());
    println!("  rejected by a 12-byte rule:   {}", name.len() > 12);
    println!(
        "  rejected by a 12-letter rule: {}",
        name.chars().count() > 12
    );
}

/// Byte length and scalar count, with the text last so the columns survive
/// right-to-left rendering.
fn report(text: &str) {
    println!("{:>5}  {:>5}   {text}", text.len(), text.chars().count());
}
