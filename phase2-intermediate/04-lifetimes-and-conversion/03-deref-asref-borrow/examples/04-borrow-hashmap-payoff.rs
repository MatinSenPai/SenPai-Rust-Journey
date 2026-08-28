//! `cargo run -p p2-04-03-deref-asref-borrow --example 04-borrow-hashmap-payoff`
//!
//! The `ratings` map is the exact one 2.1.2 built and asked you to trust.
//! `find_by_ref` is not a `HashMap` method — it is a hand-written function
//! whose bound (`K: Borrow<Q>, Q: Hash + Eq + ?Sized`) mirrors the real
//! `HashMap::get`, so you can see the mechanism instead of trusting it.

use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;

fn find_by_ref<'a, K, Q, V>(map: &'a HashMap<K, V>, key: &Q) -> Option<&'a V>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq + ?Sized,
{
    map.get(key)
}

fn main() {
    let mut ratings: HashMap<String, u8> = HashMap::new();
    ratings.insert(String::from("Frieren"), 10);
    ratings.insert(String::from("Bocchi the Rock"), 9);

    println!("{:?}", find_by_ref(&ratings, "Frieren"));

    let name = String::from("Bocchi the Rock");
    println!("{:?}", find_by_ref(&ratings, &name));
}
