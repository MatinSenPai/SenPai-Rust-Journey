//! DELIBERATELY BROKEN — expected: E0277
//!
//!     cargo run -p p2-03-06-supertraits-blanket-impls-orphan-rule --example 06-missing-supertrait-impl --features broken

trait Named {
    fn name(&self) -> String;
}

trait Greet: Named {
    fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name())
    }
}

struct Villager {
    name: String,
}

// Missing: `impl Named for Villager`. `Greet` requires it as a supertrait.
impl Greet for Villager {}

fn main() {
    let rin = Villager {
        name: "Rin".to_string(),
    };
    println!("{}", rin.greet());
}
