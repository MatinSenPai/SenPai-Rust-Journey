//! Solution for 2.3.1 — defining and implementing traits.

/// A trait for things that can describe themselves in a catalog.
pub trait Summarize {
    /// A short title for this item. Every implementor must provide this —
    /// there is no default body, so `impl Summarize for X` will not compile
    /// until `X` supplies its own `title`.
    fn title(&self) -> String;

    /// A one-line summary. This has a default body, so implementors get it
    /// for free just by implementing `title` — or they can override it with
    /// something more specific, as `AnimeSeries` and `GameTitle` do below.
    fn summary(&self) -> String {
        format!("{} (no summary available)", self.title())
    }
}

pub struct AnimeSeries {
    pub title: String,
    pub episodes: u32,
}

impl Summarize for AnimeSeries {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {} episodes", self.title(), self.episodes)
    }
}

pub struct MangaVolume {
    pub title: String,
}

impl Summarize for MangaVolume {
    fn title(&self) -> String {
        self.title.clone()
    }

    // No `summary` override here, on purpose: MangaVolume relies entirely
    // on Summarize's default implementation.
}

pub struct GameTitle {
    pub title: String,
    pub hours_to_beat: u32,
}

impl Summarize for GameTitle {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {}h to beat", self.title(), self.hours_to_beat)
    }
}

/// `series`'s summary and `volume`'s summary, joined by `"; "`.
pub fn shelf_summary(series: &AnimeSeries, volume: &MangaVolume) -> String {
    format!("{}; {}", series.summary(), volume.summary())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anime_series_uses_its_own_summary() {
        let ds = AnimeSeries {
            title: "Death Note".to_string(),
            episodes: 37,
        };
        assert_eq!(ds.title(), "Death Note");
        assert_eq!(ds.summary(), "Death Note — 37 episodes");
    }

    #[test]
    fn manga_volume_uses_the_default_summary() {
        let vol = MangaVolume {
            title: "Berserk Vol. 1".to_string(),
        };
        assert_eq!(vol.title(), "Berserk Vol. 1");
        assert_eq!(vol.summary(), "Berserk Vol. 1 (no summary available)");
    }

    #[test]
    fn game_title_uses_its_own_summary() {
        let g = GameTitle {
            title: "Elden Ring".to_string(),
            hours_to_beat: 60,
        };
        assert_eq!(g.title(), "Elden Ring");
        assert_eq!(g.summary(), "Elden Ring — 60h to beat");
    }

    #[test]
    fn shelf_summary_joins_both_summaries_with_a_semicolon() {
        let series = AnimeSeries {
            title: "Death Note".to_string(),
            episodes: 37,
        };
        let volume = MangaVolume {
            title: "Berserk Vol. 1".to_string(),
        };
        assert_eq!(
            shelf_summary(&series, &volume),
            "Death Note — 37 episodes; Berserk Vol. 1 (no summary available)"
        );
    }
}
