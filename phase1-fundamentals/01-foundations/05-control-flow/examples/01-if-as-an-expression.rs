//! `if` is an expression, so it is worth something.
//!
//!     cargo run -p p1-01-05-control-flow --example 01-if-as-an-expression

fn main() {
    let score = 73;

    // The familiar shape first. No brackets round the condition — Rust does
    // not want them and clippy will tell you so.
    if score >= 90 {
        println!("grade:     A");
    } else if score >= 70 {
        println!("grade:     B");
    } else {
        println!("grade:     C");
    }

    // Now the part Python needs separate syntax for. The whole `if` is an
    // expression, so it goes straight into a `let`.
    let grade = if score >= 90 {
        'A'
    } else if score >= 70 {
        'B'
    } else {
        'C'
    };
    println!("grade:     {grade}");

    // Every branch has to be worth the same type, because the binding gets
    // exactly one type. Both arms here are `char`.

    // Used as an expression, the `else` is compulsory: without it there would
    // be no value when the condition is false.
    let capped = if score > 100 { 100 } else { score };
    println!("capped:    {capped}");

    // The condition must be a `bool`. Not a number, not a string, not an
    // empty collection. There is no truthiness in Rust, and 04-truthiness
    // shows what happens when you try.
    let stock = 0;
    println!("is zero:   {}", stock == 0);

    // A block already produces a value, so this reads perfectly naturally
    // once you are used to it.
    let message = if stock == 0 { "sold out" } else { "in stock" };
    println!("message:   {message}");
}
