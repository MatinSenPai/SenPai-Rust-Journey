//! Reads and writes `.course-progress.json`, the source of truth for how far
//! through the course you are.
//!
//! See `docs/adr/0001-web-ui-progress-state.md` for why this file exists rather
//! than the server rewriting `PROGRESS.md`'s checkboxes, and why it's
//! gitignored.
//!
//! # Schema v2
//!
//! v1 stored a single set of completed lesson paths — enough for a checkmark
//! and nothing else. v2 stores a record per lesson so the dashboard can answer
//! the question the checkmark could not: *do I actually know this?* Records
//! carry when you first opened a lesson, when you finished it, which exercise
//! rungs you ticked, how confident you felt, and a note to yourself.
//!
//! Two properties from v1 are preserved and tested below:
//!
//! - Keys are repo-relative **directory paths**, which the filesystem enforces
//!   — unlike the prose titles `PROGRESS.md` uses.
//! - Orphaned keys (left behind when a lesson directory is renamed) are
//!   preserved, never pruned. A rename costs one re-tick instead of silently
//!   discarding history, and the file stays safe to hand-edit.
//!
//! A v1 file is migrated on read and rewritten as v2 on the next save. The
//! migration is lossless in the only sense available: everything v1 recorded
//! survives, and the fields v1 never had are simply absent.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

pub const FILE_NAME: &str = ".course-progress.json";

const VERSION: u32 = 2;

/// A lesson left open overnight is not four hours of study. Time on a lesson
/// is derived from two page loads, so it needs a ceiling to stay honest — the
/// dashboard labels the total an estimate for the same reason.
const MAX_LESSON_SECONDS: i64 = 3 * 60 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    #[default]
    Untouched,
    InProgress,
    Done,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Record {
    #[serde(default)]
    pub status: Status,
    /// Unix seconds. Set the first time the lesson page is rendered.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_seen_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    /// Slugs of the exercise rungs ticked off, e.g. `warm-up`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exercises: Vec<String>,
    /// Self-rated 1-3. Never shown as a score, only as "what to revisit".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<u8>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub note: String,
}

impl Record {
    pub fn is_done(&self) -> bool {
        self.status == Status::Done
    }

    /// Estimated seconds spent, from first view to completion.
    pub fn seconds(&self) -> Option<i64> {
        let (start, end) = (self.first_seen_at?, self.completed_at?);
        (end > start).then(|| (end - start).min(MAX_LESSON_SECONDS))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentV2 {
    version: u32,
    /// A `BTreeMap` so the file is stable: it diffs and hand-edits cleanly.
    lessons: BTreeMap<String, Record>,
}

#[derive(Debug, Deserialize)]
struct DocumentV1 {
    completed: Vec<String>,
}

#[derive(Debug, Default)]
pub struct Progress {
    lessons: BTreeMap<String, Record>,
    /// The file on disk exists but could not be parsed. We still serve the UI
    /// (from an empty state), but we refuse to write — saving here would
    /// replace a file we failed to understand with one holding nothing, and
    /// this file is not in git to recover from.
    unreadable_source: bool,
}

impl Progress {
    /// Load progress for the repo at `root`. A *missing* file means "nothing
    /// recorded yet" — a fresh clone must not error. A present-but-unparseable
    /// file also yields an empty state so the UI stays usable, but marks itself
    /// read-only; see `unreadable_source`.
    pub fn load(root: &Path) -> Self {
        let Ok(text) = std::fs::read_to_string(path_for(root)) else {
            return Self::default();
        };
        if let Ok(doc) = serde_json::from_str::<DocumentV2>(&text) {
            if doc.version >= 2 {
                return Self {
                    lessons: doc.lessons,
                    unreadable_source: false,
                };
            }
        }
        if let Ok(doc) = serde_json::from_str::<DocumentV1>(&text) {
            return Self::from_v1(&doc.completed);
        }
        Self {
            lessons: BTreeMap::new(),
            unreadable_source: true,
        }
    }

    fn from_v1(completed: &[String]) -> Self {
        Self {
            lessons: completed
                .iter()
                .map(|path| {
                    (
                        path.clone(),
                        Record {
                            status: Status::Done,
                            ..Record::default()
                        },
                    )
                })
                .collect(),
            unreadable_source: false,
        }
    }

    /// Write atomically: serialise to a sibling temp file, then rename over the
    /// real one. `fs::write` truncates first, so a reader in another request
    /// could see a half-written — or empty — file, load nothing from it, and
    /// then save *that* back over everything. Rename is atomic on both Windows
    /// and Unix, so a reader sees either the old file or the new one.
    pub fn save(&self, root: &Path) -> std::io::Result<()> {
        if self.unreadable_source {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!(
                    "{FILE_NAME} exists but could not be parsed; refusing to overwrite it. \
                     Fix or delete the file to start recording again."
                ),
            ));
        }
        let doc = DocumentV2 {
            version: VERSION,
            lessons: self.lessons.clone(),
        };
        let mut json = serde_json::to_string_pretty(&doc)?;
        json.push('\n');

        let final_path = path_for(root);
        let temp_path = final_path.with_extension("json.tmp");
        std::fs::write(&temp_path, json)?;
        std::fs::rename(&temp_path, &final_path)
    }

