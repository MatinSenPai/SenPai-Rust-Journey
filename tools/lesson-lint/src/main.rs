//! `lesson-lint` — the machine half of `docs/lesson-standard.md`.
//!
//! ```sh
//! cargo run -p lesson-lint                     # the whole curriculum
//! cargo run -p lesson-lint -- phase1-fundamentals
//! cargo run -p lesson-lint -- --list-pending   # what is still on the allowlist
//! ```
//!
//! # The allowlist
//!
//! The curriculum is being rebuilt one phase at a time (see
//! `plans/005-curriculum-rebuild.md`). Lessons listed in
//! `docs/lesson-lint-allow.txt` have not been rebuilt yet: their findings are
//! counted and summarised but do not fail the run. Migrating a lesson means
//! bringing it up to the standard *and* deleting its line from that file, so
//! the allowlist can only ever shrink.
//!
//! Everything not on the allowlist must pass, which is what keeps CI honest
//! while roughly 180 lessons are rewritten.

mod checks;
mod concepts;
mod markdown;
mod model;
mod rtl;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use checks::Finding;
use model::{rel_display, Lesson, Locale};

const ALLOWLIST: &str = "docs/lesson-lint-allow.txt";

fn main() {
    let options = Options::parse();
    let repo = options.repo.clone();

    let mut findings: Vec<Finding> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    // Repo-wide first: these are not about any one lesson.
    findings.extend(check_no_checkpoints(&repo));

    let all_lessons = model::discover(&repo);
    if all_lessons.is_empty() {
        eprintln!("lesson-lint: no lessons found under {}", repo.display());
        eprintln!("             is {} the repo root?", repo.display());
        std::process::exit(2);
    }

    let map = match concepts::load(&repo) {
        Ok(map) => map,
        Err(err) => {
            eprintln!("lesson-lint: {err}");
            std::process::exit(2);
        }
    };

    let pending = map.pending(&all_lessons);
    if !pending.is_empty() {
        notes.push(format!(
            "{} concepts point at lessons that do not exist yet (seeded ahead of content); \
             their ordering is not enforced",
            pending.len()
        ));
    }
    if options.list_pending {
        for entry in &pending {
            println!("PENDING  {:<24} -> {}", entry.id, entry.introduced_in);
        }
        return;
    }
    if options.list_concepts {
        for concept in &map.concepts {
            let state = if pending.iter().any(|p| p.id == concept.id) {
                "pending"
            } else {
                "placed "
            };
            println!(
                "{state}  {:<24} {:<44} {}",
                concept.id, concept.en, concept.fa
            );
        }
        return;
    }

    if options.fix_rtl_code {
        fix_rtl_code(&repo);
        return;
    }

    findings.extend(check_rtl_code(&repo, &all_lessons));

    let allowed = read_allowlist(&repo);
    let selected: Vec<&Lesson> = all_lessons
        .iter()
        .filter(|lesson| match &options.scope {
            Some(scope) => lesson.path.starts_with(scope.as_str()),
            None => true,
        })
        .collect();

    if selected.is_empty() {
        eprintln!(
            "lesson-lint: no lessons matched `{}`",
            options.scope.unwrap_or_default()
        );
        std::process::exit(2);
    }

    for lesson in &selected {
        findings.extend(lint_lesson(&repo, lesson, &all_lessons, &map));
    }

    report(&findings, &selected, &allowed, &notes, &options);
}

fn lint_lesson(
    repo: &Path,
    lesson: &Lesson,
    all: &[Lesson],
    map: &concepts::ConceptMap,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut docs: BTreeMap<&'static str, markdown::Document> = BTreeMap::new();

    for locale in Locale::BOTH {
        let path = lesson.readme(locale);
        let display = rel_display(repo, &path);
        let Ok(source) = std::fs::read_to_string(&path) else {
            findings.push(Finding {
                rule: "missing-readme",
                lesson: lesson.path.clone(),
                file: display,
                line: 0,
                message: format!(
                    "no `{}` — Persian is the canonical text and English is its mirror; both \
                     are required",
                    locale.readme()
                ),
            });
            continue;
        };
        let doc = markdown::parse(&source);

        findings.extend(checks::check_sections(lesson, locale, &doc, &display));
        findings.extend(checks::check_exercise_ladder(
            lesson, locale, &doc, &display,
        ));
        findings.extend(checks::check_wrap_up(lesson, locale, &doc, &display));
        findings.extend(checks::check_prerequisites(
            lesson, locale, &source, &display,
        ));
        findings.extend(checks::check_links(
            repo, lesson, &doc, &path, &display, locale,
        ));
        docs.insert(locale.label(), doc);
    }

    if let (Some(fa), Some(en)) = (docs.get("fa"), docs.get("en")) {
        findings.extend(checks::check_parity(lesson, fa, en));
    }

    findings.extend(checks::check_examples_exist(lesson));
    findings.extend(checks::check_broken_markers(repo, lesson));
    findings.extend(checks::check_todo_messages(repo, lesson));
    findings.extend(check_concept_order(repo, lesson, all, map));
    findings
}

