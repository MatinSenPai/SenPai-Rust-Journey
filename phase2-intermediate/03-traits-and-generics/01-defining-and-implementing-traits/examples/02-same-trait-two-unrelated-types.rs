//! The payoff: implement the same trait for a second, completely unrelated
//! type, and call the same method on both.

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

struct MangaVolume {
    title: String,
}

impl Summarize for MangaVolume {
    fn title(&self) -> String {
        self.title.clone()
    }

    // No `summary` override here, on purpose: MangaVolume relies entirely on
    // Summarize's default implementation.
}

fn main() {
    let death_note = AnimeSeries {
        title: "Death Note".to_string(),
        episodes: 37,
    };
    let berserk = MangaVolume {
        title: "Berserk Vol. 1".to_string(),
    };

    println!("{}", death_note.summary());
    println!("{}", berserk.summary());
}
