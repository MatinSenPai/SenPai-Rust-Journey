//! DELIBERATELY BROKEN — expected: E0596.
//!
//! `Rc<T>` implements `Deref` but not `DerefMut` — it only ever hands out
//! `&T`. Calling a method that needs `&mut self` through a shared `Rc`
//! cannot borrow-check, no matter how many (or how few) owners exist.
//!
//!     cargo run -p p2-06-03-rc-and-arc --example 06-cannot-mutate-through-rc-broken --features broken

use std::rc::Rc;

struct AppConfig {
    max_connections: u32,
}

impl AppConfig {
    fn raise_limit(&mut self, by: u32) {
        self.max_connections += by;
    }
}

fn main() {
    let config = Rc::new(AppConfig {
        max_connections: 100,
    });

    config.raise_limit(50);
    println!("max_connections: {}", config.max_connections);
}
