//! `return` finally has something to do.
//!
//!     cargo run -p p1-01-05-control-flow --example 03-early-return

fn main() {
    println!("21, 3:     {}", split_evenly(21, 3));
    println!("21, 0:     {}", split_evenly(21, 0));
    println!("prime 97:  {}", is_prime(97));
    println!("prime 91:  {}", is_prime(91));
}

/// A guard clause: deal with the impossible case first and leave, so the rest
/// of the function does not have to think about it.
///
/// Returning 0 for "cannot divide" is a poor answer and you can already feel
/// why — 0 is also a perfectly good real answer. 1.6.1 fixes this properly.
fn split_evenly(total: u32, parts: u32) -> u32 {
    if parts == 0 {
        return 0;
    }
    total / parts
}

/// Leaving early out of a loop, which is the other half of what `return` is
/// for. The moment the answer is settled, stop working.
fn is_prime(n: u32) -> bool {
    if n < 2 {
        return false;
    }
    let mut divisor = 2;
    while divisor * divisor <= n {
        if n % divisor == 0 {
            return false;
        }
        divisor += 1;
    }
    true
}
