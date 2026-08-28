//! DELIBERATELY BROKEN — expected: E0382.
//!
//! An `Rc<T>` is still an ordinary owned value. Handing the *same* `Rc` to
//! two functions that each take ownership moves it away the first time,
//! exactly like any other non-`Copy` value — `Rc::clone` is what you meant.
//!
//!     cargo run -p p2-06-03-rc-and-arc --example 05-forgot-to-clone-broken --features broken

use std::rc::Rc;

struct AppConfig {
    app_name: String,
}

fn announce(config: Rc<AppConfig>) {
    println!("announcing: {}", config.app_name);
}

fn log_startup(config: Rc<AppConfig>) {
    println!("logging: {}", config.app_name);
}

fn main() {
    let config = Rc::new(AppConfig {
        app_name: "senpai-api".to_string(),
    });

    announce(config);
    log_startup(config);
}
