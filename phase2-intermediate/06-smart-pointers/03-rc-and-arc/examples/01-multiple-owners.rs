//! `Rc<T>` lets more than one variable be a real, simultaneous owner of the
//! same heap value. Try this with `Box<T>` instead and the second `let`
//! would move the value away from the first — this is genuinely different.
//!
//!     cargo run -p p2-06-03-rc-and-arc --example 01-multiple-owners

use std::rc::Rc;

struct AppConfig {
    app_name: String,
    max_connections: u32,
}

fn main() {
    let config = Rc::new(AppConfig {
        app_name: "senpai-api".to_string(),
        max_connections: 100,
    });

    let for_handler = Rc::clone(&config);
    let for_logger = Rc::clone(&config);

    println!("handler sees:           {}", for_handler.app_name);
    println!("logger sees:            {}", for_logger.app_name);
    println!("original still usable:  {}", config.app_name);
    println!("owners right now:       {}", Rc::strong_count(&config));
    println!("max_connections too:    {}", for_handler.max_connections);
}
