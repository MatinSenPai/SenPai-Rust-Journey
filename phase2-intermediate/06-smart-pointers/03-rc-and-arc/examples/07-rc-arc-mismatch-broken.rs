//! DELIBERATELY BROKEN — expected: E0308.
//!
//! `Rc<T>` and `Arc<T>` share an API but are not the same type, and Rust
//! never converts one into the other for you — matching method names is
//! not the same thing as being interchangeable.
//!
//!     cargo run -p p2-06-03-rc-and-arc --example 07-rc-arc-mismatch-broken --features broken

use std::rc::Rc;
use std::sync::Arc;

struct AppConfig {
    app_name: String,
}

fn spawn_worker(config: Arc<AppConfig>) {
    println!("worker sees: {}", config.app_name);
}

fn main() {
    let config = Rc::new(AppConfig {
        app_name: "senpai-api".to_string(),
    });

    spawn_worker(config);
}
