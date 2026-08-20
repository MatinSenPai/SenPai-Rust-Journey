//! The stylesheet and the client script, embedded in the binary.
//!
//! Both live as real files under `web-ui/assets/` rather than as string
//! literals in Rust: editors syntax-highlight them, the layer comments in the
//! CSS survive, and a diff to the design shows up as a diff to a `.css` file.
//! `include_str!` means there is still no build step and nothing to serve from
//! disk at run time.

pub const CSS: &str = include_str!("../assets/style.css");
pub const JS: &str = include_str!("../assets/app.js");

/// Runs before first paint so a reader who chose light does not get a flash of
/// dark. It has to be inline and blocking, which is why it is not in `app.js`.
pub const THEME_BOOTSTRAP: &str = "try{var t=localStorage.getItem('course-ui-theme');\
if(t==='light'||t==='dark')document.documentElement.setAttribute('data-theme',t)}catch(e){}";

#[cfg(test)]
mod tests {
    use super::*;

    /// The bug this stylesheet was rewritten to fix: `bdi` and `[dir=ltr]` wrap
    /// inline English inside Persian prose, and giving them the monospace stack
    /// made the typeface change mid-sentence.
    #[test]
    fn inline_english_is_not_given_the_monospace_stack() {
        let mono_rule = CSS
            .lines()
            .find(|line| line.starts_with("code, pre, kbd, samp"))
            .expect("the monospace rule exists");
        assert!(!mono_rule.contains("bdi"));
        assert!(!mono_rule.contains("[dir='ltr']"));
    }

    #[test]
    fn both_themes_define_every_colour_token() {
        // A token defined in one theme and missing in the other reads as a
        // silent inherit — usually invisible text on its own background.
        let tokens = [
            "--bg:",
            "--surface:",
            "--surface-2:",
            "--text:",
            "--text-soft:",
            "--muted:",
            "--line:",
            "--line-strong:",
            "--accent:",
            "--accent-soft:",
            "--on-accent:",
            "--focus:",
            "--shadow:",
        ];
        let dark_start = CSS.find(":root[data-theme='dark']").expect("dark block");
        let dark_block = &CSS[dark_start..dark_start + 900];
        for token in tokens {
            assert!(CSS.contains(token), "light palette is missing {token}");
            assert!(
                dark_block.contains(token),
                "dark palette is missing {token}"
            );
        }
    }

    #[test]
    fn the_hero_heading_is_defined_once() {
        // The previous stylesheet appended four patch layers and ended up
        // defining this selector three times.
        assert_eq!(CSS.matches("\n.hero h1 {").count(), 1);
    }
}
