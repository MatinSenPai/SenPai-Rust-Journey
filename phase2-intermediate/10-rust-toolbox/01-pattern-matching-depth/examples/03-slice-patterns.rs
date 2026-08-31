//! Slices match on shape, and the compiler proves three arms — `[]`,
//! `[x]`, `[first, .., last]` — cover every possible length: 0, 1, and 2
//! or more. `rest @ ..` names the part `..` would otherwise throw away,
//! which is how you reach the elements strictly *between* first and last.
//!
//!     cargo run -p p2-10-01-pattern-matching-depth --example 03-slice-patterns

fn describe(samples: &[u64]) -> String {
    match samples {
        [] => "no samples".to_string(),
        [only] => format!("1 sample: {only}ms"),
        [first, .., last] => format!("first {first}ms, last {last}ms"),
    }
}

fn middle(samples: &[u64]) -> &[u64] {
    match samples {
        [_, rest @ .., _] => rest,
        _ => &[],
    }
}

fn main() {
    for samples in [[].as_slice(), &[42], &[5, 80, 9, 12], &[7, 3]] {
        println!("{:<16} -> {}", format!("{samples:?}"), describe(samples));
    }

    println!("---");

    for samples in [[].as_slice(), &[42], &[7, 3], &[5, 80, 9, 12]] {
        println!(
            "{:<16} -> middle {:?}",
            format!("{samples:?}"),
            middle(samples)
        );
    }
}
