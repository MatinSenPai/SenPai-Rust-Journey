//! `format!` is `println!` that hands you the string instead of printing it.
//!
//!     cargo run -p p1-04-03-building-and-transforming-strings --example 01-format-basics

fn main() {
    // The same mini-language you have used inside `println!` since Phase 0 —
    // only the destination changed.
    let name = "ماتین";
    let lesson = 3;
    let line = format!("{name} is on lesson {lesson}");
    println!("{line}");

    // Three ways to say which argument goes where.
    println!("{} / {}", "left", "right");
    println!("{0} / {1} / {0}", "first", "second");
    println!("{who} / {what}", who = "Matin", what = "Rust");

    // `{}` asks for the human form. `{:?}` asks for the programmer's form.
    let title = "سلام دنیا".to_string();
    println!("display: {title}");
    println!("debug:   {title:?}");

    // Debug is what a collection has. Display is what it does not.
    let parts = vec!["one", "two"];
    println!("debug:   {parts:?}");
    println!("pretty:  {parts:#?}");

    // A tuple has Debug too, and it keeps the quotes on the text inside.
    let pair = ("سلام", 3);
    println!("tuple:   {pair:?}");

    // `format!` builds a brand-new `String` and hands you the buffer.
    let built = format!("{}-{}", "report", 2026);
    println!(
        "built @ {:p} len {} cap {}",
        built.as_ptr(),
        built.len(),
        built.capacity()
    );
}
