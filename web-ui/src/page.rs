//! Persian-first, zero-JavaScript server-rendered course pages.

use std::path::Path;

use crate::locale::Locale;
use crate::progress::Progress;
use crate::render;
use crate::search::SearchHit;
use crate::tree::{self, Node};
use crate::visual;

pub fn render_node(
    root: &Path,
    tree: &Node,
    node: &Node,
    progress: &Progress,
    locale: Locale,
) -> String {
    let page_title = if node.path.is_empty() {
        if locale.is_fa() {
            "سفر Rust متین".to_string()
        } else {
            tree.title.clone()
        }
    } else {
        format!("{} · {}", node_title(node, locale), tree.title)
    };
    let body = if node.path.is_empty() {
        dashboard(tree, progress, locale)
    } else {
        content(root, tree, node, progress, locale)
    };
    shell(&page_title, tree, node, progress, locale, &body)
}

pub fn render_search(
    tree: &Node,
    progress: &Progress,
    locale: Locale,
    query: &str,
    hits: &[SearchHit<'_>],
) -> String {
    let heading = if locale.is_fa() {
        format!("نتیجه‌های جست‌وجو برای «{}»", escape(query))
    } else {
        format!("Search results for “{}”", escape(query))
    };
    let mut body = format!(
        "<header class=\"page-intro\"><p class=\"eyebrow\">{}</p><h1>{heading}</h1><p class=\"muted\">{} {}</p></header>",
        locale.search(),
        hits.len(),
        if locale.is_fa() { "نتیجه" } else { "results" }
    );
    if hits.is_empty() {
        body.push_str(if locale.is_fa() {
            "<div class=\"empty-state\"><h2>چیزی پیدا نشد</h2><p>املای واژه را بررسی کن یا عبارت کوتاه‌تری مثل «مالکیت»، «خطا» یا <bdi>async</bdi> بنویس.</p><a class=\"button secondary\" href=\"/fa/\">دیدن نقشه‌ی دوره</a></div>"
        } else {
            "<div class=\"empty-state\"><h2>No results</h2><p>Try a shorter term such as ownership, error, or async.</p><a class=\"button secondary\" href=\"/en/\">Browse the course map</a></div>"
        });
    } else {
        body.push_str("<ol class=\"search-results\">");
        for hit in hits {
            body.push_str(&format!(
                "<li><a href=\"/{locale}/{path}\"><span>{title}</span><small dir=\"ltr\">{path}</small></a><p>{snippet}</p></li>",
                locale = locale.code(),
                path = escape(&hit.node.path),
                title = escape(node_title(hit.node, locale)),
                snippet = escape(&hit.snippet)
            ));
        }
        body.push_str("</ol>");
    }
    shell(
        &heading,
        tree,
        tree,
        progress,
        locale,
        &format!("<div class=\"reading\">{body}</div>"),
    )
}

pub fn render_missing(tree: &Node, path: &str, progress: &Progress, locale: Locale) -> String {
    let (title, message) = if locale.is_fa() {
        ("صفحه پیدا نشد", "در این مخزن چنین مسیری وجود ندارد.")
    } else {
        (
            "Not found",
            "Nothing in this repository lives at this path.",
        )
    };
    let body = format!(
        "<div class=\"reading empty-state\"><h1>{title}</h1><p>{message}</p><code dir=\"ltr\">{}</code><p><a class=\"button\" href=\"/{}/\">{}</a></p></div>",
        escape(path),
        locale.code(),
        locale.home()
    );
    shell(title, tree, tree, progress, locale, &body)
}

fn shell(
    title: &str,
    tree: &Node,
    current: &Node,
    progress: &Progress,
    locale: Locale,
    main: &str,
) -> String {
    let other = locale.other();
    let path = if current.path.is_empty() {
        String::new()
    } else {
        current.path.clone()
    };
    let language_label = if locale.is_fa() {
        "English"
    } else {
        "فارسی"
    };
    let direction_label = if locale.is_fa() {
        "تغییر زبان به انگلیسی"
    } else {
        "Switch language to Persian"
    };
    let nav = sidebar(tree, current, progress, locale);
    format!(
        "<!doctype html>\n<html lang=\"{lang}\" dir=\"{dir}\" class=\"theme-dark\">\n<head>\
         <meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\
         <meta name=\"theme-color\" content=\"#050505\"><title>{title}</title>\
         <link rel=\"preload\" href=\"/assets/vazirmatn.woff2\" as=\"font\" type=\"font/woff2\" crossorigin>\
         <link rel=\"stylesheet\" href=\"/assets/style.css\"></head>\n<body>\
         <a class=\"skip-link\" href=\"#main-content\">{skip}</a>\
         <header class=\"topbar\"><a class=\"brand\" href=\"https://github.com/MatinSenPai/SenPai-Rust-Journey\" target=\"_blank\" rel=\"noopener noreferrer\" translate=\"no\"><span>MatinSenPai/SenPai-Rust-Journey</span></a>\
         <form class=\"search\" role=\"search\" method=\"get\" action=\"/{lang}/search\">\
         <label class=\"visually-hidden\" for=\"site-search\">{search}</label>\
         <input id=\"site-search\" name=\"q\" type=\"search\" minlength=\"2\" autocomplete=\"off\" placeholder=\"{placeholder}\">\
         <button type=\"submit\">{search}</button></form>\
         <a class=\"language\" aria-label=\"{direction_label}\" hreflang=\"{other}\" href=\"/{other}/{path}\">{language_label}</a></header>\
         <div class=\"app-shell\">{nav}<main id=\"main-content\" tabindex=\"-1\">{main}</main></div></body></html>",
        lang = locale.code(),
        dir = locale.dir(),
        title = escape(title),
        skip = locale.skip(),
        search = locale.search(),
        placeholder = locale.search_placeholder(),
        direction_label = direction_label,
        other = other.code(),
        path = escape(&path),
        language_label = language_label,
    )
}

fn dashboard(tree: &Node, progress: &Progress, locale: Locale) -> String {
    let lessons = tree.lessons();
    let done = lessons
        .iter()
        .filter(|lesson| progress.is_complete(&lesson.path))
        .count();
    let total = lessons.len();
    let percent = (done * 100).checked_div(total).unwrap_or(0);
    let next = lessons
        .iter()
        .find(|lesson| !progress.is_complete(&lesson.path))
        .copied();
    let (hello, intro, continue_label) = if locale.is_fa() {
        (
            "سلام متین؛ بیا از همین‌جا شروع کنیم.",
            "قرار نیست Rust رو حفظ کنی. قدم‌به‌قدم می‌فهمی هر بخش چرا وجود داره، با دستای خودت تمرینش می‌کنی و تو آخرِ مسیر باهاش یه بک‌اندِ واقعی می‌سازی.",
            "ادامه‌ی یادگیری",
        )
    } else {
        (
            "Welcome back, Matin. Ready to understand Rust?",
            "Build every step yourself, from the first program to a production-minded backend. The goal is durable understanding, not a syntax sprint.",
            "Continue learning",
        )
    };
    let next_link = next
        .map(|lesson| format!("/{}/{}", locale.code(), lesson.path))
        .unwrap_or_else(|| format!("/{}/", locale.code()));
    let next_title = next
        .map(|lesson| node_title(lesson, locale))
        .unwrap_or_else(|| {
            if locale.is_fa() {
                "تمومِ درس‌ها کامل شدن"
            } else {
                "All lessons complete"
            }
        });
    let mut phases = String::new();
    for child in &tree.children {
        let (child_done, child_total) = lesson_counts(child, progress);
        if child_total == 0 {
            continue;
        }
        let phase_count = if locale.is_fa() {
            let raw = format!("{child_done:02}/{child_total:02}");
            raw.chars()
                .map(|digit| match digit {
                    '0' => '۰',
                    '1' => '۱',
                    '2' => '۲',
                    '3' => '۳',
                    '4' => '۴',
                    '5' => '۵',
                    '6' => '۶',
                    '7' => '۷',
                    '8' => '۸',
                    '9' => '۹',
                    other => other,
                })
                .collect::<String>()
        } else {
            format!("{child_done:02}/{child_total:02}")
        };
        phases.push_str(&format!(
            "<li><a href=\"/{locale}/{path}\"><span class=\"phase-index\">{count}</span><strong>{title}</strong><span class=\"phase-line\" aria-hidden=\"true\"></span></a></li>",
            locale = locale.code(),
            path = escape(&child.path),
            count = phase_count,
            title = escape(node_title(child, locale))
        ));
    }
    let steps = if locale.is_fa() {
        [
            ("۰۱", "بخون", "ایده و دلیلِ وجودش رو درک کن."),
            ("۰۲", "بساز", "کُدِ ناقص رو انقدر کامل کن تا تست‌هاش سبز بشن."),
            (
                "۰۳",
                "به خاطر بیار",
                "به سؤالاتِ ایستِ بازرسی بدونِ نگاه‌کردن به جوابا پاسخ بده.",
            ),
        ]
    } else {
        [
            ("01", "Read", "Understand the idea and why it exists."),
            ("02", "Build", "Finish the starter code until tests pass."),
            (
                "03",
                "Recall",
                "Answer the checkpoint without looking back.",
            ),
        ]
    };
    let step_html = steps
        .iter()
        .map(|(number, title, body)| {
            format!("<li><span>{number}</span><h3>{title}</h3><p>{body}</p></li>")
        })
        .collect::<String>();
    format!(
        "<div class=\"dashboard\"><section class=\"hero\"><div><p class=\"eyebrow\">{journey_label}</p>\
         <h1>{hello}</h1><p class=\"lead\">{intro}</p><a class=\"button primary\" href=\"{next_link}\">{continue_label}<span aria-hidden=\"true\"> ←</span></a>\
         <p class=\"next-up\"><span>{percent}%</span> · {next_title}</p></div>{visual}</section>\
         <section class=\"learning-loop\" aria-labelledby=\"loop-title\"><p class=\"eyebrow\">{loop_label}</p><h2 id=\"loop-title\">{loop_title}</h2><ol>{step_html}</ol></section>\
         <section class=\"roadmap\" aria-labelledby=\"roadmap-title\"><p class=\"eyebrow\">{map_label}</p><h2 id=\"roadmap-title\">{map_title}</h2><ol>{phases}</ol></section></div>",
        journey_label = if locale.is_fa() { "مسیر یادگیری" } else { "RUST / BACKEND / SYSTEM DESIGN" },
        hello = hello,
        intro = intro,
        next_link = escape(&next_link),
        continue_label = continue_label,
        percent = localized_number(percent, locale),
        next_title = escape(next_title),
        visual = visual::fallback(
            "roadmap",
            if locale.is_fa() {
                &["شروع", "مالکیت", "Backend", "TaskForge"]
            } else {
                &["Start", "Ownership", "Backend", "TaskForge"]
            },
            9000,
        ),
        loop_label = if locale.is_fa() { "شیوه‌ی پیش‌روی" } else { "LEARNING LOOP" },
        loop_title = if locale.is_fa() { "هر درس را چطور پیش ببری؟" } else { "The lesson loop" },
        map_label = if locale.is_fa() { "نقشه‌ی راه" } else { "ROADMAP" },
        map_title = if locale.is_fa() { "راهی که پیش رو داری" } else { "Your complete path" },
    )
}

fn content(root: &Path, tree: &Node, node: &Node, progress: &Progress, locale: Locale) -> String {
    let mut html = String::from("<div class=\"lesson-layout\"><article class=\"reading\">");
    html.push_str(&crumbs(tree, node, locale));
    for page in &node.pages {
        let dir = if node.path.is_empty() {
            root.to_path_buf()
        } else {
            root.join(&node.path)
        };
        let canonical = dir.join(&page.file);
        let localized = tree::localized_path(&canonical, locale);
        if locale.is_fa() && !tree::has_translation(&canonical) {
            html.push_str(&format!(
                "<aside class=\"translation-notice\" role=\"status\">{}</aside>",
                locale.translation_missing()
            ));
        }
        let markdown = std::fs::read_to_string(&localized)
            .unwrap_or_else(|err| format!("Could not read `{}`: {err}", localized.display()));
        let base_dir = match page.file.rsplit_once('/') {
            Some((sub, _)) if node.path.is_empty() => sub.to_string(),
            Some((sub, _)) => format!("{}/{}", node.path, sub),
            None => node.path.clone(),
        };
        let body = render::to_html(&markdown, &base_dir, locale);
        if page.gated {
            html.push_str(&format!(
                "<details class=\"reveal\" id=\"{}\"><summary>{}</summary>{body}</details>",
                escape(&page.anchor),
                locale.solution(),
            ));
        } else {
            html.push_str(&format!(
                "<section class=\"page\" id=\"{}\">{body}</section>",
                escape(&page.anchor),
            ));
        }
    }
    if !node.children.is_empty() {
        html.push_str(&children_index(node, progress, locale));
    }
    if node.is_lesson() {
        html.push_str(&lesson_navigation(tree, node, locale));
        html.push_str(&mark_form(tree, node, progress, locale));
    }
    html.push_str("</article></div>");
    html
}

fn sidebar(tree: &Node, current: &Node, progress: &Progress, locale: Locale) -> String {
    let mut list = String::new();
    for child in &tree.children {
        list.push_str(&sidebar_entry(child, current, progress, locale));
    }
    format!(
        "<aside class=\"course-nav\"><details class=\"mobile-nav\"><summary>{}</summary><nav aria-label=\"{}\"><ul>{list}</ul></nav></details>\
         <nav class=\"desktop-nav\" aria-label=\"{}\"><p class=\"nav-label\">{}</p><ul>{list}</ul></nav></aside>",
        locale.course_map(),
        locale.course_map(),
        locale.course_map(),
        locale.course_map(),
    )
}

fn sidebar_entry(node: &Node, current: &Node, progress: &Progress, locale: Locale) -> String {
    let complete = is_complete(node, progress);
    let current_attr = if node.path == current.path {
        " aria-current=\"page\""
    } else {
        ""
    };
    let class = match (complete, node.path == current.path) {
        (true, true) => " class=\"done current\"",
        (true, false) => " class=\"done\"",
        (false, true) => " class=\"current\"",
        _ => "",
    };
    let mark = if complete {
        "<span aria-hidden=\"true\">✓</span>"
    } else {
        ""
    };
    let link = format!(
        "{mark}<a{current_attr} href=\"/{locale}/{path}\">{title}</a>",
        locale = locale.code(),
        path = escape(&node.path),
        title = escape(node_title(node, locale))
    );
    if node.children.is_empty() {
        return format!("<li{class}>{link}</li>");
    }
    let open = if on_path(node, current) { " open" } else { "" };
    let count = count_badge(node, progress, locale);
    let children = node
        .children
        .iter()
        .map(|child| sidebar_entry(child, current, progress, locale))
        .collect::<String>();
    format!("<li{class}><details{open}><summary>{link}{count}</summary><ul>{children}</ul></details></li>")
}

fn crumbs(tree: &Node, node: &Node, locale: Locale) -> String {
    let mut links = vec![format!(
        "<a href=\"/{}/\">{}</a>",
        locale.code(),
        locale.home()
    )];
    let mut prefix = String::new();
    for segment in node.path.split('/') {
        if !prefix.is_empty() {
            prefix.push('/');
        }
        prefix.push_str(segment);
        let title = tree
            .find(&prefix)
            .map(|item| node_title(item, locale))
            .unwrap_or(segment);
        links.push(format!(
            "<a href=\"/{}/{}\">{}</a>",
            locale.code(),
            escape(&prefix),
            escape(title)
        ));
    }
    links.pop();
    format!(
        "<nav class=\"crumbs\" aria-label=\"Breadcrumb\">{}</nav>",
        links.join(" <span aria-hidden=\"true\">/</span> ")
    )
}

fn children_index(node: &Node, progress: &Progress, locale: Locale) -> String {
    let mut items = String::new();
    for child in &node.children {
        let class = if is_complete(child, progress) {
            " class=\"done\""
        } else {
            ""
        };
        items.push_str(&format!(
            "<li{class}><a href=\"/{locale}/{path}\">{title}</a>{count}</li>",
            locale = locale.code(),
            path = escape(&child.path),
            title = escape(node_title(child, locale)),
            count = count_badge(child, progress, locale)
        ));
    }
    format!(
        "<section class=\"children-index\"><h2>{}</h2><ul>{items}</ul></section>",
        locale.contents()
    )
}

fn lesson_navigation(tree: &Node, node: &Node, locale: Locale) -> String {
    let lessons = tree.lessons();
    let index = lessons.iter().position(|lesson| lesson.path == node.path);
    let previous = index
        .and_then(|value| value.checked_sub(1))
        .map(|value| lessons[value]);
    let next = index.and_then(|value| lessons.get(value + 1)).copied();
    let link = |target: Option<&Node>, label: &str, class: &str| {
        target.map_or_else(String::new, |lesson| {
            format!(
                "<a class=\"{class}\" href=\"/{}/{path}\"><small>{label}</small><span>{title}</span></a>",
                locale.code(),
                path = escape(&lesson.path),
                title = escape(node_title(lesson, locale))
            )
        })
    };
    format!(
        "<nav class=\"lesson-nav\" aria-label=\"Lesson pagination\">{}{}</nav>",
        link(previous, locale.previous(), "previous"),
        link(next, locale.next(), "next")
    )
}

fn mark_form(tree: &Node, node: &Node, progress: &Progress, locale: Locale) -> String {
    let complete = progress.is_complete(&node.path);
    let next_exists = tree
        .lessons()
        .iter()
        .position(|lesson| lesson.path == node.path)
        .is_some_and(|index| index + 1 < tree.lessons().len());
    let label = if complete {
        locale.mark_incomplete()
    } else {
        locale.mark_complete()
    };
    let state = if complete {
        format!(
            "<span class=\"state\" role=\"status\">✓ {}</span>",
            locale.completed()
        )
    } else {
        String::new()
    };
    let next_button = if !complete && next_exists {
        format!(
            "<button class=\"primary\" name=\"advance\" value=\"true\" type=\"submit\">{}</button>",
            locale.complete_next()
        )
    } else {
        String::new()
    };
    format!(
        "<form class=\"mark\" method=\"post\" action=\"/{}/mark\"><input type=\"hidden\" name=\"path\" value=\"{}\"><input type=\"hidden\" name=\"complete\" value=\"{}\"><div><button class=\"secondary\" type=\"submit\">{label}</button>{next_button}</div>{state}</form>",
        locale.code(),
        escape(&node.path),
        if complete { "false" } else { "true" },
    )
}

fn on_path(node: &Node, current: &Node) -> bool {
    current.path == node.path || current.path.starts_with(&format!("{}/", node.path))
}

fn count_badge(node: &Node, progress: &Progress, locale: Locale) -> String {
    let (done, total) = lesson_counts(node, progress);
    if total == 0 {
        String::new()
    } else {
        format!(
            "<span class=\"count\">{}/{}</span>",
            localized_number(done, locale),
            localized_number(total, locale)
        )
    }
}

fn node_title(node: &Node, locale: Locale) -> &str {
    if !locale.is_fa() {
        return &node.title;
    }
    match node.path.as_str() {
        "phase3-backend-foundations" => "فاز سه — مبانی بک‌اند",
        "phase4-backend-advanced" => "فاز چهار — بک‌اند پیشرفته و طراحی سیستم",
        "phase5-system-design-mastery" => "فاز پنج — تسلط بر طراحی سیستم",
        "side-quests" => "مأموریت‌های جانبی",
        "capstone-taskforge" => "پروژه‌ی نهایی: TaskForge",
        "docs" => "راهنماها و مستندات",
        _ => &node.title,
    }
}

fn localized_number(number: usize, locale: Locale) -> String {
    let raw = number.to_string();
    if !locale.is_fa() {
        return raw;
    }
    raw.chars()
        .map(|digit| match digit {
            '0' => '۰',
            '1' => '۱',
            '2' => '۲',
            '3' => '۳',
            '4' => '۴',
            '5' => '۵',
            '6' => '۶',
            '7' => '۷',
            '8' => '۸',
            '9' => '۹',
            other => other,
        })
        .collect()
}

fn lesson_counts(node: &Node, progress: &Progress) -> (usize, usize) {
    let lessons = node.lessons();
    let done = lessons
        .iter()
        .filter(|lesson| progress.is_complete(&lesson.path))
        .count();
    (done, lessons.len())
}

fn is_complete(node: &Node, progress: &Progress) -> bool {
    if node.is_lesson() {
        progress.is_complete(&node.path)
    } else {
        let (done, total) = lesson_counts(node, progress);
        total > 0 && done == total
    }
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

    fn tree() -> Node {
        Node {
            path: String::new(),
            title: "Repo".to_string(),
            pages: Vec::new(),
            children: vec![lesson("phase0/01-a", "A"), lesson("phase0/02-b", "B")],
        }
    }

    #[test]
    fn persian_shell_is_rtl_and_keeps_the_language_switch_path() {
        let tree = tree();
        let current = tree.find("phase0/01-a").unwrap();
        let html = shell(
            "درس",
            &tree,
            current,
            &Progress::default(),
            Locale::Fa,
            "body",
        );
        assert!(html.contains("lang=\"fa\" dir=\"rtl\""));
        assert!(html.contains("href=\"/en/phase0/01-a\""));
        assert!(html.contains("class=\"skip-link\""));
    }

    #[test]
    fn completion_form_offers_server_side_advance() {
        let tree = tree();
        let html = mark_form(
            &tree,
            tree.find("phase0/01-a").unwrap(),
            &Progress::default(),
            Locale::Fa,
        );
        assert!(html.contains("action=\"/fa/mark\""));
        assert!(html.contains("name=\"advance\""));
        assert!(html.contains("تمام شد؛ درس بعدی"));
    }

    #[test]
    fn parents_complete_only_after_every_lesson() {
        let tree = tree();
        let mut progress = Progress::default();
        progress.set("phase0/01-a", true);
        assert!(!is_complete(&tree, &progress));
        progress.set("phase0/02-b", true);
        assert!(is_complete(&tree, &progress));
    }
}
