pub struct Book {
    pub title: String,
    pub chapters_read: u32,
    pub total_chapters: u32,
    pub is_favorite: bool,
}

impl Book {
    /// Constructs a new `Book` with 0 chapters read and `is_favorite: false`.
    pub fn new(title: &str, total_chapters: u32) -> Self {
        todo!("build a Book from the args, chapters_read: 0, is_favorite: false")
    }

    /// Advances `chapters_read` by 1, unless already at `total_chapters`.
    /// Returns `true` if it advanced, `false` if it was already caught up.
    pub fn read_chapter(&mut self) -> bool {
        todo!("if self.chapters_read < self.total_chapters, increment and return true, else false")
    }

    /// Sets `is_favorite` to `true`.
    pub fn mark_favorite(&mut self) {
        todo!("self.is_favorite = true")
    }

    /// Returns `chapters_read / total_chapters * 100.0` as a percentage.
    pub fn progress_percent(&self) -> f64 {
        todo!("cast both to f64, divide, multiply by 100.0")
    }

    /// Returns e.g. `"One Piece (5/10) *"` if favorite, or
    /// `"One Piece (5/10)"` if not (no trailing space when not a favorite).
    pub fn describe(&self) -> String {
        todo!("format! title, chapters_read/total_chapters, and a trailing \" *\" only if is_favorite")
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
        assert!(!b.read_chapter()); // already caught up
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
