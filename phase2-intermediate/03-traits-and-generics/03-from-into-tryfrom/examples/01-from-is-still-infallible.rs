//! `From` is not only for errors. 1.6.5 taught `impl From<A> for MyError`
//! so `?` could convert error types; this is the same trait, doing its
//! most ordinary job — turning one type into another when the conversion
//! can never fail.
//!
//!     cargo run -p p2-03-03-from-into-tryfrom --example 01-from-is-still-infallible

fn main() {
    // Every u8 (0..=255) fits inside an i32 with room to spare — no bit
    // pattern is ever lost.
    let score: u8 = 200;
    let widened: i32 = i32::from(score);
    println!("i32::from(u8):           {widened}");

    // A different pair, same idea: every i32 fits inside an f64 exactly —
    // f64's mantissa has 52 bits, far more than i32 needs.
    let delta: i32 = -12_000;
    let as_float: f64 = f64::from(delta);
    println!("f64::from(i32):          {as_float}");

    // `.into()` is the same conversion, read from the destination's side —
    // it exists only because `From` exists, exactly as 1.6.5 showed for
    // error types.
    let via_into: i32 = score.into();
    println!(".into() (u8 -> i32):     {via_into}");
}
