//! Detecting — and repairing — inline code that was scrambled by
//! bidirectional text reordering.
//!
//! # What went wrong
//!
//! Persian prose is right-to-left; the code inside it is left-to-right. When
//! a mixed paragraph is *rendered*, the Unicode Bidirectional Algorithm moves
//! the neutral characters at the edges of a Latin run (brackets, `.`, `&`,
//! quotes) to the other end and mirrors them. That is correct display
//! behaviour. It becomes corruption the moment somebody copies the rendered
//! text back into the file, because the saved bytes are now in *visual* order.
//!
//! It happened at scale in this repository. Real examples, verbatim from the
//! Persian lessons:
//!
//! | On disk | Should be |
//! |---|---|
//! | `` `str&` `` | `` `&str` `` |
//! | `` `()bind.` `` | `` `.bind()` `` |
//! | `` `(char_count(s` `` | `` `char_count(s)` `` |
//! | `` `(Ok(None => _` `` | `` `_ => Ok(None)` `` |
//!
//! A lesson that prints `str&` is not a typo — it teaches the wrong syntax.
//!
//! # How the repair works
//!
//! Reordering permutes and mirrors characters but never adds or removes them.
//! So the multiset of characters survives, and if brackets are folded onto
//! their opening form (`)` → `(`) mirroring survives too. That gives a key
//! which is identical for the scrambled Persian span and its intact English
//! original. Where exactly one English span in the companion file shares the
//! key, the repair is unambiguous and mechanical.
//!
//! Where it is ambiguous or the English file has no counterpart, the span is
//! reported and left alone — a wrong "fix" here would be worse than the bug.

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Span {
    pub text: String,
    pub line: usize,
}

/// Inline code spans, skipping fenced blocks (whose contents are LTR-isolated
/// by every renderer and so were never reordered).
pub fn code_spans(source: &str) -> Vec<Span> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for (index, raw) in source.lines().enumerate() {
        let trimmed = raw.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        let chars: Vec<char> = raw.chars().collect();
        let mut position = 0;
        while position < chars.len() {
            if chars[position] != '`' {
                position += 1;
                continue;
            }
            let Some(close) = (position + 1..chars.len()).find(|i| chars[*i] == '`') else {
                break;
            };
            let text: String = chars[position + 1..close].iter().collect();
            if !text.is_empty() {
                out.push(Span {
                    text,
                    line: index + 1,
                });
            }
            position = close + 1;
        }
    }
    out
}

/// Why this span looks like it was captured in visual order, or `None`.
///
/// Tuned to stay quiet on the partial snippets documentation legitimately
/// quotes — `` `.map_err(` `` has one unclosed bracket and is fine.
pub fn looks_mangled(text: &str) -> Option<&'static str> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 3 || !text.chars().any(|c| c.is_ascii_alphabetic()) {
        return None;
    }

    let mut depth: HashMap<char, i32> = HashMap::new();
    let mut closer_before_opener = false;
    for ch in &chars {
        match ch {
            '(' | '[' | '{' => *depth.entry(*ch).or_insert(0) += 1,
            ')' | ']' | '}' => {
                let opener = match ch {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                let slot = depth.entry(opener).or_insert(0);
                if *slot == 0 {
                    closer_before_opener = true;
                } else {
                    *slot -= 1;
                }
            }
            _ => {}
        }
    }

    if depth.values().sum::<i32>() >= 2 {
        return Some("two or more unclosed brackets");
    }
    if closer_before_opener {
        return Some("a closing bracket appears before its opener");
    }
    // `matches!(x, ...)` and `todo!()` reorder to `!(matches!(x, ...` and
    // `!()todo` — bracket-balanced, so only the leading `!(` gives them away.
    if text.starts_with("!(") {
        return Some("starts with `!(` — a macro bang moved to the front");
    }
    match chars.last() {
        Some('&') => Some("ends with `&` — a reference sigil moved to the wrong end"),
        // `#[derive(Debug)]` reorders to `[derive(Debug)]#`, which is balanced.
        Some('#') => Some("ends with `#` — an attribute hash moved to the wrong end"),
        // A trailing `..` is an ellipsis or a range, not a displaced method dot.
        Some('.') if chars.get(chars.len().wrapping_sub(2)) != Some(&'.') => {
            Some("ends with `.` — a method dot moved to the wrong end")
        }
        _ => None,
    }
}

/// Fold mirrored brackets together and sort, so a scrambled span and its
/// intact original produce the same key.
fn key(text: &str) -> String {
    let mut chars: Vec<char> = text
        .chars()
        .map(|c| match c {
            ')' => '(',
            ']' => '[',
            '}' => '{',
            '>' => '<',
            other => other,
        })
        .collect();
    chars.sort_unstable();
    chars.into_iter().collect()
}

