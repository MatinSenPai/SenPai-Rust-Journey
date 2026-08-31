//! The fix for example 05: a type's author can grant `Send` by hand, once
//! they've personally verified it's genuinely safe — exactly the promise
//! 2.8.4 said "belongs to a different lesson."

struct RawHolder<T> {
    ptr: *mut T,
}

unsafe impl<T: Send> Send for RawHolder<T> {}

fn assert_send<T: Send>() {}

fn main() {
    assert_send::<RawHolder<i32>>();
    println!("RawHolder<i32> is Send");
}
