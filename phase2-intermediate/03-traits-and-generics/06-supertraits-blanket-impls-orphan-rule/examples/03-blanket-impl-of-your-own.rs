//! Same `Named`/`Greet` pair and the same two types as
//! `01-supertraits-basics.rs` — but this time nobody writes `impl Greet for
//! Villager {}` or `impl Greet for Merchant {}`. One blanket impl gives every
//! `Named` type `Greet` for free, exactly like the standard library did for
//! `Into`.

trait Named {
    fn name(&self) -> String;
}

trait Greet: Named {
    fn greet(&self) -> String {
        format!("Hello, I'm {}!", self.name())
    }
}

impl<T: Named> Greet for T {}

struct Villager {
    name: String,
}

impl Named for Villager {
    fn name(&self) -> String {
        self.name.clone()
    }
}

struct Merchant {
    name: String,
    shop: String,
}

impl Named for Merchant {
    fn name(&self) -> String {
        format!("{} of {}", self.name, self.shop)
    }
}

fn main() {
    let rin = Villager {
        name: "Rin".to_string(),
    };
    println!("{}", rin.greet());

    let tom = Merchant {
        name: "Old Tom".to_string(),
        shop: "the Blacksmith".to_string(),
    };
    println!("{}", tom.greet());
}
