//! An array is a fixed number of values that all share one type.
//!
//!     cargo run -p p1-01-03-compound-types-and-destructuring --example 02-arrays

fn main() {
    // `[type; how many]`. The length is part of the type.
    let readings: [i32; 5] = [12, 7, 19, 3, 14];

    println!("readings:  {readings:?}");
    println!("length:    {}", readings.len());

    // Indexing starts at zero, like the tuple positions.
    println!("first:     {}", readings[0]);
    println!("last:      {}", readings[readings.len() - 1]);

    // Repeat syntax: this value, that many times.
    let zeroed = [0_u8; 8];
    println!("zeroed:    {zeroed:?}");

    // The length is genuinely part of the type, so these two cannot be mixed.
    // Uncomment the assignment and you get E0308 — see the README.
    let four: [i32; 4] = [1, 2, 3, 4];
    let five: [i32; 5] = [1, 2, 3, 4, 5];
    println!("four:      {four:?}");
    println!("five:      {five:?}");
    // let same: [i32; 4] = five;

    // Every index is checked. `.get()` asks without risking a panic: it hands
    // back `Some(value)` or `None` instead of stopping the program.
    println!("get(2):    {:?}", readings.get(2));
    println!("get(99):   {:?}", readings.get(99));

    // An array is one value, so it can go straight into a tuple.
    let labelled = ("sensor-a", readings);
    println!("labelled:  {labelled:?}");
}
