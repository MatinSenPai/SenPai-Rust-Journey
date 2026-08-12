//! Small, deterministic server-side search over localized Markdown.

use std::path::Path;

use crate::locale::Locale;
use crate::tree::{self, Node};

pub struct SearchHit<'a> {
    pub node: &'a Node,
    pub snippet: String,
    score: usize,
}

pub fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .replace('ي', "ی")
        .replace('ك', "ک")
        .replace(['\u{200c}', '\u{200f}', '\u{200e}'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn find<'a>(root: &Path, tree: &'a Node, locale: Locale, query: &str) -> Vec<SearchHit<'a>> {
    let needle = normalize(query);
    if needle.len() < 2 {
        return Vec::new();
    }
    let mut hits = Vec::new();
    for node in tree
        .nodes()
        .into_iter()
        .filter(|node| !node.path.is_empty())
    {
        let mut body = String::new();
        for page in &node.pages {
            let canonical = root.join(&node.path).join(&page.file);
            let localized = tree::localized_path(&canonical, locale);
            if let Ok(text) = std::fs::read_to_string(localized) {
                body.push_str(&text);
                body.push('\n');
            }
        }
        let title = normalize(&node.title);
        let normalized = normalize(&body);
        let title_match = title.contains(&needle);
        let body_match = normalized.contains(&needle);
        if title_match || body_match {
            hits.push(SearchHit {
                node,
                snippet: snippet(&body, &needle),
                score: usize::from(title_match) * 10 + normalized.matches(&needle).count(),
            });
        }
    }
    hits.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.node.path.cmp(&right.node.path))
    });
    hits.truncate(50);
    hits
}

fn snippet(markdown: &str, needle: &str) -> String {
    let plain = markdown
        .lines()
        .filter(|line| !line.trim_start().starts_with("```") && !line.trim().is_empty())
        .map(|line| line.trim_start_matches(['#', '-', '>', ' ']))
        .collect::<Vec<_>>()
        .join(" ");
    let normalized = normalize(&plain);
    let start = normalized.find(needle).unwrap_or(0).saturating_sub(70);
    let excerpt: String = normalized.chars().skip(start).take(180).collect();
    if excerpt.is_empty() {
        plain.chars().take(180).collect()
    } else {
        excerpt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_persian_variants_and_half_spaces() {
        assert_eq!(normalize("مالكيت‌ داده"), "مالکیت داده");
    }
}
