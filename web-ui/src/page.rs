//! Builds the HTML for a node: sidebar tree on the left, the node's own
//! markdown pages stacked on the right.
//!
//! There is no JavaScript. Sidebar collapsing and the solution reveal are both
//! native `<details>`, and marking a lesson complete is a plain form POST
//! followed by a redirect — so the sidebar re-renders with the checkmark
//! already applied, with no client-side state to keep in sync.

use std::path::Path;

use crate::progress::Progress;
use crate::render;
use crate::tree::Node;

pub fn render_node(root: &Path, tree: &Node, node: &Node, progress: &Progress) -> String {
    let title = if node.path.is_empty() {
        tree.title.clone()
    } else {
        format!("{} · {}", node.title, tree.title)
    };
    shell(
        &title,
        &sidebar(tree, node, progress),
        &content(root, tree, node, progress),
    )
}

pub fn render_missing(tree: &Node, path: &str, progress: &Progress) -> String {
    let body = format!(
        "<h1>Not found</h1><p class=\"notice\">Nothing in this repo lives at <code>{}</code>.</p>",
        escape(path)
    );
    shell(
        &format!("Not found · {}", tree.title),
        &sidebar(tree, tree, progress),
        &body,
    )
}

fn shell(title: &str, nav: &str, main: &str) -> String {
    format!(
        "<!doctype html>\n\
<html lang=\"en\">\n\
<head>\n\
<meta charset=\"utf-8\">\n\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
<title>{title}</title>\n\
<link rel=\"stylesheet\" href=\"/style.css\">\n\
</head>\n\
<body>\n\
<div class=\"layout\">{nav}<main>{main}</main></div>\n\
</body>\n\
</html>\n",
        title = escape(title)
    )
}

// ---------------------------------------------------------------- sidebar

fn sidebar(tree: &Node, current: &Node, progress: &Progress) -> String {
    let mut html = String::from("<nav>");
    html.push_str(&format!(
        "<a class=\"home\" href=\"/\">{}</a>",
        escape(&tree.title)
    ));
    html.push_str("<ul>");
    for child in &tree.children {
        html.push_str(&sidebar_entry(child, current, progress));
    }
    html.push_str("</ul></nav>");
    html
}

fn sidebar_entry(node: &Node, current: &Node, progress: &Progress) -> String {
    let complete = is_complete(node, progress);
    let is_current = node.path == current.path;
    let mut classes = Vec::new();
    if complete {
        classes.push("done");
    }
    if is_current {
        classes.push("current");
    }
    let class_attr = if classes.is_empty() {
        String::new()
    } else {
        format!(" class=\"{}\"", classes.join(" "))
    };

    let mark = if complete {
        "<span class=\"done-mark\">\u{2713}</span>"
    } else {
        ""
    };
    let link = format!(
        "{mark}<a href=\"/{path}\">{title}</a>",
        path = escape(&node.path),
        title = escape(&node.title)
    );

    if node.children.is_empty() {
        return format!("<li{class_attr}>{link}</li>");
    }

    let count = count_badge(node, progress);
    let open = if on_path(node, current) { " open" } else { "" };
    let mut html = format!("<li{class_attr}><details{open}><summary>{link}{count}</summary><ul>");
    for child in &node.children {
        html.push_str(&sidebar_entry(child, current, progress));
    }
    html.push_str("</ul></details></li>");
    html
}

/// Is `current` this node or somewhere beneath it?
fn on_path(node: &Node, current: &Node) -> bool {
    current.path == node.path || current.path.starts_with(&format!("{}/", node.path))
}

// ---------------------------------------------------------------- content

