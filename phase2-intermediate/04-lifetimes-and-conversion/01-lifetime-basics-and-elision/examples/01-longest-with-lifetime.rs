//! Run: cargo run -p p2-04-01-lifetime-basics-and-elision --example 01-longest-with-lifetime

/// Returns whichever of `x`, `y` is longer — `x` on a tie.
///
/// `<'a>` ties both parameters and the return value to one shared lifetime:
/// the returned reference is only ever as valid as the shorter-lived of the
/// two inputs.
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let a = String::from("senpai");
    let b = String::from("kouhai teaches back");
    println!("longest: {}", longest(&a, &b));
}
