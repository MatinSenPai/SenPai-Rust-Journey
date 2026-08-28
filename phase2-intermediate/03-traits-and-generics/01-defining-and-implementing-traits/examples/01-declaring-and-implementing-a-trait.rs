//! Declare a trait with one required method and one default method, then
//! implement it for a single type.

trait Summarize {
    fn title(&self) -> String;

    fn summary(&self) -> String {
        format!("{} (no summary available)", self.title())
    }
}

struct AnimeSeries {
    title: String,
    episodes: u32,
}

impl Summarize for AnimeSeries {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {} episodes", self.title(), self.episodes)
    }
}

fn main() {
    let death_note = AnimeSeries {
        title: "Death Note".to_string(),
        episodes: 37,
    };
    println!("{}", death_note.title());
    println!("{}", death_note.summary());
}
