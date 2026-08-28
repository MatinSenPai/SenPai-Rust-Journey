//! DELIBERATELY BROKEN — expected: a run-time panic, "called
//! `Result::unwrap()` on an `Err` value: TryFromIntError(PosOverflow)". It
//! compiles cleanly — `.unwrap()` type-checks on any `Result<T, E>` — and
//! then it dies the moment the value doesn't fit.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 08-narrow-panics-on-overflow --features broken

fn main() {
    let big: i32 = 300;
    let narrowed = u8::try_from(big).unwrap();
    println!("{narrowed}");
}
