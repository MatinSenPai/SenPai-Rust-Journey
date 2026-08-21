//! Cleanup you can watch happen.
//!
//!     cargo run -p p1-02-05-drop-and-raii --example 01-drop-runs
//!
//! `struct` and `impl` get their own lesson in 1.5.1. Here they are only "a
//! type made of one field" and "code attached to that type".

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

// The one method of the `Drop` trait. The compiler calls it; you never do.
impl Drop for Guard {
    fn drop(&mut self) {
        println!("close {}", self.name);
    }
}

fn main() {
    println!("-- before the block");
    {
        let _inner = Guard::new("inner");
        println!("-- inside the block");
    } // <- the closing brace is where `close inner` comes from
    println!("-- after the block");

    println!();

    // Nothing below says "close this". The compiler put the call in, at the
    // end of `main`, and there is no path out that skips it.
    let _last = Guard::new("last");
    println!("-- end of main");
}
