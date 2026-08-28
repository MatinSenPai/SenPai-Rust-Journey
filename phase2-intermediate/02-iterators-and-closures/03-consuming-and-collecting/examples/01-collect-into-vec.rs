//! Two ways to tell `.collect()` what to build: annotate the binding, or
//! turbofish the call itself. Same inference either way — this is purely a
//! choice of *where* you write the target type down.
//!
//!     cargo run -p p2-02-03-consuming-and-collecting --example 01-collect-into-vec

fn main() {
    let episodes = [12, 24, 6, 50, 13];

    // Annotate the binding — Rust reads `Vec<i32>` off `long_runs` and works
    // backwards to figure out what `.collect()` must build.
    let long_runs: Vec<i32> = episodes.iter().filter(|&&n| n >= 12).copied().collect();
    println!("long_runs (annotated binding): {long_runs:?}");

    // Or turbofish the call itself — same inference, written at the call
    // site instead of the `let`. Useful when the value is used right away
    // and there is no binding to hang a type off of.
    let long_runs_turbofish = episodes
        .iter()
        .filter(|&&n| n >= 12)
        .copied()
        .collect::<Vec<i32>>();
    println!("long_runs (turbofish):        {long_runs_turbofish:?}");

    // `Vec<_>` also works in the turbofish — Rust only needs to be told
    // *which* type to build; it can read the element type off `episodes`
    // itself without your help.
    let doubled = episodes.iter().map(|n| n * 2).collect::<Vec<_>>();
    println!("doubled (Vec<_> turbofish):   {doubled:?}");
}
