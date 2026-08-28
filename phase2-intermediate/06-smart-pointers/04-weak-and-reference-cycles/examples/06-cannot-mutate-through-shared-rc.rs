//! DELIBERATELY BROKEN — expected: E0594
//!
//! This is the exact wall `01-the-cycle-problem.rs` needed `RefCell` to get
//! past: an `Rc<T>` only ever hands out `&T`, never `&mut T`, no matter how
//! many — or how few — owners currently exist.
//!
//!     cargo run -p p2-06-04-weak-and-reference-cycles --example 06-cannot-mutate-through-shared-rc --features broken

use std::rc::{Rc, Weak};

struct Folder {
    name: String,
    parent: Weak<Folder>,
}

fn main() {
    let root = Rc::new(Folder {
        name: "root".to_string(),
        parent: Weak::new(),
    });
    let docs = Rc::new(Folder {
        name: "docs".to_string(),
        parent: Weak::new(),
    });

    docs.parent = Rc::downgrade(&root);
}
