use p0_03_cargo_basics::format_greeting;
use rand::Rng;

fn main() {
    // `std::env::args()` gives you the command-line arguments; the first one
    // is always the binary's own path, so `.skip(1)` drops it. Phase 1
    // explains iterators/`Option` properly — for now, just trust this reads
    // "the name, if one was given, else default to \"World\"".
    let mut args = std::env::args().skip(1);
    let name = args.next().unwrap_or_else(|| "World".to_string());
    let times: u32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(1);

    println!("{}", format_greeting(&name, times));

    // A tiny taste of using an external crate: `rand` picks a random pick-me-up.
    let lines = [
        "Keep going.",
        "One `todo!()` at a time.",
        "You've got this.",
    ];
    let pick = rand::thread_rng().gen_range(0..lines.len());
    println!("\n{}", lines[pick]);
}
