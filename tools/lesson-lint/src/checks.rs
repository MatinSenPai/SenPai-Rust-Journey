//! The rules. Each `check_*` returns findings for one lesson; `main` decides
//! whether a finding fails the build or is filtered by the migration allowlist.
//!
//! Every rule here exists because the old curriculum broke it somewhere real.
//! `docs/lesson-standard.md` is the prose version of this file.

use std::path::Path;

use crate::markdown::{self, Document};
use crate::model::{rel_display, Lesson, Locale};

#[derive(Debug)]
pub struct Finding {
    pub rule: &'static str,
    pub lesson: String,
    pub file: String,
    /// 1-based. `0` means the finding is about the file as a whole.
    pub line: usize,
    pub message: String,
}

impl Finding {
    fn new(
        rule: &'static str,
        lesson: &Lesson,
        file: impl Into<String>,
        line: usize,
        message: impl Into<String>,
    ) -> Self {
        Self {
            rule,
            lesson: lesson.path.clone(),
            file: file.into(),
            line,
            message: message.into(),
        }
    }
}

// ------------------------------------------------------------------ sections

/// The eight required `##` headings, in order. Index 3 and 4 (`Hands on`,
/// `Errors you will meet`) may be absent from a lesson with no code.
const SECTIONS: [(&str, &str); 8] = [
    ("At a glance", "در یک نگاه"),
    ("Why this matters", "چرا اهمیت دارد"),
    ("The concept", "مفهوم"),
    ("Hands on", "دست‌به‌کد"),
    ("Errors you will meet", "خطاهایی که خواهی دید"),
    ("Exercises", "تمرین"),
    ("Wrapping up", "جمع‌بندی"),
    ("Going further", "بیشتر"),
];
const CODE_ONLY_SECTIONS: [usize; 2] = [3, 4];

/// The exercise ladder, in order. Required in `## Exercises` for code lessons.
const RUNGS: [(&str, &str); 5] = [
    ("Warm up", "گرم‌کردن"),
    ("Repair", "تعمیر"),
    ("Implement", "پیاده‌سازی"),
    ("Build", "بساز"),
    ("Challenge", "چالش"),
];

/// The three fixed subsections of `## Wrapping up`.
const WRAP_UP: [(&str, &str); 3] = [
    ("What you now know", "الان می‌دانی"),
    ("What comes back later", "بعداً کامل‌تر می‌بینی"),
    ("Can you explain?", "می‌توانی توضیح بدهی؟"),
];

fn expected(pair: (&str, &str), locale: Locale) -> String {
    markdown::normalize(match locale {
        Locale::En => pair.0,
        Locale::Fa => pair.1,
    })
}

/// R2 — the eight sections are present, and in the standard's order.
pub fn check_sections(lesson: &Lesson, locale: Locale, doc: &Document, file: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    let seen: Vec<String> = doc
        .headings
        .iter()
        .filter(|h| h.level == 2)
        .map(|h| h.text.clone())
        .collect();

    let mut previous_position: Option<usize> = None;
    for (index, pair) in SECTIONS.iter().enumerate() {
        let want = expected(*pair, locale);
        let optional = !lesson.has_code && CODE_ONLY_SECTIONS.contains(&index);
        let Some(position) = seen.iter().position(|h| *h == want) else {
            if !optional {
                findings.push(Finding::new(
                    "sections",
                    lesson,
                    file,
                    0,
                    format!("missing required section `## {}`", label(*pair, locale)),
                ));
            }
            continue;
        };
        if let Some(previous) = previous_position {
            if position < previous {
                findings.push(Finding::new(
                    "sections",
                    lesson,
                    file,
                    0,
                    format!(
                        "section `## {}` is out of order — the standard's order is fixed",
                        label(*pair, locale)
                    ),
                ));
            }
        }
        previous_position = Some(position);
    }
    findings
}