    pub fn get(&self, lesson_path: &str) -> Option<&Record> {
        self.lessons.get(lesson_path)
    }

    pub fn is_complete(&self, lesson_path: &str) -> bool {
        self.lessons.get(lesson_path).is_some_and(Record::is_done)
    }

    /// Note that a lesson page was opened. Only the *first* view is recorded,
    /// so re-reading a finished lesson never rewrites its history.
    pub fn touch(&mut self, lesson_path: &str) -> bool {
        let record = self.lessons.entry(lesson_path.to_string()).or_default();
        if record.first_seen_at.is_some() {
            return false;
        }
        record.first_seen_at = Some(now());
        if record.status == Status::Untouched {
            record.status = Status::InProgress;
        }
        true
    }

    pub fn set(&mut self, lesson_path: &str, complete: bool) {
        let record = self.lessons.entry(lesson_path.to_string()).or_default();
        if complete {
            record.status = Status::Done;
            record.completed_at = Some(now());
            record.first_seen_at.get_or_insert_with(now);
        } else {
            record.status = Status::InProgress;
            record.completed_at = None;
        }
    }

    pub fn set_exercise(&mut self, lesson_path: &str, slug: &str, done: bool) {
        let record = self.lessons.entry(lesson_path.to_string()).or_default();
        let position = record.exercises.iter().position(|item| item == slug);
        match (done, position) {
            (true, None) => record.exercises.push(slug.to_string()),
            (false, Some(index)) => {
                record.exercises.remove(index);
            }
            _ => {}
        }
        if record.status == Status::Untouched {
            record.status = Status::InProgress;
        }
    }

    pub fn set_confidence(&mut self, lesson_path: &str, confidence: Option<u8>) {
        self.lessons
            .entry(lesson_path.to_string())
            .or_default()
            .confidence = confidence.filter(|value| (1..=3).contains(value));
    }

    pub fn set_note(&mut self, lesson_path: &str, note: &str) {
        self.lessons
            .entry(lesson_path.to_string())
            .or_default()
            .note = note.trim().to_string();
    }

    /// Estimated total study time across every finished lesson, in seconds.
    pub fn total_seconds(&self) -> i64 {
        self.lessons.values().filter_map(Record::seconds).sum()
    }

    /// Lessons finished, most recent first. Records migrated from v1 have no
    /// timestamp and so cannot appear here — they are counted, not dated.
    pub fn recent(&self, limit: usize) -> Vec<(&str, i64)> {
        let mut done: Vec<(&str, i64)> = self
            .lessons
            .iter()
            .filter_map(|(path, record)| Some((path.as_str(), record.completed_at?)))
            .collect();
        done.sort_by_key(|(_, completed_at)| std::cmp::Reverse(*completed_at));
        done.truncate(limit);
        done
    }

    /// Consecutive days, counting back from today, on which at least one lesson
    /// was finished. Today not yet having one does not break a streak — it has
    /// not ended until a whole day passes with nothing in it.
    pub fn streak(&self) -> u32 {
        let mut days: Vec<i64> = self
            .lessons
            .values()
            .filter_map(|record| Some(day_of(record.completed_at?)))
            .collect();
        days.sort_unstable();
        days.dedup();
        if days.is_empty() {
            return 0;
        }

        let today = day_of(now());
        let last = *days.last().expect("checked non-empty");
        if today - last > 1 {
            return 0;
        }

        let mut streak = 1;
        for pair in days.windows(2).rev() {
            if pair[1] - pair[0] == 1 {
                streak += 1;
            } else {
                break;
            }
        }
        streak
    }
}

fn path_for(root: &Path) -> PathBuf {
    root.join(FILE_NAME)
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0)
}

