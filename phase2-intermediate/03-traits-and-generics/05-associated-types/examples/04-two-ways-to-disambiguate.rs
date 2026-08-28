//! Two ways to tell the compiler which `impl` of a generic-parameter trait
//! you mean: an explicit type on the binding, or fully qualified syntax
//! naming the trait and its `T` directly.

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

    let by_annotation: f64 = boiling.convert();
    let by_qualified = <Celsius as Converts<String>>::convert(&boiling);

    println!("{by_annotation}");
    println!("{by_qualified}");
}
