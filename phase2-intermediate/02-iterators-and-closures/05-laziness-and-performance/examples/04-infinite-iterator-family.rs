//! Three ways to build an iterator with no defined end. None of these could
//! exist under an eager model — there is no way to eagerly build an
//! infinite `Vec`. They are safe here only because nothing runs until a
//! bounded `.take()` asks for values.
//!
//!     cargo run -p p2-02-05-laziness-and-performance --example 04-infinite-iterator-family

fn main() {
    let repeated: Vec<&str> = std::iter::repeat("frieren").take(4).collect();
    println!("repeat(\"frieren\").take(4):        {repeated:?}");

    let pattern = [1, 2, 3];
    let cycled: Vec<i32> = pattern.iter().copied().cycle().take(8).collect();
    println!("[1, 2, 3].iter().cycle().take(8): {cycled:?}");

    let powers: Vec<u32> = std::iter::successors(Some(1u32), |&x| Some(x * 2))
        .take(6)
        .collect();
    println!("successors(1, |x| x * 2).take(6): {powers:?}");

    let empty: Vec<i32> = Vec::new();
    let cycled_empty: Vec<i32> = empty.iter().copied().cycle().take(5).collect();
    println!("[].iter().cycle().take(5):        {cycled_empty:?}");
}
