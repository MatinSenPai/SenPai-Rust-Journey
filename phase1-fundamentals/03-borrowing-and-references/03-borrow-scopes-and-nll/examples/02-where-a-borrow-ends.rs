//! 1.3.3 — three borrows of one `Vec`, and where each of them stops.
//!
//! `unused` is never read on purpose: the `unused variable` warning this
//! example prints is part of what it is showing you. A borrow with no uses
//! has nothing to reach forward to, so it conflicts with nothing.

fn main() {
    let mut totals = vec![10, 20, 30];

    let view = &totals;
    let counted = view.len();
    // `view` is over. Its last use was the line above.

    totals.push(40);

    let unused = &totals;

    totals.push(50);

    let again = &totals;
    println!("first look: {counted} items");
    println!("last look:  {} items", again.len());
}
