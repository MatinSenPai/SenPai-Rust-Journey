//! `.clone()` on an `Rc<T>` and `.clone()` on the `T` it wraps look
//! identical but do opposite things. Prove it with addresses: cloning the
//! `Rc` keeps the same allocation; cloning the `AppConfig` itself makes a
//! genuinely new one.
//!
//!     cargo run -p p2-06-03-rc-and-arc --example 03-clone-cost-comparison

use std::rc::Rc;

#[derive(Clone)]
struct AppConfig {
    app_name: String,
}

fn main() {
    let original = Rc::new(AppConfig {
        app_name: "senpai-api".to_string(),
    });
    let rc_clone = Rc::clone(&original);
    let data_clone: AppConfig = (*original).clone();

    println!(
        "Rc::as_ptr(&original) == Rc::as_ptr(&rc_clone):             {}",
        Rc::as_ptr(&original) == Rc::as_ptr(&rc_clone)
    );
    println!(
        "original.app_name.as_ptr() == data_clone.app_name.as_ptr(): {}",
        original.app_name.as_ptr() == data_clone.app_name.as_ptr()
    );
}
