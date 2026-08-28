//! `Option<T>` usually needs room for a tag — "is this Some or None?" — on
//! top of `T` itself. For pointer-shaped `T`s it doesn't, because a valid
//! pointer is never all-zero-bits, and `None` can borrow that bit pattern
//! for free. This is the null-pointer optimisation.
//!
//! `bool` gets the same trick for a different reason: it only uses 2 of a
//! byte's 256 patterns, so `None` moves into one of the other 254.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 05-null-pointer-optimization

use std::mem::size_of;

fn main() {
    // No spare bit pattern in a plain i32 or bool: every value they can hold
    // is meaningful, so `Option` needs somewhere else to put "there's
    // nothing here" — and that costs real bytes.
    println!("size_of::<i32>():           {}", size_of::<i32>());
    println!("size_of::<Option<i32>>():   {}", size_of::<Option<i32>>());
    println!();
    println!("size_of::<bool>():          {}", size_of::<bool>());
    println!("size_of::<Option<bool>>():  {}", size_of::<Option<bool>>());

    // A reference is never null in safe Rust — that guarantee is the whole
    // point of the borrow checker. All-zero-bits is a pattern `&i32` was
    // never going to use for a real value, so `None` moves in there instead.
    println!();
    println!("size_of::<&i32>():          {}", size_of::<&i32>());
    println!("size_of::<Option<&i32>>():  {}", size_of::<Option<&i32>>());

    // Same story for `Box<T>` — it owns a heap pointer and that pointer is
    // never null either. `Box` gets its own lesson in Phase 2; today you
    // only need that it holds one pointer, same shape as `&T`.
    println!();
    println!("size_of::<Box<i32>>():         {}", size_of::<Box<i32>>());
    println!(
        "size_of::<Option<Box<i32>>>(): {}",
        size_of::<Option<Box<i32>>>()
    );

    println!();
    println!("Option<i32> costs more than i32: every bit pattern of i32 is a");
    println!("real number, so None needs a byte of its own to be told apart.");
    println!();
    println!("Option<bool>, Option<&T> and Option<Box<T>> cost NOTHING extra:");
    println!("bool only uses 2 of a byte's 256 patterns, and a real pointer is");
    println!("never all-zero-bits — so None moves into a pattern going spare.");
}
