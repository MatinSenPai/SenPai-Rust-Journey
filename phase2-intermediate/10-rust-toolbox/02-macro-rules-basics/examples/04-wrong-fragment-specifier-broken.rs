//! DELIBERATELY BROKEN — expected: E0425
//!
//!     cargo run -p p2-10-02-macro-rules-basics --example 04-wrong-fragment-specifier-broken --features broken

macro_rules! make_var {
    ($name:expr) => {
        let $name = 5;
    };
}

fn main() {
    make_var!(x);
    println!("{x}");
}
