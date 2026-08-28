//! A generic parameter on the trait: `Converts<f64>` and `Converts<String>`
//! are different concrete traits as far as the compiler is concerned, so
//! `Celsius` can implement both at once.

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
    let fahrenheit: f64 = boiling.convert();
    let label: String = boiling.convert();
    println!("{fahrenheit}");
    println!("{label}");
}
