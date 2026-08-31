//! `Box::into_raw` gives up a `Box`'s ownership and hands you the raw
//! pointer it was managing. `Box::from_raw` is the exact reverse: it takes a
//! raw pointer and reconstructs the `Box` around it, ownership and all.
//! Between the two calls, nothing owns the allocation — you do, by hand.

fn main() {
    let boxed = Box::new(42);
    let raw: *mut i32 = Box::into_raw(boxed);

    // SAFETY: `raw` came straight from `Box::into_raw` above, was never
    // freed, and is not aliased by anything else — the exact contract
    // `Box::from_raw` requires.
    let recovered = unsafe { Box::from_raw(raw) };
    println!("recovered: {recovered}");
}
