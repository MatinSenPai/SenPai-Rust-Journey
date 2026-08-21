//! DELIBERATELY BROKEN — expected: E0040
//!
//!     cargo run -p p1-02-05-drop-and-raii --example 05-calling-drop-yourself --features broken

struct Guard {
    name: String,
}

impl Guard {
    fn new(name: &str) -> Guard {
        Guard {
            name: name.to_string(),
        }
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        println!("close {}", self.name);
    }
}

fn main() {
    let guard = Guard::new("early");

    // It is a method, it is in scope, and it does exactly what we want.
    guard.drop();

    println!("done");
}
