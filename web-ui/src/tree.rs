//! Turns the repo's directory layout into the navigable node tree.
//!
//! The whole tree is derived from the filesystem on every request — there is no
//! cache, so adding a lesson directory or editing a `README.md` shows up on
//! refresh. A full walk of this repo is a few milliseconds, which is cheaper
//! than the cache-invalidation bugs the alternative would buy.
//!
//! The classification rule (see `docs/conventions.md`):
//!
//! - A directory is a **node** if it directly contains at least one `.md` file.
//! - A subdirectory that isn't a node is *passed through* — we keep descending
//!   to find nodes below it. That's how `capstone-taskforge/docs/adr/` is
//!   reachable even though `capstone-taskforge/docs/` holds no markdown itself.
//! - A node is a **leaf** when it has no child nodes, and a leaf that owns a
//!   `README.md` is a **lesson** — the only thing that can be marked complete.
//!
//! `solution/` is skipped as a directory so it never becomes a node of its own;
//! `SOLUTION.md` is pulled in as a gated page of the lesson that owns it.

use std::path::Path;

use crate::locale::Locale;

/// Directory name holding a lesson's reference solution.
const SOLUTION_DIR: &str = "solution";
/// Directories that never contain curriculum content. `web-ui` is this crate:
/// the reader shouldn't list its own source as something to work through.
const SKIP_DIRS: &[&str] = &["target", "node_modules", "web-ui", "plans"];

#[derive(Debug, Clone)]
pub struct Node {
    /// Repo-relative path with `/` separators. Empty string for the repo root.
    pub path: String,
    pub title: String,
    /// Markdown files this node owns, in reading order.
    pub pages: Vec<Page>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone)]
pub struct Page {
    /// Path relative to the node's directory, e.g. `README.md`.
    pub file: String,
    /// Fragment id this page renders under.
    pub anchor: String,
    /// Rendered behind a click-to-reveal (`SOLUTION.md` only).
    pub gated: bool,
}

impl Node {
    /// A lesson is the atomic unit you work through — and the only markable one.
    pub fn is_lesson(&self) -> bool {
        self.children.is_empty() && self.pages.iter().any(|p| p.file == "README.md")
    }

    /// Every lesson at or below this node, in tree order.
    pub fn lessons(&self) -> Vec<&Node> {
        let mut out = Vec::new();
        self.collect_lessons(&mut out);
        out
    }

    /// Every visible content node, including parents and documentation folders.
    pub fn nodes(&self) -> Vec<&Node> {
        let mut out = Vec::new();
        self.collect_nodes(&mut out);
        out
    }

    fn collect_nodes<'a>(&'a self, out: &mut Vec<&'a Node>) {
        out.push(self);
        for child in &self.children {
            child.collect_nodes(out);
        }
    }

    fn collect_lessons<'a>(&'a self, out: &mut Vec<&'a Node>) {
        if self.is_lesson() {
            out.push(self);
        }
        for child in &self.children {
            child.collect_lessons(out);
        }
    }

    /// Find a node by its repo-relative path.
    pub fn find(&self, path: &str) -> Option<&Node> {
        if self.path == path {
            return Some(self);
        }
        // Only descend where the target could actually live.
        if !path.starts_with(&self.path) {
            return None;
        }
        self.children.iter().find_map(|c| c.find(path))
    }
}

/// Build the tree rooted at `root`. Returns `None` if `root` holds no markdown.
pub fn build(root: &Path, locale: Locale) -> Option<Node> {
    node_at(root, "", locale)
}

fn node_at(root: &Path, rel: &str, locale: Locale) -> Option<Node> {
    let dir = join(root, rel);
    let pages = pages_in(&dir);
    if pages.is_empty() {
        return None;
    }
    let children = child_nodes(root, rel, locale);
    let title = title_for(&dir, &pages, rel, locale);
    Some(Node {
        path: rel.to_string(),
        title,
        pages,
        children,
    })
}

/// Child nodes of `rel`, passing through directories that hold no markdown.
fn child_nodes(root: &Path, rel: &str, locale: Locale) -> Vec<Node> {
    let mut out = Vec::new();
    for name in subdirs(&join(root, rel)) {
        if name.starts_with('.') || name == SOLUTION_DIR || SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        let child_rel = if rel.is_empty() {
            name
        } else {
            format!("{rel}/{name}")
        };
        match node_at(root, &child_rel, locale) {
            Some(node) => out.push(node),
            None => out.extend(child_nodes(root, &child_rel, locale)),
        }
    }
    out
}

