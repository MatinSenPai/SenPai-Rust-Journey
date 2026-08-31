//! A trait with an `async fn` written directly — no macro, stable since
//! Rust 1.75. `print_fetch` is generic over `Fetcher`, so this is static
//! dispatch: one compiled copy per concrete type, exactly like any other
//! generic bound.
//!
//!     cargo run -p p2-09-04-async-traits-and-blocking --example 01-native-async-trait

trait Fetcher {
    async fn fetch(&self) -> String;
}

struct Server;

impl Fetcher for Server {
    async fn fetch(&self) -> String {
        "data from Server".to_string()
    }
}

async fn print_fetch<F: Fetcher>(f: &F) {
    println!("{}", f.fetch().await);
}

#[tokio::main]
async fn main() {
    print_fetch(&Server).await;
}
