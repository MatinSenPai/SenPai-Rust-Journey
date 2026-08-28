//! DELIBERATELY BROKEN — expected: E0038.
//!
//! `rating_as::<T>` is generic — every distinct `T` a caller might ask for
//! needs its own vtable entry, but a vtable is one fixed table, built once,
//! before any caller has chosen a `T`. There is no way to pre-build an
//! unbounded number of slots.
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 09-not-object-safe-generic-method-broken --features broken

trait Rated {
    fn rating_as<T: From<u8>>(&self) -> T;
}

struct AnimeSeries {
    rating: u8,
}

impl Rated for AnimeSeries {
    fn rating_as<T: From<u8>>(&self) -> T {
        T::from(self.rating)
    }
}

fn announce(_item: &dyn Rated) {}

fn main() {
    println!("compiled");
}