/// R4 — the five exercise rungs, in order, inside `## Exercises`.
pub fn check_exercise_ladder(
    lesson: &Lesson,
    locale: Locale,
    doc: &Document,
    file: &str,
) -> Vec<Finding> {
    if !lesson.has_code {
        return Vec::new();
    }
    let exercises = expected(SECTIONS[5], locale);
    let Some(start) = doc
        .headings
        .iter()
        .position(|h| h.level == 2 && h.text == exercises)
    else {
        return Vec::new(); // already reported by check_sections
    };
    let end = doc.headings[start + 1..]
        .iter()
        .position(|h| h.level == 2)
        .map(|offset| start + 1 + offset)
        .unwrap_or(doc.headings.len());

    let inside: Vec<&str> = doc.headings[start..end]
        .iter()
        .filter(|h| h.level == 3)
        .map(|h| h.text.as_str())
        .collect();

    let mut findings = Vec::new();
    let mut previous: Option<usize> = None;
    for pair in RUNGS {
        let want = expected(pair, locale);
        // Prefix match: `### Challenge (optional)` is still the challenge rung.
        // The rung must be there and be in order; its heading may say more.
        let Some(position) = inside.iter().position(|h| h.starts_with(&want)) else {
            findings.push(Finding::new(
                "ladder",
                lesson,
                file,
                0,
                format!(
                    "exercise ladder is missing `### {}` — all five rungs are required so the \
                     reader is never asked to jump from prose to a blank function body",
                    label(pair, locale)
                ),
            ));
            continue;
        };
        if previous.is_some_and(|p| position < p) {
            findings.push(Finding::new(
                "ladder",
                lesson,
                file,
                0,
                format!(
                    "exercise rung `### {}` is out of order",
                    label(pair, locale)
                ),
            ));
        }
        previous = Some(position);
    }
    findings
}

/// R7b — `## Wrapping up` carries its three fixed subsections. The forward-link
/// one is what makes the course continuous instead of episodic.
pub fn check_wrap_up(lesson: &Lesson, locale: Locale, doc: &Document, file: &str) -> Vec<Finding> {
    let wrapping = expected(SECTIONS[6], locale);
    let Some(start) = doc
        .headings
        .iter()
        .position(|h| h.level == 2 && h.text == wrapping)
    else {
        return Vec::new();
    };
    let end = doc.headings[start + 1..]
        .iter()
        .position(|h| h.level == 2)
        .map(|offset| start + 1 + offset)
        .unwrap_or(doc.headings.len());
    let inside: Vec<&str> = doc.headings[start..end]
        .iter()
        .filter(|h| h.level == 3)
        .map(|h| h.text.as_str())
        .collect();

    WRAP_UP
        .iter()
        .filter(|pair| !inside.contains(&expected(**pair, locale).as_str()))
        .map(|pair| {
            Finding::new(
                "wrap-up",
                lesson,
                file,
                0,
                format!(
                    "`## {}` is missing `### {}`",
                    label(SECTIONS[6], locale),
                    label(*pair, locale)
                ),
            )
        })
        .collect()
}

/// R3 — the two language files teach the same lesson in the same shape.
pub fn check_parity(lesson: &Lesson, fa: &Document, en: &Document) -> Vec<Finding> {
    let mut findings = Vec::new();
    let shape = |doc: &Document| -> Vec<usize> { doc.headings.iter().map(|h| h.level).collect() };
    if shape(fa) != shape(en) {
        findings.push(Finding::new(
            "parity",
            lesson,
            format!("{}/README.fa.md", lesson.path),
            0,
            format!(
                "heading structure differs from README.md ({} headings vs {}) — both languages \
                 teach the same lesson in the same order",
                fa.headings.len(),
                en.headings.len()
            ),
        ));
    }
    let code = |doc: &Document| doc.fences.iter().filter(|f| f.lang == "rust").count();
    if code(fa) != code(en) {
        findings.push(Finding::new(
            "parity",
            lesson,
            format!("{}/README.fa.md", lesson.path),
            0,
            format!(
                "{} Rust code blocks here vs {} in README.md — the code must be identical in \
                 both languages; only the prose around it changes",
                code(fa),
                code(en)
            ),
        ));
    }
    findings
}

