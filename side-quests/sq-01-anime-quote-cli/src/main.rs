// This file is provided for you — no `todo!()`s here, only in `lib.rs`.
// One new bit of syntax you'll see below: `let Some(x) = opt else { ... };`
// ("let-else") — it's `if let` turned inside out: bind `x` if `opt` matches
// `Some(x)`, or run the `else` block (which must diverge — `return`,
// `break`, `continue`, or `panic!`) if it doesn't. Handy specifically for
// "bail out early if this doesn't match" without nesting the rest of the
// function inside an `if let` block.
use sq_01_anime_quote_cli::{
    all_quotes, find_by_anime, find_by_character, format_quote, pick_random,
};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let quotes = all_quotes();

    match args.first().map(String::as_str) {
        None => match pick_random(&quotes) {
            Some(q) => println!("{}", format_quote(q)),
            None => println!("No quotes available."),
        },
        Some("list") => {
            for q in &quotes {
                println!("{}", format_quote(q));
            }
        }
        Some("anime") => {
            let Some(query) = args.get(1) else {
                eprintln!("usage: anime <name>");
                return;
            };
            let results = find_by_anime(&quotes, query);
            print_results(&results, query);
        }
        Some("character") => {
            let Some(query) = args.get(1) else {
                eprintln!("usage: character <name>");
                return;
            };
            let results = find_by_character(&quotes, query);
            print_results(&results, query);
        }
        Some(other) => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: [list | anime <name> | character <name>]");
        }
    }
}

fn print_results(results: &[&sq_01_anime_quote_cli::Quote], query: &str) {
    if results.is_empty() {
        println!("No quotes found for \"{query}\".");
        return;
    }
    for q in results {
        println!("{}", format_quote(q));
    }
}
