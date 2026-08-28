//! `Weak<T>` on its own: holding one, and the one thing you can do with it.
//!
//!     cargo run -p p2-06-04-weak-and-reference-cycles --example 02-weak-and-upgrade

use std::rc::{Rc, Weak};

fn main() {
    let strong = Rc::new(String::from("shared"));
    let weak: Weak<String> = Rc::downgrade(&strong);

    // Holding `weak` did not change the strong count at all.
    println!("strong_count: {}", Rc::strong_count(&strong));
    println!("weak_count:   {}", Rc::weak_count(&strong));

    match weak.upgrade() {
        Some(value) => println!("upgrade while alive: got {value:?}"),
        None => println!("upgrade while alive: got nothing"),
    }

    drop(strong);

    match weak.upgrade() {
        Some(value) => println!("upgrade after drop:  got {value:?}"),
        None => println!("upgrade after drop:  got nothing"),
    }
}
