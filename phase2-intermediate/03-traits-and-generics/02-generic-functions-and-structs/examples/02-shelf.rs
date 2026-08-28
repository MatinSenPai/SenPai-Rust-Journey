//! Run: cargo run -p p2-03-02-generic-functions-and-structs --example 02-shelf

struct Shelf<T> {
    items: Vec<T>,
}

impl<T> Shelf<T> {
    fn new() -> Self {
        Shelf { items: Vec::new() }
    }

    fn add(&mut self, item: T) {
        self.items.push(item);
    }

    fn len(&self) -> usize {
        self.items.len()
    }

    // Only this method needs ordering, so only this method asks for it.
    fn highest(&self) -> Option<&T>
    where
        T: PartialOrd,
    {
        let mut best = self.items.first()?;
        for item in &self.items {
            if item > best {
                best = item;
            }
        }
        Some(best)
    }

    // Two extra requirements, scoped to this one method: an owned copy needs
    // `Clone`, and the caller-supplied predicate needs its own bound.
    fn find_and_clone<F>(&self, matches: F) -> Option<T>
    where
        T: Clone,
        F: Fn(&T) -> bool,
    {
        self.items.iter().find(|item| matches(item)).cloned()
    }
}

fn main() {
    let mut ratings: Shelf<u32> = Shelf::new();
    ratings.add(7);
    ratings.add(9);
    ratings.add(6);
    println!("ratings on shelf: {}", ratings.len());
    println!("highest rating: {:?}", ratings.highest());

    let found = ratings.find_and_clone(|&r| r >= 8);
    println!("first rating >= 8: {found:?}");
}
