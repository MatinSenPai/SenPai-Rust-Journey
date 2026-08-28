//! Two `Rc`s that each strongly own the other — a genuine memory leak,
//! entirely in safe Rust. Nothing here panics and nothing fails to compile.
//!
//!     cargo run -p p2-06-04-weak-and-reference-cycles --example 01-the-cycle-problem

use std::cell::RefCell;
use std::rc::Rc;

struct Friend {
    name: String,
    // `RefCell` lets us set this field *after* both `Friend`s already exist
    // behind an `Rc` — without it there is no way to make `alice` point at
    // `bob` and `bob` point at `alice` at the same time. 2.6.5 is where this
    // tool is actually taught; for now, `.borrow_mut()` just means
    // "temporarily unlock this field for writing".
    best_friend: RefCell<Option<Rc<Friend>>>,
}

impl Drop for Friend {
    fn drop(&mut self) {
        println!("dropping: {}", self.name);
    }
}

fn main() {
    let alice = Rc::new(Friend {
        name: "Alice".to_string(),
        best_friend: RefCell::new(None),
    });
    let bob = Rc::new(Friend {
        name: "Bob".to_string(),
        best_friend: RefCell::new(None),
    });

    *alice.best_friend.borrow_mut() = Some(Rc::clone(&bob));
    *bob.best_friend.borrow_mut() = Some(Rc::clone(&alice));

    println!("alice strong_count: {}", Rc::strong_count(&alice));
    println!("bob strong_count:   {}", Rc::strong_count(&bob));

    drop(alice);
    drop(bob);
    println!("both local bindings dropped — no \"dropping: ...\" line printed above");
}
