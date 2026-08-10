//! Reads and writes `.course-progress.json`, the source of truth for which
//! lessons are complete.
//!
//! See `docs/adr/0001-web-ui-progress-state.md` for why this file exists rather
//! than the server rewriting `PROGRESS.md`'s checkboxes, and why it's
//! gitignored. Two properties matter and are tested below:
//!
//! - Keys are repo-relative **directory paths**, which the filesystem enforces
//!   — unlike the prose titles `PROGRESS.md` uses.
//! - Orphaned keys (left behind when a lesson directory is renamed) are
//!   preserved, never pruned. A rename costs one re-tick instead of silently
//!   discarding history, and the file stays safe to hand-edit.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub const FILE_NAME: &str = ".course-progress.json";

/// Bumped only if the on-disk shape changes; lets a future version migrate
/// instead of failing to parse.
const VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct Document {
    version: u32,
    /// Sorted for a stable file that diffs and hand-edits cleanly.
    completed: BTreeSet<String>,
}

#[derive(Debug, Default)]
pub struct Progress {
    completed: BTreeSet<String>,
}

impl Progress {
    /// Load progress for the repo at `root`. A missing or unreadable file means
    /// "nothing completed yet" — a fresh clone must not error.
    pub fn load(root: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path_for(root)) else {
            return Self::default();
        };
        match serde_json::from_str::<Document>(&text) {
            Ok(doc) => Self {
                completed: doc.completed,
            },
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, root: &Path) -> std::io::Result<()> {
        let doc = Document {
            version: VERSION,
            completed: self.completed.clone(),
        };
        let mut json = serde_json::to_string_pretty(&doc)?;
        json.push('\n');
        std::fs::write(path_for(root), json)
    }

    pub fn is_complete(&self, lesson_path: &str) -> bool {
        self.completed.contains(lesson_path)
    }

    pub fn set(&mut self, lesson_path: &str, complete: bool) {
        if complete {
            self.completed.insert(lesson_path.to_string());
        } else {
            self.completed.remove(lesson_path);
        }
    }
}

fn path_for(root: &Path) -> PathBuf {
    root.join(FILE_NAME)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("course-ui-prog-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        root
    }

    #[test]
    fn missing_file_means_nothing_completed() {
        let root = temp_root("missing");
        let progress = Progress::load(&root);
        assert!(!progress.is_complete("phase0/01-intro"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn round_trips_and_preserves_orphaned_keys() {
        let root = temp_root("roundtrip");

        let mut progress = Progress::load(&root);
        progress.set("phase0/01-intro", true);
        // A lesson that has since been renamed away — its key is now an orphan.
        progress.set("phase1/02-owning/01-old-name", true);
        progress.save(&root).unwrap();

        let reloaded = Progress::load(&root);
        assert!(reloaded.is_complete("phase0/01-intro"));
        assert!(
            reloaded.is_complete("phase1/02-owning/01-old-name"),
            "orphaned keys survive a load/save cycle instead of being pruned"
        );

        // Un-ticking removes only the key asked for.
        let mut progress = reloaded;
        progress.set("phase0/01-intro", false);
        progress.save(&root).unwrap();

        let reloaded = Progress::load(&root);
        assert!(!reloaded.is_complete("phase0/01-intro"));
        assert!(reloaded.is_complete("phase1/02-owning/01-old-name"));

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn corrupt_file_degrades_to_empty_rather_than_panicking() {
        let root = temp_root("corrupt");
        std::fs::write(root.join(FILE_NAME), "{ not json").unwrap();
        assert!(!Progress::load(&root).is_complete("phase0/01-intro"));
        let _ = std::fs::remove_dir_all(&root);
    }
}
