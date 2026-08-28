//! Destructuring: taking a compound value apart by writing its shape.
//!
//!     cargo run -p p1-01-03-compound-types-and-destructuring --example 03-destructuring

fn main() {
    let sample = (1_700_000_000_u32, 21.5_f64, true);

    // The left of the `=` is not three new statements. It is one pattern that
    // has the same shape as the value, and the names in it get filled in.
    let (timestamp, celsius, verified) = sample;
    println!("timestamp: {timestamp}");
    println!("celsius:   {celsius}");
    println!("verified:  {verified}");

    // `_` means "there is a value here and I do not want a name for it".
    let (_, only_celsius, _) = sample;
    println!("only:      {only_celsius}");

    // Arrays destructure the same way, and the compiler checks the count.
    let corners = [1, 2, 3, 4];
    let [top_left, top_right, bottom_left, bottom_right] = corners;
    println!("corners:   {top_left} {top_right} {bottom_left} {bottom_right}");

    // Patterns nest as deeply as the value does.
    let route = ((35.7, 51.4), (32.6, 51.7));
    let ((from_lat, from_lon), (to_lat, to_lon)) = route;
    println!("from:      {from_lat}, {from_lon}");
    println!("to:        {to_lat}, {to_lon}");

    // A tuple returned from a function is destructured at the call site, which
    // is how Rust returns more than one value without inventing a type for it.
    let (boxes, left_over) = divide(47, 6);
    println!("47 in 6s:  {boxes} boxes, {left_over} left over");

    // Because the right-hand side is evaluated before anything is bound, a
    // swap needs no temporary variable.
    let mut a = 1;
    let mut b = 2;
    println!("before:    a={a} b={b}");
    (a, b) = (b, a);
    println!("after:     a={a} b={b}");
}

/// How many whole boxes `total` fills, and how many items are left over.
fn divide(total: u32, per_box: u32) -> (u32, u32) {
    (total / per_box, total % per_box)
}
