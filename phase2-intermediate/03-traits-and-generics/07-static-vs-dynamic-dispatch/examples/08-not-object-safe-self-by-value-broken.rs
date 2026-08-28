//! DELIBERATELY BROKEN — expected: E0038.
//!
//! `spinoff` returns `Self` *by value*. Behind `&dyn Spinoff` the compiler
//! only knows "some type implementing `Spinoff`" — it has no fixed size,
//! so there is no way to return one of it by value. A vtable slot cannot
//! promise a return size it does not know.
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 08-not-object-safe-self-by-value-broken --features broken

trait Spinoff {
    fn spinoff(&self) -> Self;
}

struct AnimeSeries {
    title: String,
}

impl Spinoff for AnimeSeries {
    fn spinoff(&self) -> Self {
        AnimeSeries {
            title: format!("{} (spinoff)", self.title),
        }
    }
}

fn announce(_item: &dyn Spinoff) {}

fn main() {
    println!("compiled");
}
