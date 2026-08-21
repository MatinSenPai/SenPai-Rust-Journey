//! `.clone()` is not free, and here is exactly what it is not free of.
//!
//!     cargo run -p p1-02-03-clone-and-copy --example 01-what-clone-costs

fn main() {
    let original = String::from("hello");
    let copy = original.clone();

    // Both usable — that is what you paid for.
    println!("original:   {original}");
    println!("copy:       {copy}");

    // And here is the payment. A move left the address alone; a clone did
    // not, because a clone asked the allocator for a second buffer.
    println!("original @: {:p}", original.as_ptr());
    println!("copy     @: {:p}", copy.as_ptr());

    // Cloning does not preserve capacity. It allocates exactly what is
    // needed, which makes `.clone()` a way to shrink an over-sized buffer.
    let mut roomy: Vec<i32> = Vec::with_capacity(100);
    roomy.push(1);
    roomy.push(2);
    roomy.push(3);
    let tight = roomy.clone();

    println!();
    println!("roomy len/cap:  {}/{}", roomy.len(), roomy.capacity());
    println!("clone len/cap:  {}/{}", tight.len(), tight.capacity());

    // Cloning a collection of owning things clones every one of them. This
    // is one allocation for the Vec plus one per String — four in total.
    let lines = vec![
        String::from("alpha"),
        String::from("beta"),
        String::from("gamma"),
    ];
    let copied = lines.clone();

    println!();
    println!("vec     @: {:p}", lines.as_ptr());
    println!("its copy@: {:p}", copied.as_ptr());
    println!("line 0  @: {:p}", lines[0].as_ptr());
    println!("its copy@: {:p}", copied[0].as_ptr());
    println!();
    println!("that clone made 4 allocations: the Vec, and one per String");
}
