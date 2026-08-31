//! A tiny hand-rolled tokenizer for syntax-colouring fenced ```rust blocks.
//!
//! Not a real lexer: good enough to colour what this curriculum's snippets
//! actually contain (keywords, strings, chars, lifetimes, numbers, types,
//! macros, attributes, comments) and nothing more.
//!
//! ponytail: no nested `/* */` comments, and `r#ident` raw identifiers are
//! read as raw-string hash-fences instead. Both are exotic enough in a
//! teaching curriculum that a real lexer (e.g. `syn`'s) is not worth the
//! dependency unless a snippet actually needs one.

const KEYWORDS: &[&str] = &[
    "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum", "extern",
    "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
    "return", "self", "Self", "static", "struct", "super", "trait", "type", "unsafe", "use",
    "where", "while", "true", "false", "union",
];

pub fn rust(code: &str) -> String {
    let mut out = String::with_capacity(code.len() + code.len() / 4);
    let mut chars = code.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'/') => {
                let mut buf = String::from("//");
                chars.next();
                while let Some(&n) = chars.peek() {
                    if n == '\n' {
                        break;
                    }
                    buf.push(n);
                    chars.next();
                }
                span(&mut out, "tok-com", &buf);
            }
            '/' if chars.peek() == Some(&'*') => {
                let mut buf = String::from("/*");
                chars.next();
                let mut prev = ' ';
                for n in chars.by_ref() {
                    buf.push(n);
                    if prev == '*' && n == '/' {
                        break;
                    }
                    prev = n;
                }
                span(&mut out, "tok-com", &buf);
            }
            '"' => {
                let mut buf = String::from("\"");
                let mut escaped = false;
                for n in chars.by_ref() {
                    buf.push(n);
                    if escaped {
                        escaped = false;
                        continue;
                    }
                    match n {
                        '\\' => escaped = true,
                        '"' => break,
                        _ => {}
                    }
                }
                span(&mut out, "tok-str", &buf);
            }
            'r' if chars.peek() == Some(&'"') || chars.peek() == Some(&'#') => {
                let mut buf = String::from("r");
                let mut hashes = 0usize;
                while chars.peek() == Some(&'#') {
                    buf.push('#');
                    hashes += 1;
                    chars.next();
                }
                if chars.peek() == Some(&'"') {
                    buf.push('"');
                    chars.next();
                    loop {
                        match chars.next() {
                            Some('"') => {
                                buf.push('"');
                                let mut closing = 0usize;
                                while closing < hashes && chars.peek() == Some(&'#') {
                                    buf.push('#');
                                    chars.next();
                                    closing += 1;
                                }
                                if closing == hashes {
                                    break;
                                }
                            }
                            Some(n) => buf.push(n),
                            None => break,
                        }
                    }
                }
                span(&mut out, "tok-str", &buf);
            }
            '\'' => {
                let mut buf = String::from("'");
                match chars.next() {
                    Some('\\') => {
                        buf.push('\\');
                        if let Some(e) = chars.next() {
                            buf.push(e);
                            if e == 'u' {
                                while let Some(&x) = chars.peek() {
                                    buf.push(x);
                                    chars.next();
                                    if x == '}' {
                                        break;
                                    }
                                }
                            }
                        }
                        if chars.peek() == Some(&'\'') {
                            buf.push('\'');
                            chars.next();
                        }
                        span(&mut out, "tok-str", &buf);
                    }
                    Some(n) => {
                        buf.push(n);
                        if chars.peek() == Some(&'\'') {
                            buf.push('\'');
                            chars.next();
                            span(&mut out, "tok-str", &buf);
                        } else {
                            while let Some(&x) = chars.peek() {
                                if x.is_alphanumeric() || x == '_' {
                                    buf.push(x);
                                    chars.next();
                                } else {
                                    break;
                                }
                            }
                            span(&mut out, "tok-lt", &buf);
                        }
                    }
                    None => span(&mut out, "tok-lt", &buf),
                }
            }
            '#' if chars.peek() == Some(&'[') || chars.peek() == Some(&'!') => {
                let mut buf = String::from("#");
                if chars.peek() == Some(&'!') {
                    buf.push('!');
                    chars.next();
                }
                if chars.peek() == Some(&'[') {
                    let mut depth = 0i32;
                    for n in chars.by_ref() {
                        buf.push(n);
                        match n {
                            '[' => depth += 1,
                            ']' => {
                                depth -= 1;
                                if depth == 0 {
                                    break;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                span(&mut out, "tok-attr", &buf);
            }
            c if c.is_ascii_digit() => {
                let mut buf = String::new();
                buf.push(c);
                while let Some(&n) = chars.peek() {
                    if n.is_ascii_alphanumeric() || n == '_' {
                        buf.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                span(&mut out, "tok-num", &buf);
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut buf = String::new();
                buf.push(c);
                while let Some(&n) = chars.peek() {
                    if n.is_alphanumeric() || n == '_' {
                        buf.push(n);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if chars.peek() == Some(&'!') {
                    buf.push('!');
                    chars.next();
                    span(&mut out, "tok-macro", &buf);
                } else if KEYWORDS.contains(&buf.as_str()) {
                    span(&mut out, "tok-kw", &buf);
                } else if buf.chars().next().is_some_and(char::is_uppercase) {
                    span(&mut out, "tok-type", &buf);
                } else {
                    escape_into(&mut out, &buf);
                }
            }
            other => escape_char(&mut out, other),
        }
    }
    out
}

fn span(out: &mut String, class: &str, raw: &str) {
    out.push_str("<span class=\"");
    out.push_str(class);
    out.push_str("\">");
    escape_into(out, raw);
    out.push_str("</span>");
}

fn escape_into(out: &mut String, raw: &str) {
    for c in raw.chars() {
        escape_char(out, c);
    }
}

fn escape_char(out: &mut String, c: char) {
    match c {
        '&' => out.push_str("&amp;"),
        '<' => out.push_str("&lt;"),
        '>' => out.push_str("&gt;"),
        _ => out.push(c),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_keywords_strings_and_comments() {
        let html = rust("fn main() { let x = \"hi\"; // done\n}");
        assert!(html.contains("<span class=\"tok-kw\">fn</span>"));
        assert!(html.contains("<span class=\"tok-kw\">let</span>"));
        assert!(html.contains("<span class=\"tok-str\">\"hi\"</span>"));
        assert!(html.contains("<span class=\"tok-com\">// done</span>"));
    }

    #[test]
    fn distinguishes_lifetimes_from_char_literals() {
        let html = rust("fn f<'a>(c: char) { let x = 'a'; let y = '\\n'; }");
        assert!(html.contains("<span class=\"tok-lt\">'a</span>"));
        assert!(html.contains("<span class=\"tok-str\">'a'</span>"));
        assert!(html.contains("<span class=\"tok-str\">'\\n'</span>"));
    }

    #[test]
    fn colours_types_macros_numbers_and_attributes() {
        let html =
            rust("#[derive(Debug)]\nstruct Pool { n: u32 }\nfn go() { println!(\"{}\", 1_000); }");
        assert!(html.contains("<span class=\"tok-attr\">#[derive(Debug)]</span>"));
        assert!(html.contains("<span class=\"tok-type\">Pool</span>"));
        assert!(html.contains("<span class=\"tok-macro\">println!</span>"));
        assert!(html.contains("<span class=\"tok-num\">1_000</span>"));
    }

    #[test]
    fn handles_raw_strings_and_inner_attributes() {
        let html = rust("#![allow(dead_code)]\nlet p = r#\"a \"quote\" b\"#;");
        assert!(html.contains("<span class=\"tok-attr\">#![allow(dead_code)]</span>"));
        assert!(html.contains("<span class=\"tok-str\">r#\"a \"quote\" b\"#</span>"));
    }

    #[test]
    fn escapes_html_special_characters_in_code_text() {
        let html = rust("if a < b && b > 0 { }");
        assert!(html.contains("&lt;"));
        assert!(html.contains("&gt;"));
        assert!(html.contains("&amp;&amp;"));
    }

    #[test]
    fn ranges_and_decimals_do_not_swallow_following_tokens() {
        let html = rust("let r = 0..16; let pi = 3.14;");
        assert!(
            html.contains("<span class=\"tok-num\">0</span>..<span class=\"tok-num\">16</span>")
        );
        assert!(html.contains("<span class=\"tok-num\">3</span>.<span class=\"tok-num\">14</span>"));
    }
}
