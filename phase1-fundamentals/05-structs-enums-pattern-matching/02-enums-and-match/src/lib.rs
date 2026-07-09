pub enum Status {
    Ongoing { latest_chapter: u32 },
    Hiatus { since_chapter: u32 },
    Completed { total_chapters: u32 },
    Cancelled,
}

/// Describes `status` in one sentence.
///
/// - `Ongoing { latest_chapter }` -> `"ongoing, latest chapter 42"`
/// - `Hiatus { since_chapter }` -> `"on hiatus since chapter 12"`
/// - `Completed { total_chapters }` -> `"completed, 100 chapters"`
/// - `Cancelled` -> `"cancelled"`
pub fn describe(status: &Status) -> String {
    todo!("match status, destructuring each variant's data into the format! calls above")
}

/// `Cancelled` is not readable; every other variant is.
pub fn is_readable(status: &Status) -> bool {
    todo!("match status {{ Status::Cancelled => false, _ => true }}")
}

/// Returns the highest chapter number currently available: `latest_chapter`
/// for `Ongoing`, `since_chapter` for `Hiatus`, `total_chapters` for
/// `Completed`, and `0` for `Cancelled`.
pub fn latest_available_chapter(status: &Status) -> u32 {
    todo!("match status, extracting the right field per variant, 0 for Cancelled")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describes_each_variant() {
        assert_eq!(
            describe(&Status::Ongoing { latest_chapter: 42 }),
            "ongoing, latest chapter 42"
        );
        assert_eq!(
            describe(&Status::Hiatus { since_chapter: 12 }),
            "on hiatus since chapter 12"
        );
        assert_eq!(
            describe(&Status::Completed {
                total_chapters: 100
            }),
            "completed, 100 chapters"
        );
        assert_eq!(describe(&Status::Cancelled), "cancelled");
    }

    #[test]
    fn only_cancelled_is_unreadable() {
        assert!(is_readable(&Status::Ongoing { latest_chapter: 1 }));
        assert!(is_readable(&Status::Hiatus { since_chapter: 1 }));
        assert!(is_readable(&Status::Completed { total_chapters: 1 }));
        assert!(!is_readable(&Status::Cancelled));
    }

    #[test]
    fn finds_latest_available_chapter() {
        assert_eq!(
            latest_available_chapter(&Status::Ongoing { latest_chapter: 42 }),
            42
        );
        assert_eq!(
            latest_available_chapter(&Status::Hiatus { since_chapter: 12 }),
            12
        );
        assert_eq!(
            latest_available_chapter(&Status::Completed {
                total_chapters: 100
            }),
            100
        );
        assert_eq!(latest_available_chapter(&Status::Cancelled), 0);
    }
}
