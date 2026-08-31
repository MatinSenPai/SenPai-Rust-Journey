//! The textbook `RwLock` use case: a read-heavy, write-rare shared cache.
//! Several reader threads look at it at once; one writer thread updates it.

use std::sync::{Arc, RwLock};
use std::thread;

fn main() {
    let cache = Arc::new(RwLock::new(vec![
        String::from("Frieren"),
        String::from("Bocchi the Rock!"),
    ]));

    let before: Vec<_> = (0..3)
        .map(|id| {
            let cache = Arc::clone(&cache);
            thread::spawn(move || (id, cache.read().unwrap().len()))
        })
        .collect();
    for handle in before {
        let (id, count) = handle.join().unwrap();
        println!("reader {id} saw {count} cached titles");
    }

    let writer_cache = Arc::clone(&cache);
    thread::spawn(move || {
        writer_cache
            .write()
            .unwrap()
            .push(String::from("Made in Abyss"));
    })
    .join()
    .unwrap();

    let after = cache.read().unwrap().len();
    println!("after the writer, the cache holds {after} titles");
}
