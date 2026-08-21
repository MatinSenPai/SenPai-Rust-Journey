//! Statements do something. Expressions are worth something.
//!
//!     cargo run -p p1-01-04-functions-and-expressions --example 02-expressions

fn main() {
    // `let total = ...;` is a statement. It performs a binding and is worth
    // nothing. `3 * 4` on its right is an expression: it is worth 12.
    let total = 3 * 4;
    println!("total:     {total}");

    // A block is an expression too. Its value is its last expression — the
    // one without a semicolon.
    let adjusted = {
        let base = 100;
        let bonus = 20;
        base + bonus
    };
    println!("adjusted:  {adjusted}");

    // Put a semicolon on that last line and the block is worth `()` instead.
    // Running this prints a warning — read it, it is the lesson in one line:
    // "the arithmetic operation produces a value". You computed something and
    // then threw it away, and the compiler noticed.
    let nothing = {
        let base = 100;
        let bonus = 20;
        base + bonus;
    };
    println!("nothing:   {nothing:?}");

    // Which is exactly what happens in a function body. These two are the
    // same function written two ways.
    println!("implicit:  {}", with_tail_expression(10));
    println!("explicit:  {}", with_return(10));

    // Blocks nest, and each one is worth its own last expression.
    let nested = {
        let inner = {
            let a = 2;
            a * 3
        };
        inner + 1
    };
    println!("nested:    {nested}");

    // Operators are expressions, and `&&` and `||` short-circuit: if the left
    // side settles the answer, the right side is never evaluated.
    let year = 2024;
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    println!("2024 leap: {leap}");

    // Note what that line did *not* need: any `if` at all. A decision is an
    // expression, and expressions are values.
}

fn with_tail_expression(n: u32) -> u32 {
    n * 3
}

fn with_return(n: u32) -> u32 {
    return n * 3;
}
