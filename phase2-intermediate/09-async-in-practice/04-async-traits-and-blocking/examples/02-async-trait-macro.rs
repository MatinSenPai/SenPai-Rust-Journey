//! The same trait and impl as `01`, both wearing `#[async_trait]`. That one
//! attribute is what makes `Box<dyn Fetcher>` legal — see `05` for what
//! happens without it.
//!
//!     cargo run -p p2-09-04-async-traits-and-blocking --example 02-async-trait-macro

use async_trait::async_trait;

#[async_trait]
trait Fetcher {
    async fn fetch(&self) -> String;
}

struct Server;

#[async_trait]
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
