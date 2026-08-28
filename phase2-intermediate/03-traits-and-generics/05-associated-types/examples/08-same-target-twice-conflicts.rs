//! DELIBERATELY BROKEN — expected: E0119
//!
//! Two impls, both for `T = f64`. `Converts<f64>` is one concrete trait, so
//! this is the exact same conflict as `05-two-impls-conflict.rs` — proof
//! that the restriction is per concrete `T`, not per trait name.
//!
//!     cargo run -p p2-03-05-associated-types --example 08-same-target-twice-conflicts --features broken

trait Converts<T> {
    fn convert(&self) -> T;
}

struct Celsius(f64);

impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 * 9.0 / 5.0 + 32.0
    }
}

impl Converts<f64> for Celsius {
    fn convert(&self) -> f64 {
        self.0 + 273.15
    }
}

fn main() {
    let boiling = Celsius(100.0);
    let fahrenheit: f64 = boiling.convert();
    println!("{fahrenheit}");
}