/// R8 — `## At a glance` states prerequisites, and they resolve.
pub fn check_prerequisites(
    lesson: &Lesson,
    locale: Locale,
    source: &str,
    file: &str,
) -> Vec<Finding> {
    let marker = match locale {
        Locale::En => "Prerequisites:",
        Locale::Fa => "پیش‌نیاز",
    };
    let glance = expected(SECTIONS[0], locale);
    let section = section_text(source, &glance);
    if section.is_none() {
        return Vec::new();
    }
    if markdown::normalize(&section.unwrap_or_default()).contains(&markdown::normalize(marker)) {
        return Vec::new();
    }
    vec![Finding::new(
        "prerequisites",
        lesson,
        file,
        0,
        format!(
            "`## {}` does not list prerequisites — every lesson links backwards to what it \
             depends on",
            label(SECTIONS[0], locale)
        ),
    )]
}

fn section_text(source: &str, heading: &str) -> Option<String> {
    let doc = markdown::parse(source);
    let start = doc
        .headings
        .iter()
        .find(|h| h.level == 2 && h.text == heading)?
        .line;
    let end = doc
        .headings
        .iter()
        .find(|h| h.level <= 2 && h.line > start)
        .map(|h| h.line)
        .unwrap_or(usize::MAX);
    Some(
        source
            .lines()
            .enumerate()
            .filter(|(index, _)| index + 1 > start && index + 1 < end)
            .map(|(_, line)| line)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

// --------------------------------------------------------------------- links

/// R5 — every relative markdown link resolves, and Persian pages link to
/// Persian companions where one exists.
pub fn check_links(
    repo: &Path,
    lesson: &Lesson,
    doc: &Document,
    md_path: &Path,
    file: &str,
    locale: Locale,
) -> Vec<Finding> {
    let base = md_path.parent().unwrap_or(repo);
    let mut findings = Vec::new();

    for link in &doc.links {
        let dest = &link.dest;
        if dest.starts_with('#')
            || dest.starts_with("http://")
            || dest.starts_with("https://")
            || dest.starts_with("mailto:")
            || dest.starts_with("//")
        {
            continue;
        }
        let (path_part, _) = dest.split_once('#').unwrap_or((dest.as_str(), ""));
        if path_part.is_empty() {
            continue;
        }
        let target = if let Some(absolute) = path_part.strip_prefix('/') {
            repo.join(absolute.replace('/', std::path::MAIN_SEPARATOR_STR))
        } else {
            base.join(path_part.replace('/', std::path::MAIN_SEPARATOR_STR))
        };
        let target = normalize_path(&target);

        if !target.exists() {
            findings.push(Finding::new(
                "links",
                lesson,
                file,
                link.line,
                format!("link target does not exist: `{dest}`"),
            ));
            continue;
        }

        if locale == Locale::Fa {
            if let Some(companion) = persian_companion(&target) {
                findings.push(Finding::new(
                    "links",
                    lesson,
                    file,
                    link.line,
                    format!(
                        "Persian page links to `{dest}` but `{}` exists — link to the Persian \
                         companion so the reader stays in one language",
                        rel_display(repo, &companion)
                    ),
                ));
            }
        }
    }
    findings
}

fn persian_companion(target: &Path) -> Option<std::path::PathBuf> {
    let name = target.file_name()?.to_str()?;
    let stem = name.strip_suffix(".md")?;
    if stem.ends_with(".fa") {
        return None;
    }
    let companion = target.with_file_name(format!("{stem}.fa.md"));
    companion.is_file().then_some(companion)
}

/// Resolve `..` textually — the target may not exist, so `canonicalize` is out.
fn normalize_path(path: &Path) -> std::path::PathBuf {
    let mut parts: Vec<std::ffi::OsString> = Vec::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                parts.pop();
            }
            std::path::Component::CurDir => {}
            other => parts.push(other.as_os_str().to_os_string()),
        }
    }
    parts.iter().collect()
}

// ---------------------------------------------------------------------- code

/// R6 — a `todo!()` message states *what*, never *how*.
///
/// The old curriculum had `todo!("s.parse::<u32>().map_err(|e| e.to_string())")`:
/// the answer written into the prompt. These substrings are the give-aways.
const GIVEAWAYS: [&str; 8] = [
    "::<",
    ".map_err(",
    ".unwrap_or",
    "|e|",
    "|x|",
    "=>",
    ".iter()",
    ".collect(",
];

