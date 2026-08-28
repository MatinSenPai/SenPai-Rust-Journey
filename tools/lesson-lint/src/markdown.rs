//! Just enough Markdown structure to lint with: headings, fenced code blocks,
//! and links. Deliberately not a parser — the checks only need to know what is
//! there and in what order, and a line scanner that respects fences is both
//! easier to reason about and immune to the differences between renderers.

#[derive(Debug, Clone)]
pub struct Heading {
    pub level: usize,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Fence {
    pub lang: String,
    pub body: String,
    /// 1-based line of the opening fence.
    pub line: usize,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub dest: String,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct Document {
    pub headings: Vec<Heading>,
    pub fences: Vec<Fence>,
    pub links: Vec<Link>,
}

pub fn parse(source: &str) -> Document {
    let mut doc = Document::default();
    let mut fence: Option<(String, String, usize)> = None;
    let mut fence_marker = String::new();

    for (index, raw) in source.lines().enumerate() {
        let line = index + 1;
        let trimmed = raw.trim_start();

        if let Some((lang, body, start)) = fence.as_mut() {
            if is_fence_close(trimmed, &fence_marker) {
                doc.fences.push(Fence {
                    lang: std::mem::take(lang),
                    body: std::mem::take(body),
                    line: *start,
                });
                fence = None;
                fence_marker.clear();
            } else {
                body.push_str(raw);
                body.push('\n');
            }
            continue;
        }

        if let Some(marker) = fence_open(trimmed) {
            fence_marker = marker.to_string();
            let lang = trimmed[marker.len()..].trim().to_string();
            fence = Some((lang, String::new(), line));
            continue;
        }

        if let Some(rest) = heading(trimmed) {
            doc.headings.push(Heading {
                level: rest.0,
                text: normalize(rest.1),
                line,
            });
        }

        for dest in links_in(raw) {
            doc.links.push(Link { dest, line });
        }
    }

    // An unterminated fence is malformed markdown, but keeping what we saw is
    // more useful to the reader than silently dropping it.
    if let Some((lang, body, start)) = fence {
        doc.fences.push(Fence {
            lang,
            body,
            line: start,
        });
    }
    doc
}

fn fence_open(trimmed: &str) -> Option<&'static str> {
    // Longest marker first: a ```` fence opens with ``` too, and matching the
    // short one would close it on the wrong line.
    ["````", "```", "~~~"]
        .into_iter()
        .find(|marker| trimmed.starts_with(marker))
}

fn is_fence_close(trimmed: &str, marker: &str) -> bool {
    trimmed.starts_with(marker) && trimmed[marker.len()..].trim().is_empty()
}

fn heading(trimmed: &str) -> Option<(usize, &str)> {
    let hashes = trimmed.chars().take_while(|c| *c == '#').count();
    if hashes == 0 || hashes > 6 {
        return None;
    }
    let rest = &trimmed[hashes..];
    if !rest.starts_with(' ') {
        return None;
    }
    Some((hashes, rest.trim()))
}

/// Inline `[text](dest)` links. Reference-style links are not used anywhere in
/// this repo; if that changes, this is where to extend.
///
/// Inline code spans are skipped. Persian prose quotes Python and SQL inline,
/// and a snippet like `` `q.filter(x)[:20]` `` otherwise reads as a link with a
/// nonsense destination.
fn links_in(line: &str) -> Vec<String> {
    let bytes: Vec<char> = line.chars().collect();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == '`' {
            index = match find_from(&bytes, index + 1, '`') {
                Some(close) => close + 1,
                None => bytes.len(),
            };
            continue;
        }
        if bytes[index] != '[' {
            index += 1;
            continue;
        }
        let Some(close) = find_from(&bytes, index + 1, ']') else {
            break;
        };
        if bytes.get(close + 1) != Some(&'(') {
            index = close + 1;
            continue;
        }
        let Some(paren) = find_from(&bytes, close + 2, ')') else {
            break;
        };
        let dest: String = bytes[close + 2..paren].iter().collect();
        // `[text](dest "title")` — the title is not a destination.
        let dest = dest.split_whitespace().next().unwrap_or("").to_string();
        if !dest.is_empty() {
            out.push(dest);
        }
        index = paren + 1;
    }
    out
}

fn find_from(chars: &[char], start: usize, needle: char) -> Option<usize> {
    (start..chars.len()).find(|index| chars[*index] == needle)
}

/// Compare headings without tripping over invisible or decorative characters.
///
/// Persian prose uses ZWNJ (`\u{200c}`) inside single words — `جمع‌بندی` is one
/// word with a ZWNJ in the middle — and whether an author typed it is not
/// something a lint should have an opinion about. Backticks and emphasis are
/// stripped for the same reason.
pub fn normalize(text: &str) -> String {
    text.chars()
        .filter(|c| !matches!(c, '\u{200c}' | '\u{200e}' | '\u{200f}' | '`' | '*' | '_'))
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn headings_inside_fences_are_not_headings() {
        let doc = parse("# Real\n\n```sh\n# a shell comment\n```\n\n## Also real\n");
        let texts: Vec<&str> = doc.headings.iter().map(|h| h.text.as_str()).collect();
        assert_eq!(texts, vec!["Real", "Also real"]);
    }

    #[test]
    fn nested_fences_close_on_their_own_marker() {
        let doc = parse("````markdown\n```rust\nlet x = 5;\n```\n````\n\n## After\n");
        assert_eq!(doc.fences.len(), 1);
        assert_eq!(doc.fences[0].lang, "markdown");
        assert!(doc.fences[0].body.contains("let x = 5;"));
        assert_eq!(doc.headings.len(), 1, "the outer fence swallowed the inner");
    }

    #[test]
    fn zwnj_and_backticks_do_not_change_a_heading() {
        assert_eq!(normalize("جمع\u{200c}بندی"), normalize("جمعبندی"));
        assert_eq!(normalize("`Option`, `Result`"), "Option, Result");
    }

    #[test]
    fn extracts_link_destinations_without_titles() {
        let doc = parse("see [a](../x/README.md) and [b](https://e.com \"T\")\n");
        let dests: Vec<&str> = doc.links.iter().map(|l| l.dest.as_str()).collect();
        assert_eq!(dests, vec!["../x/README.md", "https://e.com"]);
    }

    #[test]
    fn code_spans_are_not_links() {
        let doc = parse("query `Post.objects.filter(x)[:20](y)` then [real](a.md)\n");
        let dests: Vec<&str> = doc.links.iter().map(|l| l.dest.as_str()).collect();
        assert_eq!(dests, vec!["a.md"]);
    }
}
