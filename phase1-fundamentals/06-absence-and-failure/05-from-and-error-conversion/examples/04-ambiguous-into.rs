//! DELIBERATELY BROKEN — expected: E0282.
//!
//!     cargo run -p p1-06-05-from-and-error-conversion --example 04-ambiguous-into --features broken

struct Kilograms(f64);
struct Grams(f64);
struct Pounds(f64);

impl From<Kilograms> for Grams {
    fn from(value: Kilograms) -> Self {
        Grams(value.0 * 1000.0)
    }
}

impl From<Kilograms> for Pounds {
    fn from(value: Kilograms) -> Self {
        Pounds(value.0 * 2.20462)
    }
}

fn main() {
    // Two types implement `From<Kilograms>`. `.into()` needs the target
    // pinned down from context, and nothing here pins it to either one.
    let result = Kilograms(5.0).into();
    println!("{}", result.0);
}
