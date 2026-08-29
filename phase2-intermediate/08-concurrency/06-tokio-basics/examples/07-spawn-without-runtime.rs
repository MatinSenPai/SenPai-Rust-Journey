//! DELIBERATELY BROKEN — expected: a run-time panic, "there is no reactor
//! running, must be called from the context of a Tokio 1.x runtime".
//! Run `cargo run -p p2-08-06-tokio-basics --example 07-spawn-without-runtime --features broken`
//! and read the panic.
//!
//! This compiles cleanly — `tokio::spawn` is an ordinary function, not an
//! `async fn`, so calling it needs no `.await` and no `async` context. But
//! it still needs a runtime to hand the task TO, and `fn main` here never
//! built one. `#[tokio::main]`'s whole job is building that runtime first.

fn main() {
    tokio::spawn(async {
        println!("hi");
    });
    println!("spawned");
}