/// Concept ordering runs over the lesson's Rust files *and* the Rust fences in
/// its READMEs — a forward reference shown in prose teaches it just as well as
/// one in the starter code.
fn check_concept_order(
    repo: &Path,
    lesson: &Lesson,
    all: &[Lesson],
    map: &concepts::ConceptMap,
) -> Vec<Finding> {
    let mut sources: Vec<(String, String)> = Vec::new();

    for path in lesson.rust_files() {
        if let Ok(body) = std::fs::read_to_string(&path) {
            sources.push((rel_display(repo, &path), body));
        }
    }
    for locale in Locale::BOTH {
        let path = lesson.readme(locale);
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let display = rel_display(repo, &path);
        for fence in markdown::parse(&source).fences {
            if fence.lang == "rust" {
                sources.push((format!("{display}:{}", fence.line), fence.body));
            }
        }
    }

    map.check(lesson, all, &sources)
        .into_iter()
        .map(|violation| Finding {
            rule: "concept-order",
            lesson: lesson.path.clone(),
            file: violation.file,
            line: violation.line,
            message: format!(
                "uses `{}` ({}) but that is introduced later, in `{}` — move the concept \
                 earlier, teach the minimum inline and register this lesson under its \
                 `deepened_in`, or avoid it",
                violation.pattern, violation.concept_en, violation.introduced_in
            ) + &format!(" [concept-map id: {}]", violation.concept_id),
        })
        .collect()
}

/// Persian prose lives everywhere, not just in lessons, so the bidi check
/// sweeps the documentation and the capstone too.
const RTL_AREAS: &[&str] = &["docs", "capstone-taskforge"];

fn rtl_files(repo: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for area in model::ROOTS.iter().chain(RTL_AREAS.iter()) {
        walk_files(&repo.join(area), &mut |path| {
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".fa.md"))
            {
                out.push(path.to_path_buf());
            }
        });
    }
    out.sort();
    out
}

/// The English companion a Persian file mirrors: `X.fa.md` -> `X.md`.
fn english_companion(fa: &Path) -> Option<PathBuf> {
    let name = fa.file_name()?.to_str()?;
    let stem = name.strip_suffix(".fa.md")?;
    let companion = fa.with_file_name(format!("{stem}.md"));
    companion.is_file().then_some(companion)
}

/// R11 — inline code in Persian prose was not saved in visual order.
/// See `rtl.rs` for why this happens and how the repair is derived.
fn check_rtl_code(repo: &Path, lessons: &[Lesson]) -> Vec<Finding> {
    let mut findings = Vec::new();
    for path in rtl_files(repo) {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let display = rel_display(repo, &path);
        let owner = owning_lesson(lessons, &display);
        let english = english_companion(&path)
            .and_then(|p| std::fs::read_to_string(p).ok())
            .map(|text| rtl::index(&rtl::code_spans(&text)))
            .unwrap_or_default();

        let mut reported: Vec<String> = Vec::new();
        for span in rtl::code_spans(&source) {
            let Some(reason) = rtl::looks_mangled(&span.text) else {
                continue;
            };
            if reported.contains(&span.text) {
                continue;
            }
            reported.push(span.text.clone());
            let advice = match rtl::suggest(&span.text, &english) {
                Some(fixed) => format!(
                    "should be `{fixed}` (from the English companion) — run \
                     `cargo run -p lesson-lint -- --fix-rtl-code`"
                ),
                None => "no unambiguous original in the English companion; fix by hand".to_string(),
            };
            findings.push(Finding {
                rule: "rtl-code",
                lesson: owner.clone(),
                file: display.clone(),
                line: span.line,
                message: format!(
                    "inline code `{}` was saved in visual order ({reason}) — {advice}",
                    span.text
                ),
            });
        }
    }
    findings
}

fn owning_lesson(lessons: &[Lesson], file: &str) -> String {
    lessons
        .iter()
        .filter(|lesson| file.starts_with(&format!("{}/", lesson.path)))
        .max_by_key(|lesson| lesson.path.len())
        .map(|lesson| lesson.path.clone())
        .unwrap_or_default()
}

/// Apply every unambiguous bidi repair across the repository.
fn fix_rtl_code(repo: &Path) {
    let mut files = 0usize;
    let mut repairs = 0usize;
    for path in rtl_files(repo) {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Some(english) = english_companion(&path) else {
            continue;
        };
        let Ok(english_text) = std::fs::read_to_string(english) else {
            continue;
        };
        let index = rtl::index(&rtl::code_spans(&english_text));
        let (fixed, made) = rtl::repair(&source, &index);
        if made.is_empty() {
            continue;
        }
        if let Err(err) = std::fs::write(&path, &fixed) {
            eprintln!("lesson-lint: could not write {}: {err}", path.display());
            continue;
        }
        files += 1;
        repairs += made.len();
        println!("{}", rel_display(repo, &path));
        for (from, to) in made {
            println!("  `{from}`  ->  `{to}`");
        }
    }
    println!("\n─────────────────────────────────────────────");
    println!("repaired {repairs} inline code spans across {files} files");
    println!("spans with no unambiguous original are left alone and still reported");
}

