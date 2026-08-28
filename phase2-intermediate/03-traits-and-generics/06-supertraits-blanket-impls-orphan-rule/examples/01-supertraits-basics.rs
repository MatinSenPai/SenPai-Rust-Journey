//! A supertrait: `Greet` cannot be implemented for a type until that type
//! already implements `Named`.

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

impl Named for Villager {
    fn name(&self) -> String {
        self.name.clone()
    }
}

impl Greet for Villager {}

struct Merchant {
    name: String,
    shop: String,
}

impl Named for Merchant {
    fn name(&self) -> String {
        format!("{} of {}", self.name, self.shop)
    }
}

impl Greet for Merchant {}

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
