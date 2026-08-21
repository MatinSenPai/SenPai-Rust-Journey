//! DELIBERATELY BROKEN — expected: E0308 and E0527.
//!
//!     cargo run -p p1-01-03-compound-types-and-destructuring --example 05-wrong-arity --features broken

fn main() {
    let sample = (1_700_000_000_u32, 21.5_f64, true);

    // The pattern has two names; the value has three fields.
    let (timestamp, celsius) = sample;
    println!("{timestamp} {celsius}");

    let corners = [1, 2, 3, 4];
    // The pattern has three slots; the array has four.
    let [a, b, c] = corners;
    println!("{a} {b} {c}");
}
