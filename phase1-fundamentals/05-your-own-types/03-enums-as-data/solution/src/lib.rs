pub enum Status {
    Ongoing { latest_chapter: u32 },
    Hiatus { since_chapter: u32 },
    Completed { total_chapters: u32 },
    Cancelled,
}

pub fn describe(status: &Status) -> String {
    match status {
        Status::Ongoing { latest_chapter } => format!("ongoing, latest chapter {latest_chapter}"),
        Status::Hiatus { since_chapter } => format!("on hiatus since chapter {since_chapter}"),
        Status::Completed { total_chapters } => format!("completed, {total_chapters} chapters"),
        Status::Cancelled => "cancelled".to_string(),
    }
}

pub fn is_readable(status: &Status) -> bool {
    !matches!(status, Status::Cancelled)
}

pub fn latest_available_chapter(status: &Status) -> u32 {
    match status {
        Status::Ongoing { latest_chapter } => *latest_chapter,
        Status::Hiatus { since_chapter } => *since_chapter,
        Status::Completed { total_chapters } => *total_chapters,
        Status::Cancelled => 0,
    }
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
