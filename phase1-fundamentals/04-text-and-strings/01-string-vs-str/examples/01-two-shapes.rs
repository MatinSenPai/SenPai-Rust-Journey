//! What a `String` is made of, and what a `&str` is made of.
//!
//!     cargo run -p p1-04-01-string-vs-str --example 01-two-shapes

fn main() {
    // Three words against two. A word here is 8 bytes, on a 64-bit machine.
    println!("size_of::<String>()  = {}", std::mem::size_of::<String>());
    println!("size_of::<&str>()    = {}", std::mem::size_of::<&str>());
    println!("size_of::<&String>() = {}", std::mem::size_of::<&String>());
    println!();

    // The String owns a heap buffer. The view points into that same buffer:
    // no allocation happened on the second line, and the addresses say so.
    let owned = String::from("سلام دنیا");
    let view: &str = owned.as_str();

    println!("owned = {owned}");
    println!("view  = {view}");
    println!("owned @ {:p}", owned.as_ptr());
    println!("view  @ {:p}", view.as_ptr());
    println!();

    // The third word is the one the view has not got: reserved room.
    println!("owned len/cap = {}/{}", owned.len(), owned.capacity());
    println!("view  len     = {}", view.len());
    println!();

    // And that third word is what lets a String grow: it takes a bigger
    // block and records the new room. Whether the address survives is the
    // allocator's business, not the String's.
    let mut growing = String::from("سلام");
    println!(
        "before push: len/cap = {}/{} @ {:p}",
        growing.len(),
        growing.capacity(),
        growing.as_ptr()
    );
    growing.push_str(" دنیا");
    println!(
        "after  push: len/cap = {}/{} @ {:p}",
        growing.len(),
        growing.capacity(),
        growing.as_ptr()
    );
    println!();
    println!("a &str has no third word, so it can never grow");
}
