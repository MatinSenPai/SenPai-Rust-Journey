//! Borrowing: using a value without taking it.
//!
//!     cargo run -p p1-03-01-shared-and-mutable-refs --example 01-look-dont-take

fn main() {
    let lines = vec![
        String::from("alpha"),
        String::from("beta"),
        String::from("gamma"),
    ];

    // `&lines` is a reference: an arrow to the Vec, not the Vec itself.
    // Nothing is copied, nothing is moved, and `lines` is still ours after.
    println!("total:       {}", total_length(&lines));
    println!("still ours:  {}", lines.len());

    // Call it again. And again. In 1.2.4 the function had to hand the Vec
    // back every single time; this one never took it in the first place.
    println!("again:       {}", total_length(&lines));
    println!("and again:   {}", total_length(&lines));

    // The arrow points at the caller's value. There is no second Vec and no
    // second buffer — the addresses are the same one.
    let view = &lines;
    println!();
    println!("lines   @:   {:p}", lines.as_ptr());
    println!("view    @:   {:p}", view.as_ptr());

    // Any number of shared arrows may exist at once, because none of them
    // can change anything. Many readers is always safe.
    let a = &lines;
    let b = &lines;
    let c = &lines;
    println!();
    println!("three readers: {} {} {}", a.len(), b.len(), c.len());

    // A borrow owns nothing, so when it ends nothing is dropped. `lines` is
    // dropped once, here, by its one owner.
    println!();
    println!("owner still holds: {lines:?}");
}

/// I only want to look at it. Keep it.
fn total_length(lines: &Vec<String>) -> usize {
    let mut total = 0;
    for line in lines {
        total += line.len();
    }
    total
}
