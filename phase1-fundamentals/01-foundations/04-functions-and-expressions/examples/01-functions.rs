//! Declaring and calling functions.
//!
//!     cargo run -p p1-01-04-functions-and-expressions --example 01-functions

fn main() {
    // Order does not matter. `main` calls a function declared below it, and
    // that function calls one declared below *it*. Rust reads the whole file
    // before it compiles any of it.
    println!("area:      {}", area(3, 4));
    println!("perimeter: {}", perimeter(3, 4));
    println!("kelvin:    {}", to_kelvin(21.5));

    // A function that returns nothing returns `()`, and you can prove it.
    let nothing = announce(7);
    println!("announce:  {nothing:?}");

    // Parameters are values of their own inside the body. Reassigning one
    // needs `mut`, exactly like any other binding.
    println!("doubled:   {}", doubled_the_hard_way(21));
}

/// Every parameter is annotated. Rust never infers a parameter's type — the
/// signature is a contract, and a contract you have to guess is not one.
fn area(width: u32, height: u32) -> u32 {
    width * height
}

/// The body calls another function; nothing special about that.
fn perimeter(width: u32, height: u32) -> u32 {
    2 * (width + height)
}

fn to_kelvin(celsius: f64) -> f64 {
    celsius + 273.15
}

/// No `->` means the return type is `()`, the unit type from 1.1.3.
fn announce(order_id: u32) {
    println!("           order {order_id} received");
}

/// A parameter is a binding like any other, so it needs `mut` to change.
fn doubled_the_hard_way(mut n: u32) -> u32 {
    n = n * 2;
    n
}
