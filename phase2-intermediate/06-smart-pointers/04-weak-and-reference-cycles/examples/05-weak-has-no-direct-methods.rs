//! DELIBERATELY BROKEN — expected: E0599
//!
//! `Weak<T>` does not implement `Deref` the way `Rc<T>` does — there is no
//! auto-deref to `T` because the compiler cannot prove `T` is even still
//! there. `.upgrade()` first, always.
//!
//!     cargo run -p p2-06-04-weak-and-reference-cycles --example 05-weak-has-no-direct-methods --features broken

use std::rc::{Rc, Weak};

struct Folder {
    name: String,
}

impl Folder {
    fn shout(&self) -> String {
        self.name.to_uppercase()
    }
}

fn main() {
    let root = Rc::new(Folder {
        name: "root".to_string(),
    });
    let weak: Weak<Folder> = Rc::downgrade(&root);

    println!("{}", weak.shout());
}
