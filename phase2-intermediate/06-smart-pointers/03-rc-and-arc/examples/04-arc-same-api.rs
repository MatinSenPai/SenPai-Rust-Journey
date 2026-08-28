//! `Arc<T>` is the thread-safe sibling of `Rc<T>` — same shape, same method
//! names, just an atomic counter under the hood. Nothing here actually
//! crosses a thread; that mechanic is module 8's job. Today is only about
//! recognizing that the API is the one you already know.
//!
//!     cargo run -p p2-06-03-rc-and-arc --example 04-arc-same-api

use std::sync::Arc;

struct AppConfig {
    app_name: String,
    max_connections: u32,
}

fn main() {
    let config = Arc::new(AppConfig {
        app_name: "senpai-api".to_string(),
        max_connections: 100,
    });

    let for_handler = Arc::clone(&config);
    println!("handler sees:     {}", for_handler.app_name);
    println!("owners right now: {}", Arc::strong_count(&config));
    println!("max_connections:  {}", config.max_connections);
}
