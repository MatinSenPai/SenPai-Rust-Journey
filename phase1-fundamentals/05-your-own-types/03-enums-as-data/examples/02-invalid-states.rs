//! The same domain modelled twice. Only one of the two can be built wrong.
//!
//!     cargo run -p p1-05-03-enums-as-data --example 02-invalid-states

// `{:?}` does read every field below, but dead-code analysis deliberately
// ignores a derived `Debug` — so without this the run is buried in warnings
// about data that is in fact used.
#![allow(dead_code)]

/// The shape you reach for without enums: a tag field, plus a field for
/// every piece of data that *any* state might need.
#[derive(Debug)]
struct LooseEntry {
    status: String,
    episode: u32,
    score: u8,
    reason: String,
}

/// The same domain as an enum. Each state carries exactly the data that
/// state needs, and nothing else.
#[derive(Debug)]
enum Entry {
    Planned,
    Watching(u32),
    Rated { score: u8 },
    Dropped { episode: u32, reason: String },
}

fn main() {
    // Sensible: watching, on episode 7, no score yet, no reason to drop it.
    let sensible = LooseEntry {
        status: String::from("watching"),
        episode: 7,
        score: 0,
        reason: String::new(),
    };
    println!("sensible: {sensible:?}");

    // Nonsense: it is *planned*, yet it is somehow on episode 40, scored 9,
    // and carries a reason for having been dropped. Three states at once.
    // The compiler has no objection whatsoever.
    let nonsense = LooseEntry {
        status: String::from("planned"),
        episode: 40,
        score: 9,
        reason: String::from("too slow"),
    };
    println!("nonsense: {nonsense:?}");

    // And a misspelt tag is invisible too — `status` is a String, and every
    // String is a valid String.
    let typo = LooseEntry {
        status: String::from("wathcing"),
        episode: 7,
        score: 0,
        reason: String::new(),
    };
    println!("typo:     {typo:?}");

    // Now the enum. These four are every value an `Entry` can hold.
    println!();
    println!("planned:  {:?}", Entry::Planned);
    println!("watching: {:?}", Entry::Watching(7));
    println!("rated:    {:?}", Entry::Rated { score: 9 });
    println!(
        "dropped:  {:?}",
        Entry::Dropped {
            episode: 40,
            reason: String::from("too slow"),
        }
    );

    // There is no line you can write that produces a `Planned` carrying an
    // episode, or a `Rated` carrying a reason. The nonsense above is not
    // caught here — it is not expressible here.
    println!();
    println!("no fifth shape, and no half-filled one either");
}
