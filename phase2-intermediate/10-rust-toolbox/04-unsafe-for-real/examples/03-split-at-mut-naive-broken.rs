//! DELIBERATELY BROKEN — expected: E0499
//!
//! Two `&mut` borrows of `slice`, both alive across the return. The borrow
//! checker has no way to see that `[..mid]` and `[mid..]` provably never
//! overlap — it only sees "two mutable borrows of the same variable."
//!
//!     cargo run -p p2-10-04-unsafe-for-real --example 03-split-at-mut-naive-broken --features broken

fn split_at_mut_naive<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    (&mut slice[..mid], &mut slice[mid..])
}

fn main() {
    let mut data = [1, 2, 3, 4, 5];
    let (left, right) = split_at_mut_naive(&mut data, 2);
    println!("{left:?} {right:?}");
}
