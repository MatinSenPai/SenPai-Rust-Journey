pub struct Book {
    pub title: String,
    pub chapters_read: u32,
    pub total_chapters: u32,
    pub is_favorite: bool,
}

impl Book {
    pub fn new(title: &str, total_chapters: u32) -> Self {
        Book {
            title: title.to_string(),
            chapters_read: 0,
            total_chapters,
            is_favorite: false,
        }
    }

    pub fn read_chapter(&mut self) -> bool {
        if self.chapters_read < self.total_chapters {
            self.chapters_read += 1;
            true
        } else {
            false
        }
    }

    pub fn mark_favorite(&mut self) {
        self.is_favorite = true;
    }

    pub fn progress_percent(&self) -> f64 {
        self.chapters_read as f64 / self.total_chapters as f64 * 100.0
    }

    pub fn describe(&self) -> String {
        let star = if self.is_favorite { " *" } else { "" };
        format!(
            "{} ({}/{}){}",
            self.title, self.chapters_read, self.total_chapters, star
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_with_defaults() {
        let b = Book::new("One Piece", 1100);
        assert_eq!(b.title, "One Piece");
        assert_eq!(b.chapters_read, 0);
        assert_eq!(b.total_chapters, 1100);
        assert!(!b.is_favorite);
    }

    #[test]
    fn reads_chapters_up_to_total() {
        let mut b = Book::new("Short One", 2);
        assert!(b.read_chapter());
        assert!(b.read_chapter());
        assert!(!b.read_chapter());
        assert_eq!(b.chapters_read, 2);
    }

    #[test]
    fn marks_favorite() {
        let mut b = Book::new("Berserk", 400);
        assert!(!b.is_favorite);
        b.mark_favorite();
        assert!(b.is_favorite);
    }

    #[test]
    fn computes_progress() {
        let mut b = Book::new("Vinland Saga", 4);
        b.read_chapter();
        assert_eq!(b.progress_percent(), 25.0);
    }

    #[test]
    fn describes_with_and_without_star() {
        let mut b = Book::new("Chainsaw Man", 10);
        b.read_chapter();
        b.read_chapter();
        assert_eq!(b.describe(), "Chainsaw Man (2/10)");
        b.mark_favorite();
        assert_eq!(b.describe(), "Chainsaw Man (2/10) *");
    }
}
