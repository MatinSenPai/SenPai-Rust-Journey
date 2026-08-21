//! Ownership decides *when* a value is cleaned up, not where it was born.
//!
//!     cargo run -p p1-02-05-drop-and-raii --example 03-a-move-moves-the-drop

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
    println!("-- handing it to a function");
    let given = Guard::new("given");
    consume(given);
    println!("      back in main, and it is already closed");

    println!();
    println!("-- getting one back from a function");
    {
        let _made = make("made");
        println!("      still open here, though it was built inside `make`");
    }

    println!();
    println!("-- shadowing does not close the old one early");
    {
        let _slot = Guard::new("shadowed");
        let _slot = Guard::new("replacement");
        println!("      both are still alive; the name just points elsewhere");
    }
}

fn consume(_guard: Guard) {
    println!("      inside consume");
} // <- `_guard` is owned by this function now, so it closes here

fn make(name: &str) -> Guard {
    let guard = Guard::new(name);
    println!("      inside make");
    guard // <- moved out, so it does *not* close here
}
