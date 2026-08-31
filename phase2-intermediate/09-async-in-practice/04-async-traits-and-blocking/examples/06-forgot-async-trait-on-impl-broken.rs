//! DELIBERATELY BROKEN — expected: E0195
//! Run `cargo run -p p2-09-04-async-traits-and-blocking --example 06-forgot-async-trait-on-impl-broken --features broken`
//! and read the error.
//!
//! `#[async_trait]` rewrites the trait's `fetch` signature into something
//! that returns a boxed future — but only where the attribute is actually
//! written. This trait has it; this impl forgot it, so the two signatures
//! no longer match.

use async_trait::async_trait;

#[async_trait]
trait Fetcher {
    async fn fetch(&self) -> String;
}

struct Server;

impl Fetcher for Server {
    async fn fetch(&self) -> String {
        "data from Server".to_string()
    }
}

#[tokio::main]
async fn main() {
    let f: Box<dyn Fetcher> = Box::new(Server);
    println!("{}", f.fetch().await);
}
