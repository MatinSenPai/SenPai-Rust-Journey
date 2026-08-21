//! Three values, one scope, and the order they are cleaned up in.
//!
//!     cargo run -p p1-02-05-drop-and-raii --example 02-drop-order

struct Guard {
    name: String,
}

impl Guard {
    fn new(name: &str) -> Guard {
        println!("open  {name}");
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
    println!("-- three bindings in one scope");
    {
        let _a = Guard::new("a");
        let _b = Guard::new("b");
        let _c = Guard::new("c");
        println!("      all three alive");
    }

    println!();
    println!("-- nested scopes close from the inside out");
    {
        let _outer = Guard::new("outer");
        {
            let _middle = Guard::new("middle");
            {
                let _inner = Guard::new("inner");
            }
        }
    }

    println!();
    println!("-- but a Vec cleans its elements up front to back");
    {
        let mut group = Vec::new();
        for name in ["first", "second", "third"] {
            group.push(Guard::new(name));
        }
        println!("      the Vec owns all three");
    }
}