/// Whole days since the epoch. UTC, which can put a late-night session on the
/// next day for some readers — accepted deliberately over carrying a timezone
/// database around for a motivational counter.
fn day_of(seconds: i64) -> i64 {
    seconds.div_euclid(86_400)
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

    #[test]
    fn an_unparseable_file_is_never_overwritten_with_an_empty_one() {
        // The file is gitignored, so a bad write is unrecoverable. Better to
        // refuse and say so than to quietly replace it with `{}`.
        let root = temp_root("no-clobber");
        std::fs::write(root.join(FILE_NAME), "{ half a file").unwrap();

        let mut progress = Progress::load(&root);
        progress.set("phase0/01-intro", true);
        let error = progress.save(&root).expect_err("save must refuse");
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

        let on_disk = std::fs::read_to_string(root.join(FILE_NAME)).unwrap();
        assert_eq!(on_disk, "{ half a file", "the original bytes are untouched");
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_missing_file_is_writable_but_an_unreadable_one_is_not() {
        let root = temp_root("missing-writable");
        let mut progress = Progress::load(&root);
        progress.set("phase0/01-intro", true);
        progress
            .save(&root)
            .expect("a fresh clone can record progress");
        assert!(Progress::load(&root).is_complete("phase0/01-intro"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn saving_leaves_no_temp_file_behind() {
        let root = temp_root("atomic");
        let mut progress = Progress::default();
        progress.set("phase0/01-intro", true);
        progress.save(&root).unwrap();
        assert!(root.join(FILE_NAME).is_file());
        assert!(
            !root.join(format!("{FILE_NAME}.tmp")).exists(),
            "the temp file is renamed, not left next to the real one"
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn a_version_1_file_migrates_without_losing_completions() {
        let root = temp_root("migrate");
        std::fs::write(
            root.join(FILE_NAME),
            r#"{"version":1,"completed":["phase0/01-intro","phase0/02-install"]}"#,
        )
        .unwrap();

        let mut progress = Progress::load(&root);
        assert!(progress.is_complete("phase0/01-intro"));
        assert!(progress.is_complete("phase0/02-install"));

        progress.set("phase0/03-cargo", true);
        progress.save(&root).unwrap();

        let text = std::fs::read_to_string(root.join(FILE_NAME)).unwrap();
        assert!(text.contains("\"version\": 2"));
        let reloaded = Progress::load(&root);
        assert!(reloaded.is_complete("phase0/01-intro"));
        assert!(reloaded.is_complete("phase0/03-cargo"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn opening_a_lesson_is_recorded_once_and_never_rewritten() {
        let mut progress = Progress::default();
        assert!(progress.touch("phase0/01-intro"), "first view is recorded");
        let first = progress.get("phase0/01-intro").unwrap().first_seen_at;
        assert!(first.is_some());
        assert_eq!(
            progress.get("phase0/01-intro").unwrap().status,
            Status::InProgress
        );

        assert!(
            !progress.touch("phase0/01-intro"),
            "a second view changes nothing"
        );
        assert_eq!(
            progress.get("phase0/01-intro").unwrap().first_seen_at,
            first
        );
    }

    #[test]
    fn exercise_ticks_toggle_and_do_not_duplicate() {
        let mut progress = Progress::default();
        progress.set_exercise("l", "warm-up", true);
        progress.set_exercise("l", "warm-up", true);
        assert_eq!(progress.get("l").unwrap().exercises, vec!["warm-up"]);
        progress.set_exercise("l", "warm-up", false);
        assert!(progress.get("l").unwrap().exercises.is_empty());
    }

    #[test]
    fn confidence_outside_one_to_three_is_dropped() {
        let mut progress = Progress::default();
        progress.set_confidence("l", Some(2));
        assert_eq!(progress.get("l").unwrap().confidence, Some(2));
        progress.set_confidence("l", Some(9));
        assert_eq!(progress.get("l").unwrap().confidence, None);
    }

    #[test]
    fn time_on_a_lesson_is_capped_so_an_open_tab_is_not_study() {
        let mut progress = Progress::default();
        progress.set("l", true);
        let record = progress.lessons.get_mut("l").unwrap();
        record.first_seen_at = Some(0);
        record.completed_at = Some(1_800);
        assert_eq!(progress.get("l").unwrap().seconds(), Some(1_800));

        let record = progress.lessons.get_mut("l").unwrap();
        record.completed_at = Some(86_400);
        assert_eq!(
            progress.get("l").unwrap().seconds(),
            Some(MAX_LESSON_SECONDS),
            "a lesson left open overnight is capped, not counted whole"
        );
    }

    #[test]
    fn a_streak_counts_back_over_consecutive_days_only() {
        let mut progress = Progress::default();
        let day = 86_400;
        let today = now().div_euclid(day) * day;
        for (lesson, offset) in [("a", 0), ("b", -1), ("c", -2), ("d", -5)] {
            progress.set(lesson, true);
            progress.lessons.get_mut(lesson).unwrap().completed_at = Some(today + offset * day);
        }
        assert_eq!(progress.streak(), 3, "the five-day-old lesson breaks it");
    }

    #[test]
    fn a_streak_survives_a_day_with_nothing_finished_yet() {
        let mut progress = Progress::default();
        let day = 86_400;
        let today = now().div_euclid(day) * day;
        for (lesson, offset) in [("a", -1), ("b", -2)] {
            progress.set(lesson, true);
            progress.lessons.get_mut(lesson).unwrap().completed_at = Some(today + offset * day);
        }
        assert_eq!(progress.streak(), 2, "today has not ended yet");

        progress.set("c", true);
        progress.lessons.get_mut("c").unwrap().completed_at = Some(today - 3 * day);
        assert_eq!(progress.streak(), 3);
    }

    #[test]
    fn recent_activity_is_newest_first() {
        let mut progress = Progress::default();
        for (lesson, at) in [("a", 100), ("b", 300), ("c", 200)] {
            progress.set(lesson, true);
            progress.lessons.get_mut(lesson).unwrap().completed_at = Some(at);
        }
        let recent: Vec<&str> = progress.recent(2).into_iter().map(|(p, _)| p).collect();
        assert_eq!(recent, vec!["b", "c"]);
    }
}
