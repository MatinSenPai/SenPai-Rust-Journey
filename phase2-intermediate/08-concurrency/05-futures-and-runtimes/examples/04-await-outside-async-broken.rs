//! DELIBERATELY BROKEN — expected: E0728
//! Run `cargo run --example 04-await-outside-async-broken --features broken`
//! and read the error.

async fn fetch(id: u32) -> u32 {
    id * 2
}

fn main() {
    let result = fetch(7).await;
    println!("result = {result}");
}
