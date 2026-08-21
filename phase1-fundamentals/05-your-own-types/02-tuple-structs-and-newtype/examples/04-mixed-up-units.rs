//! DELIBERATELY BROKEN — expected: E0308
//! Run `cargo run -p p1-05-02-tuple-structs-and-newtype --example
//! 04-mixed-up-units --features broken` and read the error.
//!
//! Two wrappers around the same `f64`. To you they look interchangeable. To
//! the compiler they are as different as `String` and `bool`.

struct Meters(f64);
struct Feet(f64);

fn describe(height: Meters) -> String {
    format!("{} m", height.0)
}

fn main() {
    let measured = Feet(6.0);
    println!("{}", describe(measured));

    let raw = 1.83_f64;
    println!("{}", describe(raw));
}