fn content(root: &Path, tree: &Node, node: &Node, progress: &Progress) -> String {
    let mut html = String::new();
    html.push_str(&crumbs(tree, node));

    for page in &node.pages {
        let dir = if node.path.is_empty() {
            root.to_path_buf()
        } else {
            root.join(&node.path)
        };
        let file = dir.join(&page.file);
        let markdown = std::fs::read_to_string(&file)
            .unwrap_or_else(|err| format!("Could not read `{}`: {err}", page.file));
        // Links resolve against the directory the file itself lives in — which
        // for `solution/SOLUTION.md` is one level below the lesson.
        let base_dir = match page.file.rsplit_once('/') {
            Some((sub, _)) if node.path.is_empty() => sub.to_string(),
            Some((sub, _)) => format!("{}/{}", node.path, sub),
            None => node.path.clone(),
        };
        let body = render::to_html(&markdown, &base_dir);

        if page.gated {
            // `docs/conventions.md` step 6 puts the solution last on purpose.
            html.push_str(&format!(
                "<details class=\"reveal\" id=\"{anchor}\">\
                 <summary>Show the reference solution \u{2014} the conventions say to \
                 answer the checkpoint first</summary>{body}</details>",
                anchor = escape(&page.anchor)
            ));
        } else {
            html.push_str(&format!(
                "<section class=\"page\" id=\"{anchor}\">{body}</section>",
                anchor = escape(&page.anchor)
            ));
        }
    }

    if !node.children.is_empty() {
        html.push_str(&children_index(node, progress));
    }
    if node.is_lesson() {
        html.push_str(&mark_form(node, progress));
    }
    html
}

fn crumbs(tree: &Node, node: &Node) -> String {
    if node.path.is_empty() {
        return String::new();
    }
    let mut links = vec![format!("<a href=\"/\">{}</a>", escape(&tree.title))];
    let mut prefix = String::new();
    for segment in node.path.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(segment);
        let title = tree
            .find(&prefix)
            .map(|n| n.title.clone())
            .unwrap_or_else(|| segment.to_string());
        links.push(format!(
            "<a href=\"/{}\">{}</a>",
            escape(&prefix),
            escape(&title)
        ));
    }
    // The last crumb is the page you're on.
    links.pop();
    if links.is_empty() {
        return String::new();
    }
    format!("<div class=\"crumbs\">{}</div>", links.join(" / "))
}

fn children_index(node: &Node, progress: &Progress) -> String {
    let mut html = String::from("<h2>Contents</h2><ul class=\"children\">");
    for child in &node.children {
        let complete = is_complete(child, progress);
        let class = if complete { " class=\"done\"" } else { "" };
        let mark = if complete {
            "<span class=\"done-mark\">\u{2713}</span>"
        } else {
            ""
        };
        let count = count_badge(child, progress);
        html.push_str(&format!(
            "<li{class}>{mark}<a href=\"/{path}\">{title}</a>{count}</li>",
            path = escape(&child.path),
            title = escape(&child.title)
        ));
    }
    html.push_str("</ul>");
    html
}

fn mark_form(node: &Node, progress: &Progress) -> String {
    let complete = progress.is_complete(&node.path);
    let (label, next, state) = if complete {
        ("Mark not complete", "false", "\u{2713} Completed")
    } else {
        ("Mark complete", "true", "")
    };
    format!(
        "<form class=\"mark\" method=\"post\" action=\"/mark\">\
         <input type=\"hidden\" name=\"path\" value=\"{path}\">\
         <input type=\"hidden\" name=\"complete\" value=\"{next}\">\
         <button type=\"submit\">{label}</button>\
         <span class=\"state\">{state}</span>\
         </form>",
        path = escape(&node.path)
    )
}

// ---------------------------------------------------------------- progress

/// `3/6` for a node with lessons beneath it. Empty for a lesson itself, and for
/// a node like `docs/` that has children but no lessons — a `0/0` badge is
/// noise, not information.
fn count_badge(node: &Node, progress: &Progress) -> String {
    if node.children.is_empty() {
        return String::new();
    }
    let (done, total) = lesson_counts(node, progress);
    if total == 0 {
        return String::new();
    }
    format!("<span class=\"count\">{done}/{total}</span>")
}

