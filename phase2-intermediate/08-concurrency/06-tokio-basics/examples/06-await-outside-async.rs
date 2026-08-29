//! DELIBERATELY BROKEN — expected: E0728
//! Run `cargo run -p p2-08-06-tokio-basics --example 06-await-outside-async --features broken`
//! and read the error.
//!
//! `.await` only makes sense inside an `async fn` or `async` block — it is
//! how that function yields control back to the runtime. A plain `fn` has
//! no runtime driving it and nowhere to yield to.

use std::time::Duration;

fn main() {
    tokio::time::sleep(Duration::from_millis(10)).await;
    println!("done");
}
