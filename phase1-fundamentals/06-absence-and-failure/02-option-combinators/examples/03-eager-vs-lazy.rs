//! `.unwrap_or(x)` computes `x` every single time, whether it is needed or
//! not. `.unwrap_or_else(|| x)` only runs the closure when the `Option` is
//! `None`. The `println!` inside `expensive_default` is the proof: watch
//! which lines print and which don't.

fn expensive_default() -> i32 {
    println!("    ... expensive_default() ran ...");
    42
}

fn main() {
    let present: Option<i32> = Some(7);
    let absent: Option<i32> = None;

    println!("present.unwrap_or(expensive_default()):");
    let a = present.unwrap_or(expensive_default());
    println!("  -> {a}\n");

    println!("present.unwrap_or_else(expensive_default):");
    let b = present.unwrap_or_else(expensive_default);
    println!("  -> {b}\n");

    println!("absent.unwrap_or(expensive_default()):");
    let c = absent.unwrap_or(expensive_default());
    println!("  -> {c}\n");

    println!("absent.unwrap_or_else(expensive_default):");
    let d = absent.unwrap_or_else(expensive_default);
    println!("  -> {d}\n");

    // `.unwrap_or_default()` needs no argument at all: it reaches for the
    // type's `Default` impl. For `i32` that's 0.
    let e: Option<i32> = None;
    println!("None.unwrap_or_default(): {}", e.unwrap_or_default());
}
