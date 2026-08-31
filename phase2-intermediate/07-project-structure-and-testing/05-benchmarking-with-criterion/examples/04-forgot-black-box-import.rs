//! DELIBERATELY BROKEN — expected: E0425
//! Run `cargo run -p p2-07-05-benchmarking-with-criterion --example 04-forgot-black-box-import --features broken`
//!
//! `black_box` is not in the prelude — it has to be imported like anything
//! else in `std`. This is the single most common typo when copying a
//! criterion benchmark from memory.

fn main() {
    let hidden = black_box(7);
    println!("{hidden}");
}
