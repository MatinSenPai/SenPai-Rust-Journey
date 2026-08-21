//! Why you do not compare floats with `==`.
//!
//!     cargo run -p p1-01-02-scalar-types-and-overflow --example 03-float-equality

fn main() {
    let sum = 0.1 + 0.2;
    println!("0.1 + 0.2      = {sum}");
    println!("printed longer = {sum:.20}");
    println!("== 0.3         ? {}", sum == 0.3);

    // What you do instead: is it close enough?
    let close_enough = (sum - 0.3_f64).abs() < f64::EPSILON;
    println!("within epsilon ? {close_enough}");

    println!();
    println!("This is not a Rust quirk — Python prints the same thing.");
    println!("Rust just does not pretend otherwise.");
}