/// Index an English companion's spans by repair key.
pub fn index(spans: &[Span]) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    for span in spans {
        let entry = map.entry(key(&span.text)).or_default();
        if !entry.contains(&span.text) {
            entry.push(span.text.clone());
        }
    }
    map
}

/// The intact original for a scrambled span, when exactly one candidate
/// matches. Ambiguity means "leave it to a human".
pub fn suggest(scrambled: &str, english: &HashMap<String, Vec<String>>) -> Option<String> {
    match english.get(&key(scrambled)) {
        Some(candidates) if candidates.len() == 1 && candidates[0] != scrambled => {
            Some(candidates[0].clone())
        }
        _ => None,
    }
}

/// Rewrite every scrambled span in `source` that has an unambiguous original.
/// Returns the new text and the repairs made.
pub fn repair(
    source: &str,
    english: &HashMap<String, Vec<String>>,
) -> (String, Vec<(String, String)>) {
    let mut repairs: Vec<(String, String)> = Vec::new();
    for span in code_spans(source) {
        if looks_mangled(&span.text).is_none() {
            continue;
        }
        let Some(fixed) = suggest(&span.text, english) else {
            continue;
        };
        if !repairs.iter().any(|(from, _)| *from == span.text) {
            repairs.push((span.text.clone(), fixed));
        }
    }

    // Longest first: a short scrambled span can be a substring of a longer one,
    // and replacing the short one first would corrupt the longer.
    let mut ordered = repairs.clone();
    ordered.sort_by_key(|(from, _)| std::cmp::Reverse(from.len()));

    let mut out = source.to_string();
    for (from, to) in &ordered {
        out = out.replace(&format!("`{from}`"), &format!("`{to}`"));
    }
    (out, repairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fenced_code_is_left_alone() {
        let spans = code_spans("`inline`\n\n```rust\nlet x = `not a span`;\n```\n\n`after`\n");
        let texts: Vec<&str> = spans.iter().map(|s| s.text.as_str()).collect();
        assert_eq!(texts, vec!["inline", "after"]);
    }

    #[test]
    fn recognises_the_real_corruption_patterns() {
        assert!(looks_mangled("str&").is_some());
        assert!(looks_mangled("()bind.").is_some());
        assert!(looks_mangled("(char_count(s").is_some());
        assert!(looks_mangled("(Ok(None => _").is_some());
    }

    #[test]
    fn recognises_the_balanced_corruptions_too() {
        // Bracket-balanced, so only the displaced `#` and `!` betray them.
        assert!(looks_mangled("[derive(Debug)]#").is_some());
        assert!(looks_mangled("!()todo").is_some());
        assert!(looks_mangled("!(matches!(status, Status::Cancelled").is_some());
    }

    #[test]
    fn stays_quiet_on_legitimate_partial_snippets() {
        assert!(looks_mangled(".map_err(").is_none());
        assert!(looks_mangled("Vec<T>").is_none());
        assert!(looks_mangled("Option<&str>").is_none());
        assert!(looks_mangled("fn main()").is_none());
        assert!(looks_mangled("&str").is_none());
        assert!(looks_mangled("#[derive(Debug)]").is_none());
        // Ellipsis and range ends, not displaced method dots.
        assert!(looks_mangled("?job_type=...&limit=...").is_none());
        assert!(looks_mangled("&v[1..").is_none());
    }

    #[test]
    fn repairs_from_the_english_companion() {
        let english = index(&code_spans("The `&str` type and `char_count(s)` helper.\n"));
        let (fixed, repairs) = repair("نوع `str&` و تابع `(char_count(s`.\n", &english);
        assert!(fixed.contains("`&str`"));
        assert!(fixed.contains("`char_count(s)`"));
        assert_eq!(repairs.len(), 2);
    }

    #[test]
    fn refuses_to_guess_when_two_originals_share_a_key() {
        let english = index(&code_spans("`f(a)` and `(a)f`\n"));
        assert_eq!(suggest("(f(a", &english), None);
    }

    #[test]
    fn a_short_span_inside_a_longer_one_is_replaced_after_it() {
        let english = index(&code_spans("`x.len()` and `total += x.len()`\n"));
        let (fixed, _) = repair("`(x.len(` سپس `(total += x.len(`\n", &english);
        assert!(fixed.contains("`x.len()`"), "{fixed}");
        assert!(fixed.contains("`total += x.len()`"), "{fixed}");
    }
}
