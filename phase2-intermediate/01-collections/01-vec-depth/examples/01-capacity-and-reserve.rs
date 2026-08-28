//! `Vec::new()` starts at capacity zero and grows only as it needs to.
//! `Vec::with_capacity(n)` reserves that room immediately, for when you
//! already have a decent estimate — no guessing, no repeated regrowing.
//!
//!     cargo run -p p2-01-01-vec-depth --example 01-capacity-and-reserve

fn show(label: &str, v: &Vec<u32>) {
    println!("{label}: len {}, capacity {}", v.len(), v.capacity());
}

fn main() {
    let mut surprised: Vec<u32> = Vec::new();
    let mut planned: Vec<u32> = Vec::with_capacity(8);
    show("Vec::new()           ", &surprised);
    show("Vec::with_capacity(8)", &planned);

    for episode in 1..=8 {
        surprised.push(episode);
        planned.push(episode);
    }
    println!();
    show("surprised, after 8 pushes", &surprised);
    show("planned,   after 8 pushes", &planned);

    planned.reserve(20);
    println!();
    show("planned, after .reserve(20)", &planned);

    for _ in 0..6 {
        planned.pop();
    }
    planned.shrink_to_fit();
    show("planned, 2 left + .shrink_to_fit()", &planned);
}
