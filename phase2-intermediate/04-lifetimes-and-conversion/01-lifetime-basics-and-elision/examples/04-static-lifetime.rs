//! Run: cargo run -p p2-04-01-lifetime-basics-and-elision --example 04-static-lifetime
//!
//! `'static` is a lifetime like any other — just one specific, nameable
//! span: "valid for the rest of the running program." A string literal
//! always qualifies, because it is baked into the compiled binary itself,
//! not allocated at run time.

fn announce(message: &'static str) {
    println!("{message}");
}

fn main() {
    let literal: &'static str = "senpai never goes out of scope";
    announce(literal);
    announce("neither does this one");
}
