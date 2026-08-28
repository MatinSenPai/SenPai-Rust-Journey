//! `&text[a..b]` counts bytes. In English that looks like counting letters,
//! which is exactly why the bug survives review in an English codebase.
//!
//!     cargo run -p p1-04-04-slicing-text-safely --example 01-bytes-not-characters

fn main() {
    let english = "programming";
    let persian = "برنامه‌نویسی";

    println!(
        "english: {} chars, {} bytes",
        english.chars().count(),
        english.len()
    );
    println!(
        "persian: {} chars, {} bytes",
        persian.chars().count(),
        persian.len()
    );

    // Seven bytes of English is seven letters. The two numbers agree.
    println!();
    println!("&english[0..7] = {:?}", &english[0..7]);

    // Seven bytes of Persian is neither seven letters nor a legal cut. Asking
    // with `.get()` answers instead of ending the program.
    println!("persian.get(0..7) = {:?}", persian.get(0..7));
    println!("persian.get(0..6) = {:?}", persian.get(0..6));

    // Every byte index from 0 to 8, and what each one would give you.
    println!();
    println!("index  boundary?  get(0..index)");
    for index in 0..=8 {
        println!(
            "{index:>5}  {:>9}  {:?}",
            persian.is_char_boundary(index),
            persian.get(0..index)
        );
    }
}
