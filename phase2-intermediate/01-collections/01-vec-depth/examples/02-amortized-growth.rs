//! Every time `push` needs more room than the buffer has, `Vec` allocates a
//! new buffer roughly DOUBLE the size, copies every existing element over,
//! and frees the old buffer. Watch exactly where those jumps land.
//!
//!     cargo run -p p2-01-01-vec-depth --example 02-amortized-growth

fn main() {
    let mut episodes: Vec<u32> = Vec::new();
    let mut last_capacity = episodes.capacity();
    let mut elements_copied = 0usize;

    for episode in 1..=40 {
        episodes.push(episode);
        if episodes.capacity() != last_capacity {
            elements_copied += episodes.len() - 1; // everything but the new element moved
            println!(
                "len {:>2} -> capacity jumped {:>2} to {:>2}",
                episodes.len(),
                last_capacity,
                episodes.capacity()
            );
            last_capacity = episodes.capacity();
        }
    }

    println!();
    println!("40 pushes total, only {elements_copied} element-moves happened during a regrow");
}
