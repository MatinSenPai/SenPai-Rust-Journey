//! Handing a value to a function is a move, exactly like an assignment.
//!
//!     cargo run -p p1-02-04-ownership-across-functions --example 01-passing-moves

fn main() {
    let name = String::from("Matin");

    // `consume` takes a String by value, so this call moves `name` into it.
    let length = consume(name);
    println!("length:     {length}");

    // `name` is gone here. Uncomment for E0382, or see 04-used-after-passing.
    // println!("{name}");

    // Numbers are different, for the same reason as always: nothing to own.
    let count = 5_i32;
    let doubled = double(count);
    println!("count:      {count}");
    println!("doubled:    {doubled}");

    // A function can also *give* ownership. This is where new values come
    // from, and there is nothing special about it.
    let built = build();
    println!("built:      {built}");
    println!("built   @:  {:p}", built.as_ptr());

    // And it can do both: take one in and hand a different one back.
    let shouted = shout(built);
    println!("shouted:    {shouted}");

    // A value passed in and not returned is dropped when the function ends.
    let temporary = String::from("this will not survive the call");
    consume(temporary);
    println!("that String was freed inside `consume`");
}

fn consume(text: String) -> usize {
    text.len()
} // <- `text` is dropped here. It was ours.

fn double(n: i32) -> i32 {
    n * 2
}

fn build() -> String {
    String::from("made inside build()")
}

fn shout(text: String) -> String {
    text.to_uppercase()
}
