//! `AnimeSeries` is fully public, but its `.summary()` method only becomes
//! callable once `Summarize` itself is imported.

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

use catalog::Summarize;

fn main() {
    let death_note = catalog::AnimeSeries {
        title: "Death Note".to_string(),
    };
    println!("{}", death_note.summary());
}
