//! Five ways to build a string, and what each one costs.
//!
//!     cargo run -p p1-04-03-building-and-transforming-strings --example 03-building-a-string

use std::fmt::Write;

fn main() {
    // 1. push_str / push. One buffer, filled in place.
    let mut out = String::with_capacity(32);
    println!("empty  @ {:p} cap {}", out.as_ptr(), out.capacity());
    out.push_str("report");
    out.push('-');
    out.push_str("2026");
    println!("filled @ {:p} cap {} = {out}", out.as_ptr(), out.capacity());

    // 2. `+`. It takes the left side by value and reuses its buffer.
    let mut left = String::with_capacity(64);
    left.push_str("report");
    println!("left   @ {:p}", left.as_ptr());
    let right = "-2026".to_string();
    let sum = left + &right;
    println!("sum    @ {:p} = {sum}", sum.as_ptr());

    // 3. concat and join — the ones 1.1.6 built by hand and promised here.
    let parts = vec!["نام".to_string(), "شهر".to_string(), "سال".to_string()];
    println!("join   = {}", parts.join("، "));
    println!("concat = {}", ["a", "b", "c"].concat());

    // 4. format!. The readable one, and one fresh allocation every time.
    let row = format!("{} ({})", parts[0], parts.len());
    println!("format = {row}");

    // 5. write!. format!'s machinery aimed at a buffer you already own.
    let mut buf = String::new();
    let _ = write!(buf, "{}/{}", 3, 4);
    let _ = write!(buf, " = {:.2}", 3.0 / 4.0);
    println!("write  = {buf}");

    // The loop that allocates once per turn...
    let mut slow = String::new();
    for part in &parts {
        slow = format!("{slow}{part} ");
    }
    // ...and the loop that does not.
    let mut fast = String::new();
    for part in &parts {
        fast.push_str(part);
        fast.push(' ');
    }
    println!();
    println!("slow = [{}]", slow.trim_end());
    println!("fast = [{}]", fast.trim_end());
    println!("same answer, {} allocations against 1", parts.len());
}
