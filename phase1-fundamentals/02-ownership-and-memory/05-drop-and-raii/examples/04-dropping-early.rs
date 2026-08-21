//! Releasing something before its scope ends.
//!
//!     cargo run -p p1-02-05-drop-and-raii --example 04-dropping-early

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
    println!("-- the default: the scope decides");
    {
        let _first = Guard::new("first");
        let _second = Guard::new("second");
    }

    println!();
    println!("-- with an early drop");
    {
        let first = Guard::new("first");
        let _second = Guard::new("second");
        drop(first);
        println!("      first is closed, second is not");
    }

    println!();
    println!("-- `drop` works on anything that owns something");
    let text = "a heap buffer".to_string();
    println!("      len before: {}", text.len());
    drop(text);
    println!("      the buffer was freed on the line above, not at the end");
}
