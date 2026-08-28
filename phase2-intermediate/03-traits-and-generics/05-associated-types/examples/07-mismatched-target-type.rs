//! DELIBERATELY BROKEN — expected: E0308
//!
//! `Target` was declared `f64`, so the compiler holds the body to exactly
//! that — the associated type is a real commitment, not a comment.
//!
//!     cargo run -p p2-03-05-associated-types --example 07-mismatched-target-type --features broken

trait ConvertsTo {
    type Target;
    fn convert(&self) -> Self::Target;
}

struct Celsius(f64);

impl ConvertsTo for Celsius {
    type Target = f64;

    fn convert(&self) -> Self::Target {
        format!("{:.1}C", self.0)
    }
}

fn main() {
    let boiling = Celsius(100.0);
    println!("{}", boiling.convert());
}
