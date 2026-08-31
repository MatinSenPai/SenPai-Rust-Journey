//! DELIBERATELY BROKEN — expected: E0277
//!
//! `RawHolder<T>` has exactly one field, `ptr: *mut T` — and raw pointers
//! are neither `Send` nor `Sync` by default, for *any* `T`, even a `T` that
//! is itself `Send`. 2.8.4 promised this fact belonged here; this is it.
//!
//!     cargo run -p p2-10-04-unsafe-for-real --example 05-raw-pointer-not-send-broken --features broken

struct RawHolder<T> {
    ptr: *mut T,
}

fn assert_send<T: Send>() {}

fn main() {
    assert_send::<RawHolder<i32>>();
}
