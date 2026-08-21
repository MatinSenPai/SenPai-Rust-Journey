//! Loading `docs/concept-map.toml` and answering the one question it exists
//! for: *is this lesson using something a later lesson teaches?*

use std::collections::HashMap;
use std::path::Path;

use toml::Value;

use crate::model::Lesson;

pub const MAP_FILE: &str = "docs/concept-map.toml";

#[derive(Debug, Clone)]
pub struct Concept {
    pub id: String,
    pub en: String,
    pub fa: String,
    pub introduced_in: String,
    pub deepened_in: Vec<String>,
    pub mastered_in: Option<String>,
    /// Lessons allowed to *use* the concept before it is taught, without
    /// claiming to teach it. Phase 0 shows a working program containing
    /// `String` and `&str` — pretending that teaches them would light them up
    /// in the mastery grid, and forbidding them outright would mean a first
    /// program that prints nothing useful.
    pub previewed_in: Vec<String>,
    pub detect: Vec<String>,
}

impl Concept {
    /// Lessons allowed to use this concept regardless of order: the one that
    /// introduces it, the ones that deepen or master it, and the ones that
    /// merely preview it.
    fn permits(&self, lesson_path: &str) -> bool {
        self.introduced_in == lesson_path
            || self.deepened_in.iter().any(|p| p == lesson_path)
            || self.previewed_in.iter().any(|p| p == lesson_path)
            || self.mastered_in.as_deref() == Some(lesson_path)
    }
}

pub struct ConceptMap {
    pub concepts: Vec<Concept>,
}

/// A concept whose `introduced_in` lesson does not exist on disk yet. Reported,
/// not failed — the map is deliberately seeded ahead of the content that fills
/// it in (see `docs/lesson-standard.md` §8).
pub struct Pending {
    pub id: String,
    pub introduced_in: String,
}

#[derive(Debug)]
pub struct Violation {
    pub concept_id: String,
    pub concept_en: String,
    pub pattern: String,
    pub file: String,
    pub line: usize,
    pub introduced_in: String,
}

pub fn load(repo: &Path) -> Result<ConceptMap, String> {
    let path = repo.join(MAP_FILE.replace('/', std::path::MAIN_SEPARATOR_STR));
    let text = std::fs::read_to_string(&path)
        .map_err(|err| format!("could not read {}: {err}", path.display()))?;
    parse(&text)
}

pub fn parse(text: &str) -> Result<ConceptMap, String> {
    let value: Value = text.parse().map_err(|err| format!("{MAP_FILE}: {err}"))?;
    let entries = value
        .get("concept")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{MAP_FILE}: expected an array of [[concept]] tables"))?;

    let mut concepts = Vec::new();
    for entry in entries {
        let id = string_field(entry, "id")?;
        concepts.push(Concept {
            en: string_field(entry, "en").unwrap_or_else(|_| id.clone()),
            fa: string_field(entry, "fa").unwrap_or_else(|_| id.clone()),
            introduced_in: string_field(entry, "introduced_in")
                .map_err(|err| format!("concept `{id}`: {err}"))?,
            deepened_in: string_array(entry, "deepened_in"),
            previewed_in: string_array(entry, "previewed_in"),
            mastered_in: string_field(entry, "mastered_in").ok(),
            detect: string_array(entry, "detect"),
            id,
        });
    }

    let mut seen = HashMap::new();
    for concept in &concepts {
        if seen.insert(concept.id.clone(), ()).is_some() {
            return Err(format!("{MAP_FILE}: duplicate concept id `{}`", concept.id));
        }
    }
    Ok(ConceptMap { concepts })
}

fn string_field(value: &Value, key: &str) -> Result<String, String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("missing string field `{key}`"))
}

fn string_array(value: &Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

impl ConceptMap {
    /// Concepts pointing at a lesson that has not been written yet.
    pub fn pending(&self, lessons: &[Lesson]) -> Vec<Pending> {
        self.concepts
            .iter()
            .filter(|concept| !lessons.iter().any(|l| l.path == concept.introduced_in))
            .map(|concept| Pending {
                id: concept.id.clone(),
                introduced_in: concept.introduced_in.clone(),
            })
            .collect()
    }

    /// Every use of a concept that this lesson comes *before*.
    ///
    /// `sources` is `(display path, contents)` for each file to scan. Only the
    /// first hit per concept per file is reported: once you know a lesson
    /// reaches forward for `Vec`, twenty more `Vec<` lines add nothing.
    pub fn check(
        &self,
        lesson: &Lesson,
        lessons: &[Lesson],
        sources: &[(String, String)],
    ) -> Vec<Violation> {
        let order = |path: &str| lessons.iter().find(|l| l.path == path).map(|l| l.order);
        let mut out = Vec::new();

        for concept in &self.concepts {
            if concept.detect.is_empty() || concept.permits(&lesson.path) {
                continue;
            }
            let Some(introduced_order) = order(&concept.introduced_in) else {
                continue; // pending — reported separately, never failed on
            };
            if introduced_order <= lesson.order {
                continue;
            }
            for (file, body) in sources {
                if let Some((pattern, line)) = first_hit(body, &concept.detect) {
                    out.push(Violation {
                        concept_id: concept.id.clone(),
                        concept_en: concept.en.clone(),
                        pattern,
                        file: file.clone(),
                        line,
                        introduced_in: concept.introduced_in.clone(),
                    });
                    break;
                }
            }
        }
        out
    }
}

/// First line matching any pattern, ignoring comments and string literals.
///
/// Both exclusions are load-bearing. A doc comment saying "we'll meet `Vec<T>`
/// later" is not a use of `Vec`; neither is the word "for" inside
/// `println!("Same four exist for sub, mul, ...")`, which is what prompted the
/// string handling.
fn first_hit(body: &str, patterns: &[String]) -> Option<(String, usize)> {
    for (index, raw) in body.lines().enumerate() {
        let line = raw.trim_start();
        if line.starts_with("//") || line.starts_with("/*") || line.starts_with('*') {
            continue;
        }
        let code = strip_string_literals(strip_trailing_comment(line));
        for pattern in patterns {
            if code.contains(pattern.as_str()) {
                return Some((pattern.clone(), index + 1));
            }
        }
    }
    None
}

/// Blank out the contents of double-quoted literals, keeping the quotes so
/// that patterns anchored on them still behave. Escapes are honoured; raw
/// strings are not, which is fine — lesson code barely uses them, and the cost
/// of getting one wrong is a false positive the author can see and fix.
fn strip_string_literals(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut inside = false;
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' if inside => escaped = true,
            '"' => {
                inside = !inside;
                out.push('"');
            }
            _ if inside => {}
            _ => out.push(character),
        }
    }
    out
}

