//! The central point of this whole lesson: building a chain of adapters
//! does not run anything. Only something that calls `.next()` — here, a
//! plain `for` loop — actually pulls values through it.
//!
//!     cargo run -p p2-02-02-iterator-adapters --example 09-the-pipeline-is-just-a-plan

fn main() {
    let shows = vec!["Frieren", "Bocchi the Rock!", "Mushoku Tensei", "AOT"];

    println!("building the pipeline...");
    let pipeline = shows
        .iter()
        .map(|title| {
            println!("  map saw:    {title}");
            title.to_uppercase()
        })
        .filter(|title| {
            println!("  filter saw: {title}");
            title.len() > 8
        });
    println!("pipeline built — nothing printed above from map or filter.");

    println!();
    println!("now a `for` loop consumes it:");
    for title in pipeline {
        println!("  got:        {title}");
    }
}
