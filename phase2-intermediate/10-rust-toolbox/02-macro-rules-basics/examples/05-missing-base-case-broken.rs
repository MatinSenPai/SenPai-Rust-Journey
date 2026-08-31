//! DELIBERATELY BROKEN — expected: unexpected end of macro invocation
//!
//!     cargo run -p p2-10-02-macro-rules-basics --example 05-missing-base-case-broken --features broken

macro_rules! largest_of {
    ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
        std::cmp::max($first, largest_of!( $( $rest ),+ ))
    };
}

fn main() {
    let m = largest_of!(3, 9, 7);
    println!("{m}");
}
