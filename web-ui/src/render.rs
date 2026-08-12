//! Markdown → HTML, with locale-aware internal links and typed concept visuals.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};

use crate::locale::Locale;
use crate::tree::anchor_for;
use crate::visual;

pub fn to_html(markdown: &str, base_dir: &str, locale: Locale) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_FOOTNOTES);

    let mut closings: Vec<&'static str> = Vec::new();
    let mut visual_source: Option<String> = None;
    let mut visual_index = 0usize;
    let events: Vec<Event> = Parser::new_ext(markdown, options)
        .filter_map(|event| match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref language)))
                if language.as_ref().trim() == "senpai-visual" =>
            {
                visual_source = Some(String::new());
                None
            }
            Event::Text(ref text) if visual_source.is_some() => {
                visual_source.as_mut().unwrap().push_str(text);
                None
            }
            Event::End(TagEnd::CodeBlock) if visual_source.is_some() => {
                let source = visual_source.take().unwrap();
                let html = match visual::parse(source.trim()) {
                    Ok(spec) => visual::render(&spec, visual_index),
                    Err(err) => format!(
                        "<aside class=\"visual-error\" role=\"alert\">Invalid senpai-visual: {}</aside>",
                        escape(&err)
                    ),
                };
                visual_index += 1;
                Some(Event::Html(html.into()))
            }
            Event::Start(Tag::Link {
                ref dest_url,
                ref title,
                ..
            }) => {
                let (open, close) = link_html(dest_url, title, base_dir, locale);
                closings.push(close);
                Some(Event::Html(open.into()))
            }
            Event::End(TagEnd::Link) => {
                Some(Event::Html(closings.pop().unwrap_or("</a>").into()))
            }
            other => Some(other),
        })
        .collect();

    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, events.into_iter());
    html
}

fn link_html(dest: &str, title: &str, base_dir: &str, locale: Locale) -> (String, &'static str) {
    let title_attr = if title.is_empty() {
        String::new()
    } else {
        format!(" title=\"{}\"", escape(title))
    };
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
            let (route, generated_anchor) = route_for_markdown(&resolved, locale);
            let anchor = if fragment.is_empty() {
                generated_anchor
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
            let href = format!("/{}/{}", locale.code(), resolved.trim_end_matches('/'));
            (
                format!("<a href=\"{}\"{title_attr}>", escape(&href)),
                "</a>",
            )
        }
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

fn route_for_markdown(path: &str, locale: Locale) -> (String, String) {
    if let Some(lesson) = path.strip_suffix("/solution/SOLUTION.md") {
        return (
            format!("/{}/{lesson}", locale.code()),
            "solution".to_string(),
        );
    }
    let (dir, file) = path.rsplit_once('/').unwrap_or(("", path));
    let anchor = if file == "README.md" {
        String::new()
    } else {
        anchor_for(file)
    };
    (format!("/{}/{dir}", locale.code()), anchor)
}

fn is_external(dest: &str) -> bool {
    let lower = dest.to_lowercase();
    lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("//")
}

fn split_fragment(dest: &str) -> (&str, &str) {
    dest.split_once('#').unwrap_or((dest, ""))
}

fn resolve(base_dir: &str, rel: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if !rel.starts_with('/') {
        parts.extend(base_dir.split('/').filter(|part| !part.is_empty()));
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

    fn render(markdown: &str, base: &str) -> String {
        to_html(markdown, base, Locale::En)
    }

    #[test]
    fn rewrites_internal_links_with_locale() {
        let html = render("[Cargo](03-cargo/README.md)", "phase0");
        assert!(html.contains("href=\"/en/phase0/03-cargo\""), "{html}");
        let html = to_html("[درس](lesson/README.md)", "phase0", Locale::Fa);
        assert!(html.contains("href=\"/fa/phase0/lesson\""), "{html}");
    }

    #[test]
    fn solution_links_anchor_to_the_lesson() {
        let html = render("[Solution](solution/SOLUTION.md)", "phase1/group/lesson");
        assert!(html.contains("href=\"/en/phase1/group/lesson#solution\""));
    }

    #[test]
    fn external_links_open_in_a_new_tab() {
        let html = render("[Book](https://doc.rust-lang.org/book/)", "");
        assert!(html.contains("target=\"_blank\""));
        assert!(html.contains("rel=\"noopener noreferrer\""));
    }

    #[test]
    fn source_files_render_inert() {
        let html = render("[lib.rs](src/lib.rs)", "phase1/lesson");
        assert!(html.contains("<code class=\"inert\">lib.rs</code>"));
    }

    #[test]
    fn tables_tasks_and_raw_html_survive() {
        let html = render(
            "|a|b|\n|-|-|\n|1|2|\n\n- [ ] todo\n\n<details>more</details>",
            "",
        );
        assert!(html.contains("<table>"));
        assert!(html.contains("type=\"checkbox\""));
        assert!(html.contains("<details>"));
    }

    #[test]
    fn renders_typed_visual_fences() {
        let html = render(
            "```senpai-visual\n{\"kind\":\"ownership\",\"labels\":[\"Matin\",\"handler\"]}\n```",
            "",
        );
        assert!(html.contains("concept-ownership"));
        assert!(html.contains("<title"));
        assert!(!html.contains("<pre>"));
    }
}
