//! `T`'s own size varies wildly. `Box<T>`'s size never does — for any `T`
//! that is `Sized`, `Box<T>` is exactly one pointer, always.

use std::mem::size_of;

struct Tiny;
#[allow(dead_code)]
struct Small {
    id: u32,
}
#[allow(dead_code)]
struct Medium {
    name: String,
    count: u64,
}
#[allow(dead_code)]
struct Large {
    buffer: [u8; 4096],
}

fn main() {
    println!("--- T's own size varies wildly ---");
    println!("size_of::<Tiny>()   = {}", size_of::<Tiny>());
    println!("size_of::<Small>()  = {}", size_of::<Small>());
    println!("size_of::<Medium>() = {}", size_of::<Medium>());
    println!("size_of::<Large>()  = {}", size_of::<Large>());

    println!("--- Box<T> never does ---");
    println!("size_of::<Box<Tiny>>()     = {}", size_of::<Box<Tiny>>());
    println!("size_of::<Box<Small>>()    = {}", size_of::<Box<Small>>());
    println!("size_of::<Box<Medium>>()   = {}", size_of::<Box<Medium>>());
    println!("size_of::<Box<Large>>()    = {}", size_of::<Box<Large>>());
    println!("size_of::<Box<i32>>()      = {}", size_of::<Box<i32>>());
    println!(
        "size_of::<Box<Box<i32>>>() = {}",
        size_of::<Box<Box<i32>>>()
    );

    println!("--- the niche optimisation this buys for free ---");
    println!(
        "size_of::<Option<i32>>()      = {}",
        size_of::<Option<i32>>()
    );
    println!(
        "size_of::<Option<Box<i32>>>() = {}",
        size_of::<Option<Box<i32>>>()
    );
}
