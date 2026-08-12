//! Typed, accessible, CSS-animated concept diagrams embedded in Markdown.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisualSpec {
    pub kind: String,
    #[serde(default)]
    pub labels: Vec<String>,
}

const KINDS: &[&str] = &[
    "ownership",
    "borrowing",
    "lifetime",
    "result",
    "async",
    "queue",
    "database",
    "network",
    "concurrency",
    "roadmap",
    "concept",
];

pub fn parse(source: &str) -> Result<VisualSpec, String> {
    let spec: VisualSpec = serde_json::from_str(source).map_err(|err| err.to_string())?;
    if !KINDS.contains(&spec.kind.as_str()) {
        return Err(format!("unknown visual kind `{}`", spec.kind));
    }
    if spec.labels.len() > 6 {
        return Err("a visual supports at most 6 labels".to_string());
    }
    Ok(spec)
}

pub fn render(spec: &VisualSpec, id: usize) -> String {
    let labels = if spec.labels.is_empty() {
        vec![spec.kind.as_str(), "Rust", "backend"]
    } else {
        spec.labels.iter().map(String::as_str).collect()
    };
    let is_persian = labels.iter().any(|label| has_persian(label));
    let title = if is_persian {
        if spec.kind == "roadmap" {
            "نقشه‌ی راه یادگیری Rust".to_string()
        } else {
            "نمودار مفهومی درس".to_string()
        }
    } else {
        format!("{} concept flow", spec.kind)
    };
    let desc = if is_persian {
        if spec.kind == "roadmap" {
            format!("چهار گام مسیر: {}", labels.join(" ← "))
        } else {
            format!("مسیر مفهوم: {}", labels.join(" ← "))
        }
    } else {
        labels.join(" to ")
    };
    let mut nodes = String::new();
    let count = labels.len().max(1);
    for (index, label) in labels.iter().enumerate() {
        let x = if count == 1 {
            380
        } else {
            let offset = index * (620 / (count - 1));
            if is_persian {
                690 - offset
            } else {
                70 + offset
            }
        };
        let delay = index * 70;
        let number = if is_persian {
            ["۰۱", "۰۲", "۰۳", "۰۴", "۰۵", "۰۶"][index]
        } else {
            ["01", "02", "03", "04", "05", "06"][index]
        };
        nodes.push_str(&format!(
            "<g class=\"concept-node\" style=\"--delay:{delay}ms\" transform=\"translate({x} 78)\">\
             <circle r=\"10\"/><text class=\"stage-number\" y=\"-28\" text-anchor=\"middle\">{number}</text>\
             <text class=\"stage-label\" y=\"42\" text-anchor=\"middle\">{}</text></g>",
            escape(label)
        ));
    }
    let path = if spec.kind == "roadmap" {
        "M52 78 H710"
    } else {
        "M52 78 C180 18 260 138 380 78 S590 18 710 78"
    };
    format!(
        "<figure class=\"concept-visual concept-{kind}\" aria-labelledby=\"visual-title-{id} visual-desc-{id}\">\
         <svg role=\"img\" viewBox=\"0 0 760 150\" preserveAspectRatio=\"xMidYMid meet\">\
         <title id=\"visual-title-{id}\">{title}</title><desc id=\"visual-desc-{id}\">{desc}</desc>\
         <path class=\"ownership-thread\" pathLength=\"1\" d=\"{path}\"/>\
         {nodes}</svg><figcaption>{desc}</figcaption></figure>",
        kind = escape(&spec.kind),
        title = escape(&title),
        desc = escape(&desc),
    )
}

fn has_persian(text: &str) -> bool {
    text.chars().any(|character| {
        ('\u{0600}'..='\u{06ff}').contains(&character)
            || ('\u{0750}'..='\u{077f}').contains(&character)
    })
}

pub fn fallback(kind: &str, labels: &[&str], id: usize) -> String {
    render(
        &VisualSpec {
            kind: kind.to_string(),
            labels: labels.iter().map(|label| (*label).to_string()).collect(),
        },
        id,
    )
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

    #[test]
    fn validates_and_renders_accessible_visuals() {
        let spec = parse(r#"{"kind":"ownership","labels":["Matin","handler"]}"#).unwrap();
        let html = render(&spec, 3);
        assert!(html.contains("<title id=\"visual-title-3\""));
        assert!(html.contains("<desc id=\"visual-desc-3\""));
        assert!(html.contains("ownership-thread"));
    }

    #[test]
    fn rejects_unknown_kinds() {
        assert!(parse(r#"{"kind":"sparkles"}"#).is_err());
    }
}
