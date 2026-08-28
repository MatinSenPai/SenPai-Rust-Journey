//! An arm is a pattern: it decides *which variant this is* and pulls that
//! variant's data out, in one step.
//!
//!     cargo run -p p1-05-04-match-in-depth --example 02-binding-out-of-variants

#[derive(Debug)]
enum Progress {
    NotStarted,
    Reading { chapter: u32, of: u32 },
    Finished { rating: u8 },
    Dropped { at: u32, reason: String },
}

struct Entry {
    title: String,
    progress: Progress,
}

/// `chapter` and `of` are new names, bound from the variant's fields. There is
/// no separate "is it Reading?" test and no field access afterwards.
fn describe(progress: &Progress) -> String {
    match progress {
        Progress::NotStarted => "not started".to_string(),
        Progress::Reading { chapter, of } => format!("chapter {chapter} of {of}"),
        Progress::Finished { rating } => format!("finished, {rating}/10"),
        // `..` means "the rest of this variant's fields, whatever they are".
        Progress::Dropped { at, .. } => format!("dropped at chapter {at}"),
    }
}

/// `r @ 9..=10` tests the range *and* keeps the value under the name `r`.
fn shelf(progress: &Progress) -> String {
    match progress {
        Progress::Finished { rating: r @ 9..=10 } => format!("hall of fame ({r}/10)"),
        Progress::Finished { rating } => format!("read once ({rating}/10)"),
        // A bare name is a pattern too: it matches anything and binds it.
        other => format!("still open — {}", describe(other)),
    }
}

/// `..` again, this time keeping the *second* field and skipping the first.
/// `_` on its own arm means "every remaining shape, and I want no name for it".
fn why_dropped(progress: &Progress) -> String {
    match progress {
        Progress::Dropped { reason, .. } => format!("gave up because the {reason}"),
        _ => "not dropped".to_string(),
    }
}

/// Patterns nest as deep as the data does: a struct pattern containing an enum
/// pattern containing a literal.
fn headline(entry: &Entry) -> String {
    match entry {
        Entry {
            title,
            progress: Progress::Finished { rating: 10 },
        } => format!("{title}: a perfect score"),
        Entry {
            title,
            progress: Progress::NotStarted,
        } => format!("{title}: untouched"),
        Entry { title, .. } => format!("{title}: somewhere in the middle"),
    }
}

fn main() {
    let shelf_of = [
        Progress::NotStarted,
        Progress::Reading {
            chapter: 12,
            of: 40,
        },
        Progress::Finished { rating: 9 },
        Progress::Finished { rating: 6 },
        Progress::Dropped {
            at: 3,
            reason: "art changed".to_string(),
        },
    ];

    for progress in &shelf_of {
        println!("{}", describe(progress));
    }

    println!();
    for progress in &shelf_of {
        println!("{}", shelf(progress));
    }

    println!();
    println!("{}", why_dropped(&shelf_of[4]));
    println!("{}", why_dropped(&shelf_of[0]));

    println!();
    let entries = [
        Entry {
            title: "Vinland Saga".to_string(),
            progress: Progress::Finished { rating: 10 },
        },
        Entry {
            title: "Berserk".to_string(),
            progress: Progress::NotStarted,
        },
        Entry {
            title: "Vagabond".to_string(),
            progress: Progress::Reading {
                chapter: 200,
                of: 327,
            },
        },
    ];
    for entry in &entries {
        println!("{}", headline(entry));
    }
}
