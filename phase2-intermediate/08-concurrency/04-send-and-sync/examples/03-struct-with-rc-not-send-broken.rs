//! DELIBERATELY BROKEN — expected: E0277.
//!
//! `HoldsRc` has one field that isn't `Send` (`Rc<i32>`), so the struct
//! itself isn't `Send` either — a struct is `Send` only when every one of
//! its fields is.
//!
//!     cargo run -p p2-08-04-send-and-sync --example 03-struct-with-rc-not-send-broken --features broken

use std::rc::Rc;

struct HoldsRc {
    label: String,
    count: Rc<i32>,
}

fn assert_send<T: Send>() {}

fn main() {
    assert_send::<HoldsRc>();
}
