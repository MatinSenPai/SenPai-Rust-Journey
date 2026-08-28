//! Reads and writes your own checkpoint answers — one Markdown file per lesson,
//! under a gitignored `.checkpoint-answers/` directory that mirrors the
//! curriculum's own layout.
//!
//! It follows `docs/adr/0001-web-ui-progress-state.md` everywhere that matters:
//! keyed by repo-relative lesson **directory path** (a fact the filesystem
//! enforces, unlike a prose title), gitignored so a fresh clone starts empty,
//! and never pruned behind your back — a renamed lesson leaves its old answer
//! sitting on disk rather than deleting words you wrote.
//!
//! It departs from that ADR on one point: Markdown files instead of a single
//! JSON document. Completion is one bit the server owns, so JSON costs nothing;
//! an answer is prose *you* wrote and will want to reread, so it should stay
//! legible without this server running — openable in an editor, greppable, and
//! diffable if you ever decide to commit it.

use std::path::{Path, PathBuf};

/// Gitignored directory holding every answer, at the repo root.
pub const DIR_NAME: &str = ".checkpoint-answers";

/// Where a lesson's answers live on disk, e.g.
/// `.checkpoint-answers/phase0-setup/02-installing-rust.md`.
pub fn path_for(root: &Path, lesson_path: &str) -> PathBuf {
    root.join(DIR_NAME).join(format!("{lesson_path}.md"))
}

/// The same location as a repo-relative string, for showing you where your
/// words went.
pub fn display_path(lesson_path: &str) -> String {
    format!("{DIR_NAME}/{lesson_path}.md")
}

/// The saved answer for a lesson. A missing, unreadable, or blank file all mean
/// the same thing — nothing written yet — so a fresh clone must not error.
pub fn load(root: &Path, lesson_path: &str) -> Option<String> {
    if !is_safe(lesson_path) {
        return None;
    }
    let mut text = std::fs::read_to_string(path_for(root, lesson_path)).ok()?;
    // Drop only the trailing newline `save` adds, so what you get back is
    // character-for-character what you typed.
    if text.ends_with('\n') {
        text.pop();
    }
    if text.trim().is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Replace a lesson's answer. A blank body deletes the file rather than leaving
/// an empty husk behind, so clearing the box really does clear the answer.
pub fn save(root: &Path, lesson_path: &str, body: &str) -> std::io::Result<()> {
    if !is_safe(lesson_path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "not a lesson path",
        ));
    }
    let path = path_for(root, lesson_path);
    let body = normalize(body);
    if body.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            result => result,
        };
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, format!("{body}\n"))
}

/// Browsers submit textarea content with CRLF line breaks; store LF so the file
/// reads like anything else you'd write in an editor. Trailing whitespace goes
/// too, which is also what turns a box of blank lines into "no answer".
///
/// Leading whitespace is deliberately kept — an answer may well open with an
/// indented code block.
fn normalize(body: &str) -> String {
    body.replace("\r\n", "\n").trim_end().to_string()
}

/// The caller has already checked the path against the tree, so this only ever
/// fails for a crafted request — but this is the one place where a request
/// decides *which* file gets written, so it refuses anything that could climb
/// out of `.checkpoint-answers/`.
fn is_safe(lesson_path: &str) -> bool {
    !lesson_path.is_empty()
        && lesson_path.split('/').all(|segment| {
            !segment.is_empty()
                && segment != "."
                && segment != ".."
                && !segment.contains('\\')
                && !segment.contains(':')
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("course-ui-answers-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn missing_file_means_nothing_written() {
        let root = temp_root("missing");
        assert_eq!(load(&root, "phase0/01-intro"), None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn round_trips_verbatim_into_a_mirror_of_the_lesson_tree() {
        let root = temp_root("roundtrip");
        let answer = "۱. `rustup` نسخه‌ها را مدیریت می‌کند.\n\n۲. قفل‌کردن toolchain.";

        save(&root, "phase1/02-owning/01-moves", answer).unwrap();

        assert_eq!(
            load(&root, "phase1/02-owning/01-moves").as_deref(),
            Some(answer)
        );
        assert!(
            root.join(".checkpoint-answers/phase1/02-owning/01-moves.md")
                .is_file(),
            "the file mirrors the lesson's own path, so it's findable by hand"
        );

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn crlf_from_the_browser_is_stored_as_lf() {
        let root = temp_root("crlf");
        save(&root, "phase0/01-intro", "first\r\nsecond\r\n").unwrap();
        let on_disk = std::fs::read_to_string(path_for(&root, "phase0/01-intro")).unwrap();
        assert_eq!(on_disk, "first\nsecond\n");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_blank_body_deletes_the_file_and_leaves_other_answers_alone() {
        let root = temp_root("blank");
        save(&root, "phase0/01-intro", "keep me").unwrap();
        save(&root, "phase0/02-next", "draft").unwrap();

        save(&root, "phase0/02-next", "  \n \n").unwrap();

        assert_eq!(load(&root, "phase0/02-next"), None);
        assert!(!path_for(&root, "phase0/02-next").exists());
        assert_eq!(load(&root, "phase0/01-intro").as_deref(), Some("keep me"));

        // Clearing an answer that was never written is not an error.
        save(&root, "phase0/03-untouched", "").unwrap();

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_crafted_path_cannot_escape_the_answers_directory() {
        let root = temp_root("escape");
        assert!(save(&root, "../../etc/passwd", "nope").is_err());
        assert!(save(&root, "", "nope").is_err());
        assert_eq!(load(&root, "../../etc/passwd"), None);
        assert!(!root.parent().unwrap().join("etc").exists());
        let _ = std::fs::remove_dir_all(&root);
    }
}