pub fn check_todo_messages(repo: &Path, lesson: &Lesson) -> Vec<Finding> {
    let mut findings = Vec::new();
    for path in lesson.rust_files() {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let display = rel_display(repo, &path);
        for (index, line) in source.lines().enumerate() {
            if !line.contains("todo!(") {
                continue;
            }
            if let Some(hit) = GIVEAWAYS.iter().find(|g| line.contains(**g)) {
                findings.push(Finding::new(
                    "todo-message",
                    lesson,
                    display.clone(),
                    index + 1,
                    format!(
                        "`todo!()` message contains `{hit}` — it is giving the answer. State \
                         what the function should do, not how to write it"
                    ),
                ));
            }
        }
    }
    findings
}

/// R7 — a lesson with code has something to run before it asks you to write.
pub fn check_examples_exist(lesson: &Lesson) -> Vec<Finding> {
    if !lesson.has_code || !lesson.example_files().is_empty() {
        return Vec::new();
    }
    vec![Finding::new(
        "examples",
        lesson,
        format!("{}/examples/", lesson.path),
        0,
        "no `examples/` — a lesson with code must contain something the reader runs and \
         observes before being asked to write anything",
    )]
}

/// R9 — a deliberately-broken example declares the error code it produces, so
/// the reader knows what they are looking at and the lint knows not to compile
/// it. `docs/lesson-standard.md` §5 defines the marker.
pub fn check_broken_markers(repo: &Path, lesson: &Lesson) -> Vec<Finding> {
    let mut findings = Vec::new();
    for path in lesson.example_files() {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let head: String = source.lines().take(6).collect::<Vec<_>>().join("\n");
        if !head.contains("DELIBERATELY BROKEN") {
            continue;
        }
        // Not every rustc error has a code — "cannot find macro" is one that
        // doesn't — so the marker states what to expect, and names a code when
        // there is one to name.
        let expectation = head
            .split_once("expected:")
            .map(|(_, rest)| rest.lines().next().unwrap_or("").trim())
            .unwrap_or("");

        if expectation.is_empty() {
            findings.push(Finding::new(
                "broken-marker",
                lesson,
                rel_display(repo, &path),
                1,
                "marked DELIBERATELY BROKEN but does not say what to expect — write \
                 `//! DELIBERATELY BROKEN — expected: E0382`, or describe the error when it \
                 has no code",
            ));
        } else if expectation.starts_with('E') && !looks_like_error_code(expectation) {
            findings.push(Finding::new(
                "broken-marker",
                lesson,
                rel_display(repo, &path),
                1,
                format!("`expected: {expectation}` does not look like an error code (`E0382`)"),
            ));
        }
    }
    findings
}

/// `E` followed by four digits, as `rustc` writes them.
fn looks_like_error_code(text: &str) -> bool {
    let code: String = text.chars().take(5).collect();
    let mut chars = code.chars();
    chars.next() == Some('E') && chars.clone().count() == 4 && chars.all(|c| c.is_ascii_digit())
}

