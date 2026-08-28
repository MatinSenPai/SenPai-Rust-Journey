//! The standard fix: strong references down to children, a weak reference
//! back up to the parent. No leak, no `RefCell` even needed — the whole tree
//! is built in one shot.
//!
//!     cargo run -p p2-06-04-weak-and-reference-cycles --example 03-the-fix-parent-child-tree

use std::rc::{Rc, Weak};

struct Folder {
    name: String,
    parent: Weak<Folder>,
    children: Vec<Rc<Folder>>,
}

impl Drop for Folder {
    fn drop(&mut self) {
        println!("dropping: {}", self.name);
    }
}

fn main() {
    // `Rc::new_cyclic` hands the closure a `Weak<Folder>` that will point at
    // `root` once it exists — so each child can be built holding a working
    // weak link back to a parent that, at this exact moment, isn't finished
    // yet.
    let root = Rc::new_cyclic(|weak_root| Folder {
        name: "root".to_string(),
        parent: Weak::new(),
        children: vec![
            Rc::new(Folder {
                name: "docs".to_string(),
                parent: weak_root.clone(),
                children: vec![],
            }),
            Rc::new(Folder {
                name: "src".to_string(),
                parent: weak_root.clone(),
                children: vec![],
            }),
        ],
    });

    for child in &root.children {
        match child.parent.upgrade() {
            Some(parent) => println!("{}'s parent is {}", child.name, parent.name),
            None => println!("{}'s parent is gone", child.name),
        }
    }

    // Only the `root` binding itself owns `root` strongly — both children
    // hold a *weak* link up, so neither adds to this count.
    println!("root strong_count: {}", Rc::strong_count(&root));

    drop(root);
    println!("dropped root — every folder above was cleaned up, in order");
}
