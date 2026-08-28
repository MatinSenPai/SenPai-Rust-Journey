//! DELIBERATELY BROKEN — expected: E0599
//! Run `cargo run --example 05-trait-not-in-scope --features broken`
//! and read the error.

mod catalog {
    pub trait Summarize {
        fn summary(&self) -> String;
    }

    pub struct AnimeSeries {
        pub title: String,
    }

    impl Summarize for AnimeSeries {
        fn summary(&self) -> String {
            self.title.clone()
        }
    }
}

fn main() {
    let death_note = catalog::AnimeSeries {
        title: "Death Note".to_string(),
    };
    println!("{}", death_note.summary());
}
