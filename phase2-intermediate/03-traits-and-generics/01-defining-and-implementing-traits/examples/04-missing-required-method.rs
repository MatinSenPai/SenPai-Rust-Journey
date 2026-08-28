//! DELIBERATELY BROKEN — expected: E0046
//! Run `cargo run --example 04-missing-required-method --features broken`
//! and read the error.

trait Summarize {
    fn title(&self) -> String;

    fn summary(&self) -> String {
        format!("{} (no summary available)", self.title())
    }
}

struct LightNovel {
    title: String,
}

impl Summarize for LightNovel {}

fn main() {
    let book = LightNovel {
        title: "Re:Zero".to_string(),
    };
    println!("{}", book.summary());
}
