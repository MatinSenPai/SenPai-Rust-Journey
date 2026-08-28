//! DELIBERATELY BROKEN — expected: E0782.
//!
//! `&Summarize` reads like "a reference to the trait itself." Rust 2021
//! requires the `dyn` keyword to say what is actually meant: "a reference
//! to some value of an unknown type that implements this trait."
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 07-missing-dyn-keyword-broken --features broken

trait Summarize {
    fn summary(&self) -> String;
}

struct AnimeSeries {
    title: String,
}

impl Summarize for AnimeSeries {
    fn summary(&self) -> String {
        self.title.clone()
    }
}

fn announce(item: &Summarize) -> String {
    format!("now: {}", item.summary())
}

fn main() {
    let anime = AnimeSeries {
        title: "Trigun".to_string(),
    };
    println!("{}", announce(&anime));
}
