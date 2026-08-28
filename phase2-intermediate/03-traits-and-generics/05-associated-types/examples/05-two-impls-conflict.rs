//! DELIBERATELY BROKEN — expected: E0119
//!
//! `ConvertsTo` takes no type parameter, so `Celsius` gets exactly one
//! `Target`. A second `impl` for the same type is a conflict, not a second
//! option.
//!
//!     cargo run -p p2-03-05-associated-types --example 05-two-impls-conflict --features broken

trait ConvertsTo {
    type Target;
    fn convert(&self) -> Self::Target;
}

struct Celsius(f64);

impl ConvertsTo for Celsius {
    type Target = f64;

    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}

impl ConvertsTo for Celsius {
    type Target = String;

    fn convert(&self) -> String {
        format!("{:.1}C", self.0)
    }
}

fn main() {
    let boiling = Celsius(100.0);
    println!("{}", boiling.convert());
}