/// Markdown files a directory owns, in reading order: `README.md`, then
/// `CHECKPOINT.md`, then anything else alphabetically, then the gated
/// `solution/SOLUTION.md`.
fn pages_in(dir: &Path) -> Vec<Page> {
    let mut names: Vec<String> = read_dir_names(dir)
        .into_iter()
        .filter(|(_, is_dir)| !*is_dir)
        .map(|(name, _)| name)
        .filter(|name| name.to_lowercase().ends_with(".md"))
        .filter(|name| !is_translation(name))
        .collect();
    names.sort();

    let rank = |name: &str| match name {
        "README.md" => 0,
        "CHECKPOINT.md" => 1,
        _ => 2,
    };
    names.sort_by_key(|n| rank(n));

    let mut pages: Vec<Page> = names
        .into_iter()
        .map(|file| Page {
            anchor: anchor_for(&file),
            file,
            gated: false,
        })
        .collect();

    let solution = dir.join(SOLUTION_DIR).join("SOLUTION.md");
    if solution.is_file() {
        pages.push(Page {
            file: format!("{SOLUTION_DIR}/SOLUTION.md"),
            anchor: "solution".to_string(),
            gated: true,
        });
    }
    pages
}

/// Fragment id for a page: `CHECKPOINT.md` -> `checkpoint-md`.
pub fn anchor_for(file: &str) -> String {
    let slug: String = file
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    format!("file-{slug}")
}

/// A node's title is its `README.md`'s H1, falling back to the directory name.
///
/// Deliberately *only* `README.md`: a folder of loose documents like `docs/`
/// would otherwise be titled after whichever file happens to sort first, which
/// reads as though the folder *is* that document.
fn title_for(dir: &Path, pages: &[Page], rel: &str, locale: Locale) -> String {
    if pages.iter().any(|p| p.file == "README.md") {
        let canonical = dir.join("README.md");
        let localized = localized_path(&canonical, locale);
        if let Ok(text) = std::fs::read_to_string(localized) {
            if let Some(h1) = text.lines().find_map(|l| l.strip_prefix("# ")) {
                let title = plain_text(h1.trim());
                if !title.is_empty() {
                    return title;
                }
            }
        }
    }
    let name = rel.rsplit('/').next().unwrap_or("");
    if name.is_empty() {
        dir.file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "/".to_string())
    } else {
        name.replace(['-', '_'], " ")
    }
}

/// Resolve a canonical Markdown path to its locale companion when present.
/// English is always canonical; Persian falls back to English during staged
/// translation so the site remains navigable.
pub fn localized_path(canonical: &Path, locale: Locale) -> std::path::PathBuf {
    if locale == Locale::En {
        return canonical.to_path_buf();
    }
    let Some(stem) = canonical.file_stem().and_then(|value| value.to_str()) else {
        return canonical.to_path_buf();
    };
    let companion = canonical.with_file_name(format!("{stem}.fa.md"));
    if companion.is_file() {
        companion
    } else {
        canonical.to_path_buf()
    }
}

pub fn has_translation(canonical: &Path) -> bool {
    localized_path(canonical, Locale::Fa) != canonical
}

fn is_translation(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".fa.md")
}

/// Strip the inline markdown an H1 may carry, so a heading like
/// ``# `Option`, `Result` & error basics`` doesn't show its backticks in the
/// sidebar. Titles are rendered as plain text, never as markdown.
fn plain_text(heading: &str) -> String {
    heading.replace(['`', '*', '_'], "").trim().to_string()
}

fn subdirs(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = read_dir_names(dir)
        .into_iter()
        .filter(|(_, is_dir)| *is_dir)
        .map(|(name, _)| name)
        .collect();
    names.sort_by(|left, right| {
        curriculum_rank(left)
            .cmp(&curriculum_rank(right))
            .then_with(|| left.cmp(right))
    });
    names
}

fn curriculum_rank(name: &str) -> u8 {
    match name {
        "phase0-setup" => 0,
        "phase1-fundamentals" => 1,
        "phase2-intermediate" => 2,
        "phase3-backend-foundations" => 3,
        "phase4-backend-advanced" => 4,
        "phase5-system-design-mastery" => 5,
        "side-quests" => 6,
        "capstone-taskforge" => 7,
        "docs" => 8,
        _ => 9,
    }
}

fn read_dir_names(dir: &Path) -> Vec<(String, bool)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().into_owned();
            let is_dir = e.file_type().ok()?.is_dir();
            Some((name, is_dir))
        })
        .collect()
}

