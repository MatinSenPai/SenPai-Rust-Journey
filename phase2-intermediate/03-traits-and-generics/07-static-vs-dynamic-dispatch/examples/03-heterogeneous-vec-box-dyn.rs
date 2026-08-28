//! The thing a generic `Vec<T>` can never do: hold a genuine mix of
//! concrete types in one collection. `Box<dyn Summarize>` erases each
//! element down to "some type implementing `Summarize`, plus a vtable" —
//! and a `Vec` of that one erased type is a perfectly ordinary `Vec`.
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 03-heterogeneous-vec-box-dyn

trait Summarize {
    fn summary(&self) -> String;
}

struct AnimeSeries {
    title: String,
    episodes: u32,
}

impl Summarize for AnimeSeries {
    fn summary(&self) -> String {
        format!("{} - {} episodes", self.title, self.episodes)
    }
}

struct MangaVolume {
    title: String,
    chapters: u32,
}

impl Summarize for MangaVolume {
    fn summary(&self) -> String {
        format!("{} - {} chapters", self.title, self.chapters)
    }
}

fn main() {
    let lineup: Vec<Box<dyn Summarize>> = vec![
        Box::new(AnimeSeries {
            title: "Trigun".to_string(),
            episodes: 26,
        }),
        Box::new(MangaVolume {
            title: "Blame!".to_string(),
            chapters: 10,
        }),
    ];

    // One `Vec`, one element type (`Box<dyn Summarize>`), two genuinely
    // different structs underneath.
    for item in &lineup {
        println!("{}", item.summary());
    }
    println!("lineup holds {} items", lineup.len());
}