fn label<'a>(pair: (&'a str, &'a str), locale: Locale) -> &'a str {
    match locale {
        Locale::En => pair.0,
        Locale::Fa => pair.1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn lesson(has_code: bool) -> Lesson {
        Lesson {
            path: "phase1/01-a/01-x".to_string(),
            dir: PathBuf::from("phase1/01-a/01-x"),
            order: 0,
            has_code,
        }
    }

    fn full_en() -> String {
        let mut out = String::from("# Title\n\n");
        for (en, _) in SECTIONS {
            out.push_str(&format!("## {en}\n\nbody\n\n"));
        }
        out
    }

    #[test]
    fn a_complete_lesson_passes_the_section_check() {
        let doc = markdown::parse(&full_en());
        assert!(check_sections(&lesson(true), Locale::En, &doc, "f").is_empty());
    }

    #[test]
    fn a_missing_section_is_reported_once() {
        let source = full_en().replace("## Errors you will meet\n\nbody\n\n", "");
        let doc = markdown::parse(&source);
        let found = check_sections(&lesson(true), Locale::En, &doc, "f");
        assert_eq!(found.len(), 1);
        assert!(found[0].message.contains("Errors you will meet"));
    }

    #[test]
    fn reading_only_lessons_may_skip_the_code_sections() {
        let source = full_en()
            .replace("## Hands on\n\nbody\n\n", "")
            .replace("## Errors you will meet\n\nbody\n\n", "");
        let doc = markdown::parse(&source);
        assert!(check_sections(&lesson(false), Locale::En, &doc, "f").is_empty());
        assert_eq!(
            check_sections(&lesson(true), Locale::En, &doc, "f").len(),
            2
        );
    }

    #[test]
    fn out_of_order_sections_are_reported() {
        let source = "# T\n\n## The concept\n\n## At a glance\n";
        let doc = markdown::parse(source);
        let found = check_sections(&lesson(false), Locale::En, &doc, "f");
        assert!(found.iter().any(|f| f.message.contains("out of order")));
    }

    #[test]
    fn a_rung_may_carry_a_qualifier_in_its_heading() {
        let source = "## Exercises

### Warm up

### Repair

### Implement

                      ### Build

### Challenge (optional)

## Wrapping up
";
        let doc = markdown::parse(source);
        assert!(check_exercise_ladder(&lesson(true), Locale::En, &doc, "f").is_empty());
    }

    #[test]
    fn the_ladder_must_have_all_five_rungs_in_order() {
        let source =
            "## Exercises\n\n### Warm up\n\n### Implement\n\n### Repair\n\n## Wrapping up\n";
        let doc = markdown::parse(source);
        let found = check_exercise_ladder(&lesson(true), Locale::En, &doc, "f");
        let messages: Vec<&str> = found.iter().map(|f| f.message.as_str()).collect();
        assert!(messages.iter().any(|m| m.contains("### Build")));
        assert!(messages.iter().any(|m| m.contains("### Challenge")));
        assert!(messages.iter().any(|m| m.contains("out of order")));
    }

    #[test]
    fn error_codes_are_recognised_only_in_rustcs_shape() {
        assert!(looks_like_error_code("E0382"));
        assert!(looks_like_error_code("E0308 — mismatched types"));
        assert!(!looks_like_error_code("Erorr"));
        assert!(!looks_like_error_code("E38"));
    }

    #[test]
    fn a_todo_that_hands_over_the_answer_is_flagged() {
        let root = std::env::temp_dir().join(format!("lesson-lint-todo-{}", std::process::id()));
        let src = root.join("src");
        std::fs::create_dir_all(&src).unwrap();
        std::fs::write(
            src.join("lib.rs"),
            "pub fn f(s: &str) {\n    todo!(\"s.parse::<u32>().map_err(|e| e.to_string())\")\n}\n",
        )
        .unwrap();
        let lesson = Lesson {
            path: "x".into(),
            dir: root.clone(),
            order: 0,
            has_code: true,
        };
        let found = check_todo_messages(&root, &lesson);
        assert_eq!(found.len(), 1);
        assert!(found[0].message.contains("giving the answer"));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn parity_notices_a_language_that_drifted() {
        // Same heading shape, but only one language carries the code.
        let fa = markdown::parse("# T\n\n## در یک نگاه\n\n```rust\nlet x = 1;\n```\n");
        let en = markdown::parse("# T\n\n## At a glance\n");
        let found = check_parity(&lesson(true), &fa, &en);
        assert_eq!(found.len(), 1);
        assert!(found[0].message.contains("Rust code blocks"));

        // A subsection present in one language and missing in the other.
        let fa = markdown::parse("# T\n\n## در یک نگاه\n\n### اضافه\n");
        let found = check_parity(&lesson(true), &fa, &en);
        assert_eq!(found.len(), 1);
        assert!(found[0].message.contains("heading structure"));
    }

    #[test]
    fn parity_is_quiet_when_both_languages_match() {
        let fa = markdown::parse("# T\n\n## در یک نگاه\n\n```rust\nlet x = 1;\n```\n");
        let en = markdown::parse("# T\n\n## At a glance\n\n```rust\nlet x = 1;\n```\n");
        assert!(check_parity(&lesson(true), &fa, &en).is_empty());
    }
}
