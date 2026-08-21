//! Where a literal lives, and what every conversion between the two types
//! actually costs.
//!
//!     cargo run -p p1-04-01-string-vs-str --example 02-literals-and-conversions

fn main() {
    // A literal is not allocated at run time. It is baked into the executable
    // and this is a view of it.
    let literal: &str = "سلام";
    println!("literal     = {literal}");
    println!("literal len = {} bytes", literal.len());
    println!("literal   @ {:p}", literal.as_ptr());
    println!();

    // Three ways up, from a view to an owner. Every one of them asks the
    // allocator for a buffer and copies the bytes into it — three addresses,
    // none of them the literal's.
    let a: String = literal.to_string();
    let b: String = String::from(literal);
    let c: String = literal.to_owned();

    println!("to_string()   @ {:p}", a.as_ptr());
    println!("String::from  @ {:p}", b.as_ptr());
    println!("to_owned()    @ {:p}", c.as_ptr());
    println!("all three equal? {}", a == b && b == c);
    println!();

    // Two ways down, from an owner to a view. Neither allocates: both are
    // the owner's own address handed back with a length beside it.
    let view_one: &str = a.as_str();
    // clippy would rather you wrote `&a` here — and it is right, because the
    // compiler inserts this very `*` for you. Spelled out on purpose: this is
    // the thing deref coercion does behind your back.
    #[allow(clippy::explicit_auto_deref)]
    let view_two: &str = &*a;

    println!("a           @ {:p}", a.as_ptr());
    println!("a.as_str()  @ {:p}", view_one.as_ptr());
    println!("&*a         @ {:p}", view_two.as_ptr());
    println!();

    // Going up is the expensive direction. Going down is free.
    println!("up:   allocate + copy {} bytes", literal.len());
    println!("down: copy two words onto the stack");
}
