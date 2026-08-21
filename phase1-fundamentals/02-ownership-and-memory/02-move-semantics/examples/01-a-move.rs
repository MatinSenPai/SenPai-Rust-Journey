//! What assignment does to a value that owns heap memory.
//!
//!     cargo run -p p1-02-02-move-semantics --example 01-a-move

fn main() {
    let first = String::from("hello");

    // This does not copy the text. It hands the ownership of that heap buffer
    // from `first` to `second`. The three words are copied; the buffer is not.
    let second = first;

    // `second` works exactly as you would expect.
    println!("second:     {second}");
    println!("length:     {}", second.len());

    // `first` no longer works. Not because it is empty — because it is no
    // longer a valid binding at all. Uncomment to see E0382, or look at
    // 04-use-after-move.
    // println!("first:      {first}");

    // The buffer did not move in memory. Only the responsibility for it did.
    let third = second;
    println!("third:      {third}");
    println!("its buffer: {:p}", third.as_ptr());

    // A move is cheap and its cost does not depend on the size of the data:
    // three machine words, every time.
    let large = vec![0_u8; 10_000_000];
    let also_large = large;
    println!(
        "ten MB moved, {} bytes copied",
        std::mem::size_of_val(&also_large)
    );
}
