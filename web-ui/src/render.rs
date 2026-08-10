//! Markdown -> HTML, with the curriculum's own relative links rewritten to UI
//! routes.
//!
//! The content leans on 222 relative `.md` links to navigate itself, so link
//! rewriting isn't cosmetic — without it the site doesn't work. Four cases:
//!
//! | Link                          | Becomes                                    |
//! |-------------------------------|--------------------------------------------|
//! | `../PROGRESS.md`              | route to that node, `#`-anchored to the file |
//! | `01-move-semantics/`          | route to that node                          |
//! | `https://…`                   | untouched, opens in a new tab               |
//! | `…/src/postgres.rs`           | inert `<code>` — source belongs in your editor |

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

use crate::tree::anchor_for;

/// Render one markdown file to HTML. `base_dir` is the repo-relative directory
/// the file lives in, which is what relative links resolve against.
pub fn to_html(markdown: &str, base_dir: &str) -> String {
    let mut options = Options::empty();
    // 23 files use GFM tables; `PROGRESS.md` uses `- [ ]` task lists. Both are
    // off by default. Raw HTML (its `<details>` blocks) passes through already.
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);

    let mut closings: Vec<&'static str> = Vec::new();
    let events: Vec<Event> = Parser::new_ext(markdown, options)
        .map(|event| match event {
            Event::Start(Tag::Link {
                ref dest_url,
                ref title,
                ..
            }) => {
                let (open, close) = link_html(dest_url, title, base_dir);
                closings.push(close);
                Event::Html(open.into())
            }
            Event::End(TagEnd::Link) => Event::Html(closings.pop().unwrap_or("</a>").into()),
            other => other,
        })
        .collect();

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    html
}

/// The opening tag for a link, plus the closing tag it must be paired with.
fn link_html(dest: &str, title: &str, base_dir: &str) -> (String, &'static str) {
    let title_attr = if title.is_empty() {
        String::new()
    } else {
        format!(" title=\"{}\"", escape(title))
    };

    // Fragment-only and external links are left as the author wrote them.
    if dest.starts_with('#') {
        return (format!("<a href=\"{}\"{title_attr}>", escape(dest)), "</a>");
    }
    if is_external(dest) {
        return (
            format!(
                "<a href=\"{}\"{title_attr} target=\"_blank\" rel=\"noopener noreferrer\">",
                escape(dest)
            ),
            "</a>",
        );
    }

    let (path, fragment) = split_fragment(dest);
    let resolved = resolve(base_dir, path);

    match target_kind(&resolved) {
        Target::Markdown => {
            let (route, anchor) = route_for_markdown(&resolved);
            // An explicit `#fragment` the author wrote wins over our own anchor.
            let anchor = if fragment.is_empty() {
                anchor
            } else {
                fragment.to_string()
            };
            let href = if anchor.is_empty() {
                route
            } else {
                format!("{route}#{anchor}")
            };
            (
                format!("<a href=\"{}\"{title_attr}>", escape(&href)),
                "</a>",
            )
        }
        Target::Directory => {
            let href = format!("/{}", resolved.trim_end_matches('/'));
            (
                format!("<a href=\"{}\"{title_attr}>", escape(&href)),
                "</a>",
            )
        }
        // Source files aren't served (see Q11): render the path, don't link it.
        Target::Other => ("<code class=\"inert\">".to_string(), "</code>"),
    }
}

enum Target {
    Markdown,
    Directory,
    Other,
}

fn target_kind(path: &str) -> Target {
    let name = path.rsplit('/').next().unwrap_or("");
    match name.rsplit_once('.') {
        Some((_, ext)) if ext.eq_ignore_ascii_case("md") => Target::Markdown,
        Some(_) => Target::Other,
        None => Target::Directory,
    }
}

/// Map a repo-relative markdown file to the route of the node that owns it,
/// plus the fragment that page renders under.
fn route_for_markdown(path: &str) -> (String, String) {
    // A lesson's solution lives one directory deeper but belongs to the lesson.
    if let Some(lesson) = path.strip_suffix("/solution/SOLUTION.md") {
        return (format!("/{lesson}"), "solution".to_string());
    }
    let (dir, file) = match path.rsplit_once('/') {
        Some((dir, file)) => (dir, file),
        None => ("", path),
    };
    let anchor = if file == "README.md" {
        String::new()
    } else {
        anchor_for(file)
    };
    (format!("/{dir}"), anchor)
}