fn strip_trailing_comment(line: &str) -> &str {
    match line.find("//") {
        // Not a comment if it is inside a string literal — cheap approximation:
        // an odd number of quotes before it means we are inside one.
        Some(at) if line[..at].matches('"').count().is_multiple_of(2) => &line[..at],
        _ => line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn lesson(path: &str, order: usize) -> Lesson {
        Lesson {
            path: path.to_string(),
            dir: PathBuf::from(path),
            order,
            has_code: true,
        }
    }

    fn map() -> ConceptMap {
        parse(
            r#"
[[concept]]
id = "vec"
en = "Vec<T>"
fa = "وکتور"
introduced_in = "phase1/01-b/01-vec"
detect = ["Vec<", "vec!["]
"#,
        )
        .unwrap()
    }

    #[test]
    fn flags_a_concept_used_before_the_lesson_that_teaches_it() {
        let lessons = vec![
            lesson("phase1/01-a/01-early", 0),
            lesson("phase1/01-b/01-vec", 1),
        ];
        let sources = vec![("src/lib.rs".into(), "fn f(v: Vec<String>) {}\n".into())];
        let found = map().check(&lessons[0], &lessons, &sources);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].concept_id, "vec");
        assert_eq!(found[0].pattern, "Vec<");
    }

    #[test]
    fn the_teaching_lesson_and_later_ones_are_free_to_use_it() {
        let lessons = vec![
            lesson("phase1/01-a/01-early", 0),
            lesson("phase1/01-b/01-vec", 1),
        ];
        let sources = vec![("src/lib.rs".into(), "let v: Vec<u8> = vec![];\n".into())];
        assert!(map().check(&lessons[1], &lessons, &sources).is_empty());
    }

    #[test]
    fn mentioning_a_future_concept_in_a_comment_is_allowed() {
        let lessons = vec![
            lesson("phase1/01-a/01-early", 0),
            lesson("phase1/01-b/01-vec", 1),
        ];
        let sources = vec![(
            "src/lib.rs".into(),
            "// you'll meet Vec<T> in the next module\nlet x = 5; // not a Vec<u8>\n".into(),
        )];
        assert!(map().check(&lessons[0], &lessons, &sources).is_empty());
    }

    #[test]
    fn a_previewing_lesson_may_use_a_concept_it_does_not_teach() {
        let lessons = vec![
            lesson("phase0/01-tour", 0),
            lesson("phase1/01-a/01-early", 1),
            lesson("phase1/01-b/01-vec", 2),
        ];
        let sources = vec![(
            "src/lib.rs".into(),
            "let v: Vec<u8> = vec![];
"
            .into(),
        )];
        let map = parse(
            r#"
[[concept]]
id = "vec"
en = "Vec<T>"
fa = "وکتور"
introduced_in = "phase1/01-b/01-vec"
previewed_in = ["phase0/01-tour"]
detect = ["Vec<", "vec!["]
"#,
        )
        .unwrap();
        assert!(map.check(&lessons[0], &lessons, &sources).is_empty());
        assert_eq!(
            map.check(&lessons[1], &lessons, &sources).len(),
            1,
            "a lesson that neither teaches nor previews it is still flagged"
        );
    }

    #[test]
    fn a_pattern_inside_a_string_literal_is_not_a_use() {
        let lessons = vec![
            lesson("phase1/01-a/01-early", 0),
            lesson("phase1/01-b/01-vec", 1),
        ];
        let map = parse(
            r#"
[[concept]]
id = "control-flow"
en = "for loops"
fa = "حلقه"
introduced_in = "phase1/01-b/01-vec"
detect = ["for "]
"#,
        )
        .unwrap();

        let prose = vec![(
            "src/lib.rs".into(),
            "println!(\"Same four exist for sub, mul and div\");\n".into(),
        )];
        assert!(map.check(&lessons[0], &lessons, &prose).is_empty());

        let real = vec![("src/lib.rs".into(), "for item in list {}\n".into())];
        assert_eq!(map.check(&lessons[0], &lessons, &real).len(), 1);
    }

    #[test]
    fn escaped_quotes_do_not_end_a_string_early() {
        assert_eq!(
            strip_string_literals(r#"let s = "a \" for b"; x"#),
            r#"let s = ""; x"#
        );
    }

    #[test]
    fn a_concept_pointing_at_an_unwritten_lesson_is_pending_not_a_failure() {
        let lessons = vec![lesson("phase1/01-a/01-early", 0)];
        let sources = vec![("src/lib.rs".into(), "Vec<u8>\n".into())];
        let map = map();
        assert!(map.check(&lessons[0], &lessons, &sources).is_empty());
        assert_eq!(map.pending(&lessons).len(), 1);
    }
}