/// R1 — `CHECKPOINT.md` was removed from the curriculum in full. Recall lives
/// in `## Wrapping up`'s self-assessment list instead.
fn check_no_checkpoints(repo: &Path) -> Vec<Finding> {
    let mut found = Vec::new();
    for root in model::ROOTS {
        walk_files(&repo.join(root), &mut |path| {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name.starts_with("CHECKPOINT") && name.ends_with(".md") {
                found.push(Finding {
                    rule: "no-checkpoints",
                    lesson: String::new(),
                    file: rel_display(repo, path),
                    line: 0,
                    message: "CHECKPOINT files were removed from this curriculum — put the \
                              recall prompts in `## Wrapping up` -> `### Can you explain?`"
                        .to_string(),
                });
            }
        });
    }
    found
}

fn walk_files(dir: &Path, visit: &mut impl FnMut(&Path)) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk_files(&path, visit);
        } else {
            visit(&path);
        }
    }
}

// -------------------------------------------------------------------- report

fn report(
    findings: &[Finding],
    selected: &[&Lesson],
    allowed: &[String],
    notes: &[String],
    options: &Options,
) {
    let is_allowed =
        |finding: &Finding| !finding.lesson.is_empty() && allowed.contains(&finding.lesson);
    let (deferred, blocking): (Vec<&Finding>, Vec<&Finding>) =
        findings.iter().partition(|f| is_allowed(f));

    let mut by_lesson: BTreeMap<&str, Vec<&Finding>> = BTreeMap::new();
    let shown = if options.show_deferred {
        findings.iter().collect::<Vec<_>>()
    } else {
        blocking.clone()
    };
    for finding in shown {
        by_lesson.entry(&finding.lesson).or_default().push(finding);
    }

    for (lesson, items) in &by_lesson {
        let header = if lesson.is_empty() { "(repo)" } else { lesson };
        println!("\n{header}");
        for finding in items {
            let where_at = if finding.line == 0 {
                finding.file.clone()
            } else {
                format!("{}:{}", finding.file, finding.line)
            };
            println!("  [{}] {where_at}\n      {}", finding.rule, finding.message);
        }
    }

    let migrated = selected
        .iter()
        .filter(|lesson| !allowed.contains(&lesson.path))
        .count();

    println!("\n─────────────────────────────────────────────");
    println!("lessons checked      {}", selected.len());
    println!(
        "at the standard      {migrated}  (not on the allowlist)\npending migration    {}",
        selected.len() - migrated
    );
    println!("blocking findings    {}", blocking.len());
    if !deferred.is_empty() && !options.show_deferred {
        println!(
            "deferred findings    {}  (allowlisted — rerun with --show-deferred)",
            deferred.len()
        );
    }
    for note in notes {
        println!("note: {note}");
    }

    if blocking.is_empty() {
        println!("\nOK");
    } else {
        println!("\nFAILED — {} blocking findings", blocking.len());
        std::process::exit(1);
    }
}

fn read_allowlist(repo: &Path) -> Vec<String> {
    let path = repo.join(ALLOWLIST.replace('/', std::path::MAIN_SEPARATOR_STR));
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

// -------------------------------------------------------------------- startup

struct Options {
    repo: PathBuf,
    scope: Option<String>,
    show_deferred: bool,
    list_pending: bool,
    list_concepts: bool,
    fix_rtl_code: bool,
}

impl Options {
    fn parse() -> Self {
        let mut options = Options {
            repo: default_repo(),
            scope: None,
            show_deferred: false,
            list_pending: false,
            list_concepts: false,
            fix_rtl_code: false,
        };
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--show-deferred" => options.show_deferred = true,
                "--list-pending" => options.list_pending = true,
                "--list-concepts" => options.list_concepts = true,
                "--fix-rtl-code" => options.fix_rtl_code = true,
                "--root" => options.repo = args.next().map(PathBuf::from).unwrap_or(options.repo),
                "-h" | "--help" => {
                    println!(
                        "usage: cargo run -p lesson-lint -- [<path-prefix>] [--show-deferred] \
                         [--list-pending] [--list-concepts] [--fix-rtl-code]                          [--root <dir>]"
                    );
                    std::process::exit(0);
                }
                other if other.starts_with('-') => {
                    eprintln!("lesson-lint: unknown flag `{other}`");
                    std::process::exit(2);
                }
                other => options.scope = Some(other.trim_end_matches('/').to_string()),
            }
        }
        options
    }
}

/// The repo root is two directories above this crate (`tools/lesson-lint/`).
fn default_repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}