fn is_external(dest: &str) -> bool {
    let lower = dest.to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("//")
}

fn split_fragment(dest: &str) -> (&str, &str) {
    match dest.split_once('#') {
        Some((path, fragment)) => (path, fragment),
        None => (dest, ""),
    }
}

/// Resolve `rel` against `base_dir` lexically, collapsing `.` and `..`.
/// Both are repo-relative; the result is too, with no leading slash.
fn resolve(base_dir: &str, rel: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if !rel.starts_with('/') {
        parts.extend(base_dir.split('/').filter(|s| !s.is_empty()));
    }
    for part in rel.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(md: &str, base: &str) -> String {
        to_html(md, base)
    }

    #[test]
    fn rewrites_sibling_and_parent_markdown_links() {
        // The shape phase READMEs use to link their lessons.
        let html = render("[Cargo basics](03-cargo-basics/README.md)", "phase0-setup");
        assert!(
            html.contains("href=\"/phase0-setup/03-cargo-basics\""),
            "a lesson README links to the lesson's own route, no anchor: {html}"
        );

        // The shape lesson READMEs use to link back up.
        let html = render("[Progress](../../PROGRESS.md)", "phase1/02-owning");
        assert!(
            html.contains("href=\"/#file-progress-md\""),
            "a non-README page is an anchor on its node's page: {html}"
        );
    }

    #[test]
    fn solution_links_anchor_to_the_owning_lesson() {
        let html = render(
            "[Solution](solution/SOLUTION.md)",
            "phase1/02-owning/01-moves",
        );
        assert!(
            html.contains("href=\"/phase1/02-owning/01-moves#solution\""),
            "SOLUTION.md belongs to the lesson, not a node of its own: {html}"
        );
    }

    #[test]
    fn external_links_open_in_a_new_tab() {
        let html = render("[Book](https://doc.rust-lang.org/book/)", "");
        assert!(html.contains("target=\"_blank\""), "{html}");
        assert!(html.contains("rel=\"noopener noreferrer\""), "{html}");
        assert!(
            html.contains("href=\"https://doc.rust-lang.org/book/\""),
            "{html}"
        );
    }

    #[test]
    fn source_file_links_render_inert() {
        let html = render(
            "[postgres.rs](../../../capstone-taskforge/taskforge-storage/src/postgres.rs)",
            "phase5/02-db/03-sharding",
        );
        assert!(
            !html.contains("<a "),
            "source files aren't served, so they must not be links: {html}"
        );
        assert!(
            html.contains("<code class=\"inert\">postgres.rs</code>"),
            "{html}"
        );
    }

    #[test]
    fn directory_links_resolve_to_the_node() {
        let html = render("[Quote CLI](side-quests/sq-01-anime-quote-cli)", "");
        assert!(
            html.contains("href=\"/side-quests/sq-01-anime-quote-cli\""),
            "{html}"
        );
    }

    #[test]
    fn fragment_only_links_are_left_alone() {
        let html = render("[jump](#somewhere)", "phase0-setup");
        assert!(html.contains("href=\"#somewhere\""), "{html}");
    }

    #[test]
    fn tables_and_task_lists_are_enabled() {
        let html = render("| a | b |\n|---|---|\n| 1 | 2 |\n", "");
        assert!(html.contains("<table>"), "GFM tables must render: {html}");

        let html = render("- [ ] todo\n- [x] done\n", "");
        assert!(
            html.contains("type=\"checkbox\""),
            "task lists must render: {html}"
        );
    }

    #[test]
    fn raw_html_passes_through() {
        let html = render(
            "<details>\n<summary>More</summary>\n\ntext\n\n</details>\n",
            "",
        );
        assert!(
            html.contains("<details>"),
            "PROGRESS.md's collapsible sections must survive: {html}"
        );
    }
}