/// Lessons complete / lessons total beneath a node.
fn lesson_counts(node: &Node, progress: &Progress) -> (usize, usize) {
    let lessons = node.lessons();
    let done = lessons
        .iter()
        .filter(|l| progress.is_complete(&l.path))
        .count();
    (done, lessons.len())
}

/// A lesson is complete when it's ticked. A parent is complete only when every
/// lesson beneath it is — so a parent can never contradict its children.
fn is_complete(node: &Node, progress: &Progress) -> bool {
    if node.is_lesson() {
        return progress.is_complete(&node.path);
    }
    let (done, total) = lesson_counts(node, progress);
    total > 0 && done == total
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
    use crate::tree::{Node, Page};

    fn lesson(path: &str, title: &str) -> Node {
        Node {
            path: path.to_string(),
            title: title.to_string(),
            pages: vec![Page {
                file: "README.md".to_string(),
                anchor: "file-readme-md".to_string(),
                gated: false,
            }],
            children: Vec::new(),
        }
    }

    fn parent(path: &str, title: &str, children: Vec<Node>) -> Node {
        Node {
            path: path.to_string(),
            title: title.to_string(),
            pages: vec![Page {
                file: "README.md".to_string(),
                anchor: "file-readme-md".to_string(),
                gated: false,
            }],
            children,
        }
    }

    #[test]
    fn a_parent_is_complete_only_when_every_lesson_below_it_is() {
        let tree = parent(
            "",
            "Repo",
            vec![parent(
                "phase0",
                "Phase 0",
                vec![lesson("phase0/01-a", "A"), lesson("phase0/02-b", "B")],
            )],
        );
        let phase = tree.find("phase0").unwrap();

        let mut progress = Progress::default();
        assert_eq!(lesson_counts(phase, &progress), (0, 2));
        assert!(!is_complete(phase, &progress));

        progress.set("phase0/01-a", true);
        assert_eq!(lesson_counts(phase, &progress), (1, 2));
        assert!(!is_complete(phase, &progress), "half done is not done");

        progress.set("phase0/02-b", true);
        assert!(is_complete(phase, &progress));
    }

    #[test]
    fn sidebar_marks_completed_lessons_and_opens_the_current_branch() {
        let tree = parent(
            "",
            "Repo",
            vec![
                parent("phase0", "Phase 0", vec![lesson("phase0/01-a", "A")]),
                parent("phase1", "Phase 1", vec![lesson("phase1/01-c", "C")]),
            ],
        );
        let mut progress = Progress::default();
        progress.set("phase0/01-a", true);

        let current = tree.find("phase0/01-a").unwrap();
        let html = sidebar(&tree, current, &progress);

        assert!(
            html.contains("<details open>"),
            "current branch is expanded: {html}"
        );
        assert_eq!(
            html.matches("<details open>").count(),
            1,
            "only the current branch is expanded: {html}"
        );
        assert!(html.contains("class=\"done current\""), "{html}");
        assert!(
            html.contains("\u{2713}"),
            "completed lessons get a checkmark: {html}"
        );
        assert!(html.contains("<span class=\"count\">1/1</span>"), "{html}");
        assert!(html.contains("<span class=\"count\">0/1</span>"), "{html}");
    }

    #[test]
    fn only_lessons_get_a_mark_button() {
        let tree = parent("", "Repo", vec![lesson("phase0/01-a", "A")]);
        let docs = Node {
            path: "docs".to_string(),
            title: "Docs".to_string(),
            // No README.md, so it's a readable node but not a lesson.
            pages: vec![Page {
                file: "START-HERE.md".to_string(),
                anchor: "file-start-here-md".to_string(),
                gated: false,
            }],
            children: Vec::new(),
        };
        assert!(tree.find("phase0/01-a").unwrap().is_lesson());
        assert!(!docs.is_lesson());

        let progress = Progress::default();
        let form = mark_form(tree.find("phase0/01-a").unwrap(), &progress);
        assert!(
            form.contains("value=\"true\""),
            "un-ticked lesson offers to complete"
        );
        assert!(form.contains("Mark complete"));
    }
}
