//! DELIBERATELY BROKEN — expected: E0308.
//!
//!     cargo run -p p1-03-04-slices --example 05-vec-parameter --features broken
//!
//! The reason `&[i32]` is the parameter type and `&Vec<i32>` is not: this
//! function has locked itself to one kind of caller for no benefit at all.

fn main() {
    let grown = vec![10, 20, 30];
    println!("vec:   {}", total(&grown));

    let fixed = [1, 2, 3, 4];
    println!("array: {}", total(&fixed));
    println!("part:  {}", total(&grown[1..]));
}

fn total(values: &Vec<i32>) -> i32 {
    let mut sum = 0;
    for value in values {
        sum += value;
    }
    sum
}