fn join(root: &Path, rel: &str) -> std::path::PathBuf {
    if rel.is_empty() {
        root.to_path_buf()
    } else {
        root.join(rel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    /// Builds a miniature copy of this repo's real shapes: a 2-level phase, a
    /// 3-level phase with a module-group, a lesson with a `solution/`, and a
    /// README-less docs folder nested under a markdown-free directory.
    fn fixture(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("course-ui-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);

        let write = |rel: &str, body: &str| {
            let path = root.join(rel);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, body).unwrap();
        };

        write("README.md", "# Fixture Repo\n");
        write("PROGRESS.md", "# Progress\n");

        // 2-level: phase -> lesson
        write("phase0/README.md", "# Phase 0\n");
        write("phase0/01-intro/README.md", "# 01 - Intro\n");
        write("phase0/01-intro/CHECKPOINT.md", "# Checkpoint\n");

        // 3-level: phase -> module-group -> lesson, lesson has src/ + solution/
        write("phase1/README.md", "# Phase 1\n");
        write("phase1/02-owning/README.md", "# 02 - Owning\n");
        write("phase1/02-owning/01-moves/README.md", "# 01 - Moves\n");
        write("phase1/02-owning/01-moves/CHECKPOINT.md", "# Checkpoint\n");
        write("phase1/02-owning/01-moves/src/lib.rs", "// code\n");
        write(
            "phase1/02-owning/01-moves/solution/SOLUTION.md",
            "# Solution\n",
        );

        // README-less markdown dir, reachable only by passing through `wrap/`.
        write("wrap/adr/0001-thing.md", "# ADR-0001\n");

        // Never a node.
        write("target/debug/junk.md", "# junk\n");

        root
    }

    #[test]
    fn classifies_phases_module_groups_lessons_and_docs() {
        let root = fixture("classify");
        let tree = build(&root, Locale::En).expect("root has markdown");

        assert_eq!(tree.title, "Fixture Repo");
        assert!(!tree.is_lesson(), "root is a parent, not a lesson");

        // Root's own pages, in reading order (README first).
        let files: Vec<&str> = tree.pages.iter().map(|p| p.file.as_str()).collect();
        assert_eq!(files, vec!["README.md", "PROGRESS.md"]);

        let child_paths: Vec<&str> = tree.children.iter().map(|c| c.path.as_str()).collect();
        assert_eq!(
            child_paths,
            vec!["phase0", "phase1", "wrap/adr"],
            "`wrap/` holds no markdown so it is passed through, and `target/` is skipped"
        );

        let phase0 = tree.find("phase0").unwrap();
        assert!(!phase0.is_lesson());

        let lesson = tree.find("phase0/01-intro").unwrap();
        assert!(lesson.is_lesson());
        assert_eq!(lesson.title, "01 - Intro");

        // A module-group is a parent even though it looks like a lesson's sibling.
        let group = tree.find("phase1/02-owning").unwrap();
        assert!(!group.is_lesson());
        assert!(tree.find("phase1/02-owning/01-moves").unwrap().is_lesson());

        // A leaf without a README is a node but NOT a lesson, so it can't be ticked.
        let adr = tree.find("wrap/adr").unwrap();
        assert!(adr.children.is_empty());
        assert!(
            !adr.is_lesson(),
            "docs folders are readable but not markable"
        );

        let lessons: Vec<&str> = tree.lessons().iter().map(|l| l.path.as_str()).collect();
        assert_eq!(
            lessons,
            vec!["phase0/01-intro", "phase1/02-owning/01-moves"]
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn solution_is_a_gated_page_not_a_node() {
        let root = fixture("solution");
        let tree = build(&root, Locale::En).unwrap();
        let lesson = tree.find("phase1/02-owning/01-moves").unwrap();

        assert!(
            lesson.children.is_empty(),
            "`solution/` and `src/` must not become child nodes"
        );

        let pages: Vec<(&str, bool)> = lesson
            .pages
            .iter()
            .map(|p| (p.file.as_str(), p.gated))
            .collect();
        assert_eq!(
            pages,
            vec![
                ("README.md", false),
                ("CHECKPOINT.md", false),
                ("solution/SOLUTION.md", true),
            ],
            "reading order is README -> CHECKPOINT -> gated SOLUTION"
        );

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn persian_companions_localize_titles_without_becoming_pages() {
        let root = fixture("locale");
        fs::write(root.join("phase0/01-intro/README.fa.md"), "# ۰۱ — شروع\n").unwrap();
        let tree = build(&root, Locale::Fa).unwrap();
        let lesson = tree.find("phase0/01-intro").unwrap();
        assert_eq!(lesson.title, "۰۱ — شروع");
        assert_eq!(lesson.pages.len(), 2, "companion is not a second page");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn curriculum_starts_with_phase_zero_before_capstone_and_docs() {
        let mut names = [
            "docs".to_string(),
            "capstone-taskforge".to_string(),
            "phase1-fundamentals".to_string(),
            "phase0-setup".to_string(),
        ];
        names.sort_by(|left, right| {
            curriculum_rank(left)
                .cmp(&curriculum_rank(right))
                .then_with(|| left.cmp(right))
        });
        assert_eq!(
            names,
            [
                "phase0-setup",
                "phase1-fundamentals",
                "capstone-taskforge",
                "docs"
            ]
        );
    }
}
