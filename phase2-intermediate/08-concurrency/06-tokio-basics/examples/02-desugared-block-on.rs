//! The exact same program as 01, written by hand with no macro at all —
//! this is what `#[tokio::main]` actually expands to underneath.
//! `Runtime::new()` IS `Builder::new_multi_thread().enable_all().build()`;
//! `#[tokio::main]` builds that same runtime and calls `.block_on(...)`.
//!
//!     cargo run -p p2-08-06-tokio-basics --example 02-desugared-block-on

use std::time::Duration;

fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        println!("done after a real 50ms sleep");
    });
}
