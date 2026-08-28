//! Dynamic dispatch: `announce_dyn` takes `&dyn Summarize` instead of a
//! generic `T`. There is only ever ONE compiled copy of this function — its
//! signature has no `<T>` to instantiate — and it still handles both
//! concrete types below, one right after the other.
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 02-dynamic-one-function-many-types

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

fn announce_dyn(item: &dyn Summarize) -> String {
    format!("now: {}", item.summary())
}

fn main() {
    let anime = AnimeSeries {
        title: "Trigun".to_string(),
        episodes: 26,
    };
    let manga = MangaVolume {
        title: "Blame!".to_string(),
        chapters: 10,
    };

    // Same `announce_dyn`, no `::<...>` turbofish anywhere — there is
    // nothing to instantiate. At each call, the vtable behind the
    // reference is what decides which `summary` actually runs.
    println!("{}", announce_dyn(&anime));
    println!("{}", announce_dyn(&manga));
}
