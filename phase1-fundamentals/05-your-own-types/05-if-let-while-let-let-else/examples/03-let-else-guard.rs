//! `let ... else` — bind it, or leave. The guard clause from 1.1.5, applied
//! to a pattern instead of to a condition.
//!
//!     cargo run -p p1-05-05-if-let-while-let-let-else --example 03-let-else-guard

#[derive(Debug)]
enum Entry {
    Watching { episode: u32 },
    Completed,
    PlanToWatch,
}

/// The shape you would have written yesterday: the real work sinks into a
/// block, and it sinks one level further for every guard you add.
fn gap_nested(entry: &Entry, latest: u32) -> u32 {
    if let Entry::Watching { episode } = entry {
        latest.saturating_sub(*episode)
    } else {
        0
    }
}

/// The same rule, flat. `episode` is in scope for the rest of the function,
/// and the function's real work is at its top level where you can read it.
fn gap_flat(entry: &Entry, latest: u32) -> u32 {
    let Entry::Watching { episode } = entry else {
        return 0;
    };
    latest.saturating_sub(*episode)
}

fn main() {
    let entries = [
        Entry::Watching { episode: 7 },
        Entry::Completed,
        Entry::PlanToWatch,
    ];

    println!("latest episode out: 12");
    for entry in &entries {
        println!(
            "{:<28} nested {}   flat {}",
            format!("{entry:?}"),
            gap_nested(entry, 12),
            gap_flat(entry, 12)
        );
    }
}
