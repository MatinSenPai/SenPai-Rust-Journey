//! `.take(n)` / `.skip(n)` bound a pipeline by a fixed count. `.take_while()`
//! / `.skip_while()` bound it by a predicate instead — and stop asking the
//! predicate the moment its job is done.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 04-take-skip-and-while-variants

fn main() {
    let ratings = vec![9, 8, 9, 4, 7, 2];

    for r in ratings.iter().take(3) {
        println!("take(3):  {r}");
    }
    println!();
    for r in ratings.iter().skip(3) {
        println!("skip(3):  {r}");
    }

    println!();

    // `.take_while()` stops at the FIRST item the predicate rejects, and
    // never even looks at what comes after — unlike `.filter()`, which
    // would keep scanning the whole list.
    for r in ratings.iter().take_while(|r| **r >= 8) {
        println!("take_while(>= 8): {r}");
    }

    println!();

    // Proof of the short-circuit: this predicate announces every rating it
    // is asked about. Watch it stop right after the 4.
    let announced = ratings.iter().take_while(|r| {
        println!("  checking {r}");
        **r >= 8
    });
    for r in announced {
        println!("kept: {r}");
    }

    println!();

    // `.skip_while()` is the mirror image: skip every item while the
    // predicate holds, then take everything else, no more checking.
    for r in ratings.iter().skip_while(|r| **r >= 8) {
        println!("skip_while(>= 8): {r}");
    }
}
