//! DELIBERATELY BROKEN — expected: a run-time panic, "removal index (is 5)
//! should be < len (is 3)". `.remove()` and `.swap_remove()` both panic on
//! an out-of-range index — neither one returns an `Option` the way `.get()`
//! does.
//!
//!     cargo run -p p2-01-01-vec-depth --example 12-remove-out-of-bounds --features broken

fn main() {
    let mut queue = vec!["Frieren", "Bocchi", "Mushoku Tensei"];
    println!("queue: {queue:?}");
    let gone = queue.remove(5);
    println!("removed: {gone}");
}
