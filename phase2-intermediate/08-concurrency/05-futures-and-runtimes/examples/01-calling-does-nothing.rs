//! Calling an `async fn` does not run its body — only *polling* it does.
//! Nothing here polls `greeting`, so watch both what `cargo` warns about
//! and what does (and doesn't) get printed.

async fn greet(name: &str) -> String {
    println!("  (inside greet: actually running now)");
    format!("hello, {name}")
}

fn main() {
    println!("calling greet(\"Matin\")...");
    greet("Matin");
    println!("...called it. did \"inside greet\" print above?");
}
