//! The same fix, a different-looking problem: `Circle` and `Rectangle` are
//! different sizes, so only a fixed-size pointer to each — `Box<dyn
//! Shape>` — can sit side by side in one `Vec`.
//!
//!     cargo run -p p2-06-02-recursive-types-and-trait-objects --example 05-vec-box-dyn-shape

trait Shape {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

fn main() {
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 2.0 }),
        Box::new(Rectangle {
            width: 3.0,
            height: 4.0,
        }),
    ];

    let total: f64 = shapes.iter().map(|shape| shape.area()).sum();
    println!("total area = {total:.2}");
}
