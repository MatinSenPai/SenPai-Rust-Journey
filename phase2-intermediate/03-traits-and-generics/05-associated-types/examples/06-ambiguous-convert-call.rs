//! DELIBERATELY BROKEN — expected: E0283
//!
//! `Celsius` implements `Converts<f64>` and `Converts<String>` both. Nothing
//! here says which `T` the caller wants, so the compiler cannot pick.
//!
//!     cargo run -p p2-03-05-associated-types --example 06-ambiguous-convert-call --features broken

trait Converts<T> {
    fn convert(&self) -> T;
}

struct Celsius(f64);

impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}

impl Converts<String> for Celsius {
    fn convert(&self) -> String {
        format!("{:.1}C", self.0)
    }
}

fn main() {
    let boiling = Celsius(100.0);
    let result = boiling.convert();
    println!("{result}");
}
