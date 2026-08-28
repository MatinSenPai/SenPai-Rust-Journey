//! The count goes up on every `Rc::clone` and down on every drop. The
//! `AppConfig` underneath is only actually freed once the count reaches
//! zero — watch `Drop` fire exactly once, on the very last owner.
//!
//!     cargo run -p p2-06-03-rc-and-arc --example 02-count-and-drop-order

use std::rc::Rc;

struct AppConfig {
    app_name: String,
}

impl Drop for AppConfig {
    fn drop(&mut self) {
        println!("  {} actually freed now", self.app_name);
    }
}

fn main() {
    let a = Rc::new(AppConfig {
        app_name: "senpai-api".to_string(),
    });
    println!("after Rc::new:      count = {}", Rc::strong_count(&a));

    let b = Rc::clone(&a);
    println!("after first clone:  count = {}", Rc::strong_count(&a));

    let c = Rc::clone(&a);
    println!("after second clone: count = {}", Rc::strong_count(&a));

    drop(b);
    println!("after dropping one: count = {}", Rc::strong_count(&a));

    drop(c);
    println!("after dropping two: count = {}", Rc::strong_count(&a));

    println!("dropping the last owner:");
    drop(a);
}
