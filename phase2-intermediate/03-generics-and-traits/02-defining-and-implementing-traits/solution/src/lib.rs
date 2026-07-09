/// A trait for things that can describe themselves.
pub trait Summarize {
    /// A short title for this item. Every implementor must provide this —
    /// there's no default body, so `impl Summarize for X` won't compile
    /// until `X` supplies its own `title`.
    fn title(&self) -> String;

    /// A one-line summary. This has a default body, so implementors get it
    /// for free just by implementing `title` — or they can override it with
    /// something more specific, as `AnimeSeries` does below.
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
        format!("{} - {} episodes", self.title, self.episodes)
    }
}

pub struct MangaVolume {
    pub title: String,
}

impl Summarize for MangaVolume {
    fn title(&self) -> String {
        self.title.clone()
    }

    // No `summary` override here — on purpose. MangaVolume relies entirely
    // on Summarize's default implementation.
}

/// Calls `.summary()` on every item in `items` and collects the results.
/// Bounded by our own trait, `Summarize`, exactly like `largest` in lesson
/// 01 was bounded by the standard library's `PartialOrd`.
pub fn print_all_summaries<T: Summarize>(items: &[T]) -> Vec<String> {
    items.iter().map(|item| item.summary()).collect()
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
        assert_eq!(ds.summary(), "Death Note - 37 episodes");
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
    fn print_all_summaries_collects_each_items_summary() {
        let series = vec![
            AnimeSeries {
                title: "Cowboy Bebop".to_string(),
                episodes: 26,
            },
            AnimeSeries {
                title: "FLCL".to_string(),
                episodes: 6,
            },
        ];
        assert_eq!(
            print_all_summaries(&series),
            vec![
                "Cowboy Bebop - 26 episodes".to_string(),
                "FLCL - 6 episodes".to_string(),
            ]
        );
    }

    #[test]
    fn print_all_summaries_works_for_manga_volumes_too() {
        let volumes = vec![MangaVolume {
            title: "Vagabond Vol. 3".to_string(),
        }];
        assert_eq!(
            print_all_summaries(&volumes),
            vec!["Vagabond Vol. 3 (no summary available)".to_string()]
        );
    }
}
