//! The progress page — `/{locale}/progress`.
//!
//! The sidebar already answers "how many lessons have I ticked". This page
//! answers the question the tick cannot: *do I actually know this?* It reads
//! the same `.course-progress.json` records and the concept map, and shows
//! coverage per phase, which concepts are met versus mastered, an honest
//! estimate of time spent, and what you finished recently.
//!
//! Everything here is derived. Nothing on this page is a separate thing to
//! keep up to date, which is why it cannot drift from the lessons.

use std::path::Path;

use crate::concepts::{self, Mastery};
use crate::locale::Locale;
use crate::progress::Progress;
use crate::tree::Node;

pub fn render(root: &Path, tree: &Node, progress: &Progress, locale: Locale) -> String {
    let lessons = tree.lessons();
    let total = lessons.len();
    let done = lessons
        .iter()
        .filter(|lesson| progress.is_complete(&lesson.path))
        .count();
    let percent = (done * 100).checked_div(total).unwrap_or(0);

    format!(
        "<div class=\"progress-page\"><header class=\"page-intro\">\
         <p class=\"eyebrow\">{eyebrow}</p><h1>{title}</h1>\
         <p class=\"muted\">{intro}</p></header>\
         {stats}{phases}{mastery}{activity}</div>",
        eyebrow = locale.progress(),
        title = escape(locale.progress_title()),
        intro = escape(locale.progress_intro()),
        stats = stats(progress, locale, done, total, percent),
        phases = phases(tree, progress, locale),
        mastery = mastery_grid(root, tree, progress, locale),
        activity = activity(tree, progress, locale),
    )
}

fn stats(progress: &Progress, locale: Locale, done: usize, total: usize, percent: usize) -> String {
    // Reported to one decimal and labelled an estimate: it is derived from two
    // page loads, not from a timer, and `Record::seconds` caps each lesson.
    let hours = progress.total_seconds() as f64 / 3600.0;
    let hours = if hours >= 0.05 {
        format!("{hours:.1}")
    } else {
        "0".to_string()
    };

    let stat = |label: &str, value: String, unit: &str| {
        format!("<div class=\"stat\"><dt>{label}</dt><dd>{value}<small>{unit}</small></dd></div>")
    };

    format!(
        "<dl class=\"stat-row\">{}{}{}{}</dl>",
        stat(
            locale.stat_complete(),
            localized_number(percent, locale),
            "%"
        ),
        stat(
            locale.stat_lessons(),
            localized_number(done, locale),
            &format!(" / {}", localized_number(total, locale))
        ),
        stat(
            locale.stat_streak(),
            localized_number(progress.streak() as usize, locale),
            &format!(" {}", locale.days())
        ),
        stat(
            locale.stat_time(),
            localized_digits(&hours, locale),
            &format!(" {}", locale.hours())
        ),
    )
}

fn phases(tree: &Node, progress: &Progress, locale: Locale) -> String {
    let mut rows = String::new();
    for child in &tree.children {
        let lessons = child.lessons();
        if lessons.is_empty() {
            continue;
        }
        let done = lessons
            .iter()
            .filter(|lesson| progress.is_complete(&lesson.path))
            .count();
        let percent = done * 100 / lessons.len();
        rows.push_str(&format!(
            "<li><div class=\"row\"><a href=\"/{locale_code}/{path}\">{title}</a>\
             <span>{done} / {total} · {percent}%</span></div>\
             <div class=\"bar\" role=\"img\" aria-label=\"{percent}%\">\
             <i style=\"--fill:{raw_percent}%\"></i></div></li>",
            locale_code = locale.code(),
            path = escape(&child.path),
            title = escape(&child.title),
            done = localized_number(done, locale),
            total = localized_number(lessons.len(), locale),
            percent = localized_number(percent, locale),
            raw_percent = percent,
        ));
    }
    format!(
        "<h2>{}</h2><ol class=\"phase-progress\">{rows}</ol>",
        escape(locale.by_phase())
    )
}

