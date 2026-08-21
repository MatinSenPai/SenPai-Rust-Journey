//! Two regions of memory, and which values are in which.
//!
//!     cargo run -p p1-02-01-stack-and-heap --example 02-where-things-live
//!
//! The exact addresses differ every run — that is the operating system
//! placing your program somewhere new each time. What stays true is which
//! numbers are close to each other.

fn main() {
    // Three ordinary values. All three are on the stack, in the frame
    // belonging to `main`.
    let a = 1_i32;
    let b = 2_i32;
    let c = [0_u8; 16];

    println!("stack values");
    println!("  a:          {:p}", &a);
    println!("  b:          {:p}", &b);
    println!("  c:          {:p}", &c);

    // A Vec and a String. The *variables* are on the stack, next to a, b
    // and c. Their contents are not.
    let numbers = vec![1_i32, 2, 3];
    let text = String::from("hello");

    println!();
    println!("their headers, also on the stack");
    println!("  numbers:    {:p}", &numbers);
    println!("  text:       {:p}", &text);

    println!();
    println!("what they point at, on the heap");
    println!("  numbers:    {:p}", numbers.as_ptr());
    println!("  text:       {:p}", text.as_ptr());

    // Look at the two groups. Within a group the addresses are close
    // together; between the groups they are nowhere near each other.
}
