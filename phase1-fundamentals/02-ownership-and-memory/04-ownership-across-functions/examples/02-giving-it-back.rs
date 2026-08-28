//! The pattern that makes the case for borrowing.
//!
//!     cargo run -p p1-02-04-ownership-across-functions --example 02-giving-it-back

fn main() {
    let name = String::from("Matin");

    // We want the length *and* we want to keep the String. With only the
    // tools from this module, the function has to hand it back.
    let (length, name) = measure_and_return(name);
    println!("length:     {length}");
    println!("still ours: {name}");

    // Now do it twice and watch it get worse.
    let (length, name) = measure_and_return(name);
    let (bytes, name) = measure_and_return(name);
    println!("again:      {length} {bytes} {name}");

    // Three values in and three out for something that only needed to read.
    let first = String::from("alpha");
    let second = String::from("beta");
    let (total, first, second) = total_length(first, second);
    println!("total:      {total}");
    println!("both back:  {first} {second}");

    // This works, and nobody writes it. The next module is the reason.
    println!();
    println!("every one of those returns exists only to give the value back");
}

/// Give me the string; I will tell you its length and return the string.
fn measure_and_return(text: String) -> (usize, String) {
    let length = text.len();
    (length, text)
}

/// The same idea with two arguments, which is where it stops being tolerable.
fn total_length(first: String, second: String) -> (usize, String, String) {
    let total = first.len() + second.len();
    (total, first, second)
}
