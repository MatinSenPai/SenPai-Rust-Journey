//! DELIBERATELY BROKEN — expected: E0425.
//!
//!     cargo run -p p1-02-01-stack-and-heap --example 04-out-of-scope --features broken

fn main() {
    {
        let inner = String::from("I only exist inside these braces");
        println!("inside: {inner}");
    }

    // `inner` is gone. Not empty, not null — gone. The name does not exist
    // out here, and the memory it owned was released at that brace.
    println!("outside: {inner}");
}
