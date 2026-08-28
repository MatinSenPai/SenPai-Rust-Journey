//! `Box::new` moves a value onto the heap; the `Box` itself — a plain
//! pointer — stays on the stack and owns what it points at.

struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn distance_from_origin(&self) -> f64 {
        ((self.x * self.x + self.y * self.y) as f64).sqrt()
    }
}

struct Loud(String);

impl Drop for Loud {
    fn drop(&mut self) {
        println!("dropping: {}", self.0);
    }
}

fn main() {
    let boxed: Box<i32> = Box::new(5);
    println!("boxed  = {boxed}");
    println!("*boxed = {}", *boxed);

    // `Point` has no method of its own named `distance_from_origin` on
    // `Box<Point>` — the compiler follows `Deref` for you and calls it on
    // the `Point` at the other end. No `(*boxed_point).distance_from_origin()`.
    let boxed_point = Box::new(Point { x: 3, y: 4 });
    println!("distance = {}", boxed_point.distance_from_origin());

    println!("--- entering inner scope ---");
    {
        let inner = Box::new(Loud(String::from("inner")));
        println!("inner alive: {}", inner.0);
    }
    println!("--- inner scope ended ---");
}
