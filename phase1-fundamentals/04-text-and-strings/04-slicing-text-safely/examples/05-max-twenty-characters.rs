//! DELIBERATELY BROKEN — expected: a run-time panic, "end byte index 20 is
//! not a char boundary". This is the production bug, written the way it is
//! really written: a card component that shows "at most 20 characters".
//!
//!     cargo run -p p1-04-04-slicing-text-safely --example 05-max-twenty-characters --features broken

/// "At most 20 characters" — which is what the ticket said, and is not what
/// this does.
fn card_title(title: &str) -> &str {
    if title.len() <= 20 {
        return title;
    }
    &title[..20]
}

fn main() {
    let catalogue = [
        "Fullmetal Alchemist: Brotherhood",
        "حمله به تایتان",
        "برنامه‌نویسی سیستمی با Rust",
        "شکارچی شیاطین",
    ];

    for title in catalogue {
        let shown = card_title(title);
        println!(
            "{:>2} chars in, {:>2} chars out -> {shown:?}",
            title.chars().count(),
            shown.chars().count()
        );
    }
}
