//! DELIBERATELY BROKEN — expected: E0038
//! Run `cargo run -p p2-09-04-async-traits-and-blocking --example 05-dyn-native-async-trait-broken --features broken`
//! and read the error.
//!
//! Same trait and impl as `01`, no `#[async_trait]` anywhere. Boxing it as
//! `dyn Fetcher` is exactly the one thing a native `async fn` in a trait
//! cannot do.

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
