//! DELIBERATELY BROKEN — expected: E0308
//!
//! `.upgrade()` never hands back the value itself — only an `Option` that
//! might hold it. Forgetting that and passing the `Option` straight to code
//! that wants the real thing is the single most common `Weak` mistake.
//!
//!     cargo run -p p2-06-04-weak-and-reference-cycles --example 04-upgrade-is-not-the-value --features broken

use std::rc::{Rc, Weak};

struct Folder {
    name: String,
}

fn print_name(folder: &Rc<Folder>) {
    println!("{}", folder.name);
}

fn main() {
    let root = Rc::new(Folder {
        name: "root".to_string(),
    });
    let weak: Weak<Folder> = Rc::downgrade(&root);

    let maybe_folder = weak.upgrade();
    print_name(&maybe_folder);
}
