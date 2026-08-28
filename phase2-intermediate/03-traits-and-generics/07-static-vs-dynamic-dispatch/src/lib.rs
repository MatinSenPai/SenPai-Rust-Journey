/// A minimal version of the `Summarize` trait from lesson 02. This crate is
/// standalone, so it can't import that one — redefined here on purpose.
pub trait Summarize {
    fn summary(&self) -> String;
}

pub struct AnimeSeries {
    pub title: String,
    pub episodes: u32,
}

impl Summarize for AnimeSeries {
    fn summary(&self) -> String {
        todo!("format! self.title and self.episodes, e.g. \"{{title}} - {{episodes}} episodes\"")
    }
}

pub struct MangaVolume {
    pub title: String,
    pub chapters: u32,
}

impl Summarize for MangaVolume {
    fn summary(&self) -> String {
        todo!("format! self.title and self.chapters, e.g. \"{{title}} - {{chapters}} chapters\"")
    }
}

/// Static dispatch: `T` is fixed to one concrete type at each call site.
/// The compiler monomorphizes this — a separate compiled version per `T`
/// actually used — exactly like `largest<T>` in lesson 01.
pub fn total_summary_length_generic<T: Summarize>(items: &[T]) -> usize {
    todo!("items.iter().map(|i| i.summary().len()).sum()")
}

/// Dynamic dispatch: every element is a `Box<dyn Summarize>` — a
/// heap-allocated value of *some* type implementing `Summarize`, with the
/// concrete type erased. Calling `.summary()` looks up the right function
/// pointer in that value's vtable at runtime.
pub fn total_summary_length_dyn(items: &[Box<dyn Summarize>]) -> usize {
    todo!("same idea as total_summary_length_generic, but over trait objects")
}

/// Builds a `Vec` containing a genuine mix of `AnimeSeries` and
/// `MangaVolume`, boxed as trait objects. This is the thing a generic
/// `Vec<T>` could never express — `T` has to be one fixed type per `Vec`.
pub fn make_mixed_collection() -> Vec<Box<dyn Summarize>> {
    todo!(
        "vec![Box::new(AnimeSeries {{ ... }}), Box::new(MangaVolume {{ ... }})] as Vec<Box<dyn Summarize>>"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anime_series_summarizes() {
        let a = AnimeSeries {
            title: "Trigun".to_string(),
            episodes: 26,
        };
        assert_eq!(a.summary(), "Trigun - 26 episodes");
    }

    #[test]
    fn manga_volume_summarizes() {
        let m = MangaVolume {
            title: "Blame!".to_string(),
            chapters: 10,
        };
        assert_eq!(m.summary(), "Blame! - 10 chapters");
    }

    #[test]
    fn generic_total_matches_manual_sum_for_one_concrete_type() {
        let series = vec![
            AnimeSeries {
                title: "Trigun".to_string(),
                episodes: 26,
            },
            AnimeSeries {
                title: "Baccano!".to_string(),
                episodes: 13,
            },
        ];
        let expected: usize = series.iter().map(|s| s.summary().len()).sum();
        assert_eq!(total_summary_length_generic(&series), expected);
    }

    #[test]
    fn dyn_total_matches_manual_sum_over_boxed_trait_objects() {
        let items: Vec<Box<dyn Summarize>> = vec![
            Box::new(AnimeSeries {
                title: "Trigun".to_string(),
                episodes: 26,
            }),
            Box::new(MangaVolume {
                title: "Blame!".to_string(),
                chapters: 10,
            }),
        ];
        let expected: usize = items.iter().map(|i| i.summary().len()).sum();
        assert_eq!(total_summary_length_dyn(&items), expected);
    }

    #[test]
    fn mixed_collection_contains_both_concrete_types() {
        let mixed = make_mixed_collection();
        assert_eq!(mixed.len(), 2);
        // We can't downcast to check exact concrete types without `Any`,
        // but we CAN confirm both summaries are present and distinct,
        // proving this Vec really does hold two different kinds of value.
        let summaries: Vec<String> = mixed.iter().map(|i| i.summary()).collect();
        assert!(summaries[0].contains("episodes"));
        assert!(summaries[1].contains("chapters"));
    }
}
