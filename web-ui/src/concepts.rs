//! Reads `docs/concept-map.toml` for the mastery grid on the progress page.
//!
//! This is deliberately a *second*, much smaller reader than the one in
//! `tools/lesson-lint/src/concepts.rs`. The linter needs detection patterns and
//! ordering; the UI needs a name and two lesson paths. Sharing a crate between
//! them would couple a reader-facing page to a CI tool for the sake of about
//! forty lines — if the file's shape ever gets complicated enough that the two
//! disagree, that is the moment to extract it, not before.
//!
//! A missing or malformed file is not an error: the grid simply does not
//! render. The course has to stay readable when its tooling does not load.

use std::path::Path;

use crate::locale::Locale;
use crate::progress::Progress;

pub const MAP_FILE: &str = "docs/concept-map.toml";

pub struct Concept {
    pub id: String,
    pub fa: String,
    pub en: String,
    introduced_in: String,
    mastered_in: Option<String>,
}

/// How far a reader has got with one concept.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mastery {
    /// The lesson that teaches it has not been finished.
    Pending,
    /// Introduced and finished — you have met it.
    Met,
    /// Every lesson that deepens it is done too.
    Mastered,
}

impl Mastery {
    pub const fn slug(self) -> &'static str {
        match self {
            Mastery::Pending => "pending",
            Mastery::Met => "met",
            Mastery::Mastered => "mastered",
        }
    }
}

impl Concept {
    /// The lesson that teaches this concept. A concept whose lesson does not
    /// exist yet is seeded ahead of the content (see
    /// `plans/005-curriculum-rebuild.md`) and is left out of the grid — showing
    /// it as "pending" would read as "you have not learned this" when the truth
    /// is "this lesson has not been written".
    pub fn introduced_in(&self) -> &str {
        &self.introduced_in
    }

    pub fn name(&self, locale: Locale) -> &str {
        if locale.is_fa() {
            &self.fa
        } else {
            &self.en
        }
    }

    pub fn mastery(&self, progress: &Progress) -> Mastery {
        if self
            .mastered_in
            .as_deref()
            .is_some_and(|path| progress.is_complete(path))
        {
            Mastery::Mastered
        } else if progress.is_complete(&self.introduced_in) {
            Mastery::Met
        } else {
            Mastery::Pending
        }
    }
}

pub fn load(root: &Path) -> Vec<Concept> {
    let path = root.join(MAP_FILE.replace('/', std::path::MAIN_SEPARATOR_STR));
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    parse(&text)
}

fn parse(text: &str) -> Vec<Concept> {
    let Ok(value) = text.parse::<toml::Value>() else {
        return Vec::new();
    };
    let Some(entries) = value.get("concept").and_then(toml::Value::as_array) else {
        return Vec::new();
    };
    entries
        .iter()
        .filter_map(|entry| {
            let field = |key: &str| {
                entry
                    .get(key)
                    .and_then(toml::Value::as_str)
                    .map(str::to_string)
            };
            let id = field("id")?;
            Some(Concept {
                fa: field("fa").unwrap_or_else(|| id.clone()),
                en: field("en").unwrap_or_else(|| id.clone()),
                introduced_in: field("introduced_in")?,
                mastered_in: field("mastered_in"),
                id,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map() -> Vec<Concept> {
        parse(
            r#"
[[concept]]
id = "vec"
fa = "وکتور"
en = "Vec<T>"
introduced_in = "p1/a"
mastered_in = "p2/b"

[[concept]]
id = "shadowing"
fa = "سایه‌زنی"
en = "shadowing"
introduced_in = "p1/a"
"#,
        )
    }

    #[test]
    fn a_concept_is_pending_until_its_lesson_is_finished() {
        let concepts = map();
        let progress = Progress::default();
        assert_eq!(concepts[0].mastery(&progress), Mastery::Pending);
    }

    #[test]
    fn finishing_the_introducing_lesson_makes_it_met() {
        let concepts = map();
        let mut progress = Progress::default();
        progress.set("p1/a", true);
        assert_eq!(concepts[0].mastery(&progress), Mastery::Met);
        assert_eq!(
            concepts[1].mastery(&progress),
            Mastery::Met,
            "a concept with no `mastered_in` tops out at met"
        );
    }

    #[test]
    fn finishing_the_mastering_lesson_promotes_it() {
        let concepts = map();
        let mut progress = Progress::default();
        progress.set("p1/a", true);
        progress.set("p2/b", true);
        assert_eq!(concepts[0].mastery(&progress), Mastery::Mastered);
    }

    #[test]
    fn a_malformed_map_yields_no_grid_rather_than_an_error() {
        assert!(parse("this is not toml = = =").is_empty());
        assert!(parse("[[concept]]\nfa = \"بی‌شناسه\"\n").is_empty());
    }

    #[test]
    fn names_follow_the_locale() {
        let concepts = map();
        assert_eq!(concepts[0].name(Locale::Fa), "وکتور");
        assert_eq!(concepts[0].name(Locale::En), "Vec<T>");
    }
}
