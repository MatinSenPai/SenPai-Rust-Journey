//! `while let` — keep going for as long as the pattern keeps matching.
//!
//!     cargo run -p p1-05-05-if-let-while-let-let-else --example 02-while-let-drain

fn main() {
    // `Vec::pop` hands back `Some(last)` until the Vec is empty, then `None`.
    // That is exactly the shape `while let` was made for.
    let mut stack = vec![1, 2, 3];
    while let Some(top) = stack.pop() {
        println!("popped:     {top}   (left: {})", stack.len());
    }
    println!("stopped:    pop() answered None, so the loop ended");

    // The loop ends the FIRST time the pattern fails. Nothing is skipped and
    // retried: `checked_sub` answers None at zero, and that is the end of it.
    println!();
    let mut countdown: u32 = 3;
    while let Some(next) = countdown.checked_sub(1) {
        println!("countdown:  {next}");
        countdown = next;
    }
    println!("stopped:    0 has no predecessor in u32");

    // You can still leave early, exactly as in any other loop.
    println!();
    let mut readings = vec![4, 5, -1, 6];
    let mut total = 0;
    while let Some(value) = readings.pop() {
        if value < 0 {
            println!("negative:   {value} — leaving the loop");
            break;
        }
        total += value;
    }
    println!("total:      {total}");
    println!("left over:  {readings:?}");
}
