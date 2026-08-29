//! `LazyLock` is the declarative sibling of `OnceLock`: write the recipe once
//! as a `static`, and the first access anywhere builds it.

use std::sync::LazyLock;
use std::thread;

static SQUARES: LazyLock<Vec<u32>> = LazyLock::new(|| (0..10).map(|n| n * n).collect());

fn main() {
    let handles: Vec<_> = (0..4)
        .map(|id| thread::spawn(move || (id, SQUARES[id])))
        .collect();
    for handle in handles {
        let (id, value) = handle.join().unwrap();
        println!("SQUARES[{id}] = {value}");
    }
    println!("table has {} entries", SQUARES.len());
}