fn mastery_grid(root: &Path, tree: &Node, progress: &Progress, locale: Locale) -> String {
    let written: std::collections::HashSet<&str> = tree
        .lessons()
        .iter()
        .map(|lesson| lesson.path.as_str())
        .collect();

    let all = concepts::load(root);
    let total = all.len();
    let concepts: Vec<_> = all
        .into_iter()
        .filter(|concept| written.contains(concept.introduced_in()))
        .collect();
    if concepts.is_empty() {
        return String::new();
    }
    let unwritten = total - concepts.len();
    // Mastered first, then met, then the rest — so the part worth looking at
    // is at the top rather than buried in a wall of grey.
    let mut ranked: Vec<_> = concepts
        .iter()
        .map(|concept| (concept.mastery(progress), concept))
        .collect();
    ranked.sort_by_key(|(state, concept)| {
        (
            match state {
                Mastery::Mastered => 0,
                Mastery::Met => 1,
                Mastery::Pending => 2,
            },
            concept.id.clone(),
        )
    });

    let chips: String = ranked
        .iter()
        .map(|(state, concept)| {
            format!(
                "<span class=\"concept-chip\" data-state=\"{state}\">{name}<bdi>{id}</bdi></span>",
                state = state.slug(),
                name = escape(concept.name(locale)),
                id = escape(&concept.id),
            )
        })
        .collect();

    let pending_note = if unwritten == 0 {
        String::new()
    } else {
        format!(
            " {}",
            locale
                .mastery_unwritten()
                .replace("{n}", &localized_number(unwritten, locale))
        )
    };
    format!(
        "<h2>{heading}</h2><p class=\"muted\">{note}{pending_note}</p>\
         <div class=\"mastery\">{chips}</div>",
        heading = escape(locale.mastery()),
        note = escape(locale.mastery_note()),
        pending_note = escape(&pending_note),
    )
}

fn activity(tree: &Node, progress: &Progress, locale: Locale) -> String {
    let recent = progress.recent(10);
    if recent.is_empty() {
        return format!(
            "<h2>{}</h2><p class=\"muted\">{}</p>",
            escape(locale.recent_activity()),
            escape(locale.nothing_yet())
        );
    }
    let items: String = recent
        .iter()
        .map(|(path, at)| {
            let title = tree
                .find(path)
                .map(|node| node.title.clone())
                .unwrap_or_else(|| (*path).to_string());
            format!(
                "<li><a href=\"/{locale_code}/{path}\">{title}</a>\
                 <time datetime=\"{at}\">{ago}</time></li>",
                locale_code = locale.code(),
                path = escape(path),
                title = escape(&title),
                ago = escape(&relative_day(*at, locale)),
            )
        })
        .collect();
    format!(
        "<h2>{}</h2><ul class=\"activity\">{items}</ul>",
        escape(locale.recent_activity())
    )
}

/// "today" / "yesterday" / "N days ago". Deliberately coarse: an exact
/// timestamp would need a timezone database for no benefit on a page whose
/// job is to show momentum.
fn relative_day(at: i64, locale: Locale) -> String {
    let day = 86_400;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(0);
    let days = (now.div_euclid(day) - at.div_euclid(day)).max(0);
    match (days, locale.is_fa()) {
        (0, true) => "امروز".to_string(),
        (0, false) => "today".to_string(),
        (1, true) => "دیروز".to_string(),
        (1, false) => "yesterday".to_string(),
        (n, true) => format!("{} روز پیش", localized_number(n as usize, locale)),
        (n, false) => format!("{n} days ago"),
    }
}

fn localized_number(number: usize, locale: Locale) -> String {
    localized_digits(&number.to_string(), locale)
}

/// Persian prose uses Persian digits; code and identifiers never do.
pub fn localized_digits(raw: &str, locale: Locale) -> String {
    if !locale.is_fa() {
        return raw.to_string();
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
            '.' => '٫',
            other => other,
        })
        .collect()
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
            children: vec![Node {
                path: "phase0".to_string(),
                title: "Phase 0".to_string(),
                pages: Vec::new(),
                children: vec![lesson("phase0/01-a", "A"), lesson("phase0/02-b", "B")],
            }],
        }
    }

    #[test]
    fn an_empty_course_reports_zero_rather_than_dividing_by_it() {
        let empty = Node {
            path: String::new(),
            title: "Repo".to_string(),
            pages: Vec::new(),
            children: Vec::new(),
        };
        let html = render(Path::new("."), &empty, &Progress::default(), Locale::En);
        assert!(html.contains("0<small>%</small>"));
    }

    #[test]
    fn phase_bars_report_the_share_finished() {
        let tree = tree();
        let mut progress = Progress::default();
        progress.set("phase0/01-a", true);
        let html = render(Path::new("."), &tree, &progress, Locale::En);
        assert!(html.contains("--fill:50%"), "{html}");
        assert!(html.contains("1 / 2 · 50%"));
    }

    #[test]
    fn persian_numbers_use_persian_digits_and_separator() {
        assert_eq!(localized_digits("12.5", Locale::Fa), "۱۲٫۵");
        assert_eq!(localized_digits("12.5", Locale::En), "12.5");
    }

    #[test]
    fn recent_activity_falls_back_to_a_prompt_when_empty() {
        let html = render(Path::new("."), &tree(), &Progress::default(), Locale::En);
        assert!(html.contains("Nothing recorded yet"));
    }

    #[test]
    fn relative_days_read_as_prose() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        assert_eq!(relative_day(now, Locale::En), "today");
        assert_eq!(relative_day(now - 86_400, Locale::En), "yesterday");
        assert_eq!(relative_day(now - 3 * 86_400, Locale::En), "3 days ago");
        assert_eq!(relative_day(now, Locale::Fa), "امروز");
    }
}
