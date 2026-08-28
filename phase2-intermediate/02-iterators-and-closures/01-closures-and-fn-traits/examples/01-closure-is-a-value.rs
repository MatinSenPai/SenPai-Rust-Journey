//! A closure is an ordinary value you can bind to a `let`, pass around, and
//! call -- but its type is compiler-generated and has no name you could type
//! yourself. `std::any::type_name` proves it: the printed name contains
//! `{{closure}}`, which is not valid Rust syntax.
//!
//!     cargo run -p p2-02-01-closures-and-fn-traits --example 01-closure-is-a-value

fn type_name_of<V>(_value: &V) -> String {
    std::any::type_name::<V>().to_string()
}

fn main() {
    let add_one = |x: i32| x + 1;
    let add_two = |x: i32| x + 2;

    println!("add_one(5) = {}", add_one(5));
    println!("add_two(5) = {}", add_two(5));
    println!("type of add_one: {}", type_name_of(&add_one));
    println!("type of add_two: {}", type_name_of(&add_two));
}
