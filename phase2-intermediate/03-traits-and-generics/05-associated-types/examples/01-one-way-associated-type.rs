//! An associated type: the trait itself takes no type parameter, so there is
//! exactly one `ConvertsTo` implementation `Celsius` can ever have, and
//! exactly one answer for what `Target` is.

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

fn main() {
    let boiling = Celsius(100.0);
    println!("{}", boiling.convert());
}
