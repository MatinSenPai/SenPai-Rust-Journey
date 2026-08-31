//! DELIBERATELY BROKEN — expected: E0308
//! Run `cargo run --example 05-forgot-to-await-broken --features broken`
//! and read the error.

async fn fetch(id: u32) -> u32 {
    id * 2
}

async fn describe(id: u32) -> String {
    let value: u32 = fetch(id);
    format!("value: {value}")
}

fn main() {
    println!("compiled");
}
