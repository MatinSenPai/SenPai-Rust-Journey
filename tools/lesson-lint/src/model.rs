//! Discovering lessons and putting them in curriculum order.
//!
//! Order matters more than it looks: the concept-map check in [`crate::concepts`]
//! is entirely built on "is this lesson before or after that one", so the
//! ordering here has to match the order a learner actually walks the repo in —
//! which is the same order `web-ui/src/tree.rs` renders the sidebar in.

use std::path::{Path, PathBuf};

/// Curriculum roots, in the order they are worked through. Anything not listed
/// is not linted: `capstone-taskforge/` is project code rather than lessons,
/// and `docs/`, `plans/`, `web-ui/`, `tools/` are not curriculum at all.
pub const ROOTS: &[&str] = &[
    "phase0-setup",
    "phase1-fundamentals",
    "phase2-intermediate",
    "phase3-backend-foundations",
    "phase4-backend-advanced",
    "phase5-system-design-mastery",
    "side-quests",
];

/// Never descended into when looking for child lessons. `solution/` is a
/// lesson's own reference answer, not a lesson beneath it.
const NON_LESSON_DIRS: &[&str] = &[
    "solution",
    "src",
    "tests",
    "examples",
    "benches",
    "migrations",
    "target",
    "assets",
    "proto",
];

#[derive(Debug, Clone)]
pub struct Lesson {
    /// Repo-relative, always `/`-separated.
    pub path: String,
    pub dir: PathBuf,
    /// Position in the curriculum. Lower is earlier.
    pub order: usize,
    /// Has a `Cargo.toml`, so the reader is expected to write code.
    pub has_code: bool,
}

impl Lesson {
    pub fn readme(&self, locale: Locale) -> PathBuf {
        self.dir.join(locale.readme())
    }

    /// Every Rust file the lesson owns: starter code, examples, and the
    /// reference solution. All of it is subject to the concept-order rule —
    /// a solution that reaches forward teaches the same bad habit the
    /// exercise would.
    pub fn rust_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        for sub in ["src", "examples", "tests", "solution/src"] {
            collect_rs(
                &self
                    .dir
                    .join(sub.replace('/', std::path::MAIN_SEPARATOR_STR)),
                &mut out,
            );
        }
        out.sort();
        out
    }

    pub fn example_files(&self) -> Vec<PathBuf> {
        let mut out = Vec::new();
        collect_rs(&self.dir.join("examples"), &mut out);
        out.sort();
        out
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Locale {
    Fa,
    En,
}

impl Locale {
    pub const BOTH: [Locale; 2] = [Locale::Fa, Locale::En];

    pub const fn readme(self) -> &'static str {
        match self {
            Locale::Fa => "README.fa.md",
            Locale::En => "README.md",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Locale::Fa => "fa",
            Locale::En => "en",
        }
    }
}

/// Walk the curriculum roots and return every lesson, in reading order.
pub fn discover(repo: &Path) -> Vec<Lesson> {
    let mut lessons = Vec::new();
    for root in ROOTS {
        walk(repo, root, &mut lessons);
    }
    for (index, lesson) in lessons.iter_mut().enumerate() {
        lesson.order = index;
    }
    lessons
}

fn walk(repo: &Path, rel: &str, out: &mut Vec<Lesson>) {
    let dir = repo.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
    if !dir.is_dir() {
        return;
    }
    let children = child_dirs(&dir);
    let descendant_lessons: Vec<String> = children
        .iter()
        .map(|name| format!("{rel}/{name}"))
        .collect();

    let has_readme = dir.join("README.md").is_file() || dir.join("README.fa.md").is_file();
    let any_child_has_readme = descendant_lessons.iter().any(|child| {
        let child_dir = repo.join(child.replace('/', std::path::MAIN_SEPARATOR_STR));
        contains_readme_below(&child_dir)
    });

    // A directory that owns a README and has no README-owning directory beneath
    // it is the atomic unit — the thing you actually sit down and work through.
    if has_readme && !any_child_has_readme {
        out.push(Lesson {
            path: rel.to_string(),
            dir: dir.clone(),
            order: 0,
            has_code: dir.join("Cargo.toml").is_file(),
        });
        return;
    }

    for child in descendant_lessons {
        walk(repo, &child, out);
    }
}

fn contains_readme_below(dir: &Path) -> bool {
    if dir.join("README.md").is_file() || dir.join("README.fa.md").is_file() {
        return true;
    }
    child_dirs(dir)
        .iter()
        .any(|name| contains_readme_below(&dir.join(name)))
}

fn child_dirs(dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .flatten()
        .filter(|entry| entry.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .filter(|name| !name.starts_with('.') && !NON_LESSON_DIRS.contains(&name.as_str()))
        .collect();
    names.sort();
    names
}

fn collect_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Repo-relative, `/`-separated display path for any file under `repo`.
pub fn rel_display(repo: &Path, file: &Path) -> String {
    file.strip_prefix(repo)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}
