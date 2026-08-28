//! How big is a value, really?
//!
//!     cargo run -p p1-02-01-stack-and-heap --example 01-sizes

use std::mem::size_of;
use std::mem::size_of_val;

fn main() {
    // The simple ones are the size you would guess.
    println!("i32:            {} bytes", size_of::<i32>());
    println!("i64:            {} bytes", size_of::<i64>());
    println!("bool:           {} bytes", size_of::<bool>());
    println!("char:           {} bytes", size_of::<char>());
    println!("usize:          {} bytes", size_of::<usize>());
    println!("():             {} bytes", size_of::<()>());

    // An array is its element size times its length, all of it on the stack.
    println!();
    println!("[i32; 5]:       {} bytes", size_of::<[i32; 5]>());
    println!("[i32; 100]:     {} bytes", size_of::<[i32; 100]>());

    // And now the interesting ones. These do not change with the contents,
    // because the contents are not here.
    println!();
    println!("Vec<i32>:       {} bytes", size_of::<Vec<i32>>());
    println!("Vec<i64>:       {} bytes", size_of::<Vec<i64>>());
    println!("String:         {} bytes", size_of::<String>());
    println!("&str:           {} bytes", size_of::<&str>());
    println!("&i32:           {} bytes", size_of::<&i32>());

    // Three pointer-sized numbers: where the data is, how much of it there
    // is, and how much room was reserved.
    println!();
    println!("3 x usize:      {} bytes", 3 * size_of::<usize>());

    // `size_of_val` measures a value rather than a type — and gives the same
    // answer, because what it measures is still only the part on the stack.
    let empty: Vec<i32> = Vec::new();
    let full: Vec<i32> = vec![1; 1_000];
    println!();
    println!("empty vec:      {} bytes", size_of_val(&empty));
    println!("1000-item vec:  {} bytes", size_of_val(&full));
    println!("its contents:   {} bytes", full.len() * size_of::<i32>());
}
