//! `.drain(range)` removes that range from the `Vec` AND hands it back to
//! you as an iterator, in one pass — the `Vec` itself is still yours to use
//! afterward, it just has fewer elements.
//!
//!     cargo run -p p2-01-01-vec-depth --example 04-drain

fn main() {
    let mut queue: Vec<u32> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    println!("queue before: {queue:?}");

    let mut just_watched: Vec<u32> = Vec::new();
    for episode in queue.drain(0..4) {
        just_watched.push(episode);
    }
    println!("just watched: {just_watched:?}");
    println!("queue after:  {queue:?}");

    queue.push(11);
    println!();
    println!("still usable, pushed 11: {queue:?}");

    let capacity_before = queue.capacity();
    let mut everything: Vec<u32> = Vec::new();
    for episode in queue.drain(..) {
        everything.push(episode);
    }
    println!();
    println!("drain(..) took:        {everything:?}");
    println!(
        "queue after drain(..): {queue:?}, capacity still {} (was {capacity_before})",
        queue.capacity()
    );
}
