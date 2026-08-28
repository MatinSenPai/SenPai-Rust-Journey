//! A generic function bounded only by the subtrait can still call the
//! supertrait's methods — `T: Greet` is proof enough that `T: Named` too.

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

fn print_intro<T: Greet>(entity: &T) {
    println!("{}", entity.greet());
    println!("(bare name: {})", entity.name());
}

fn main() {
    let rin = Villager {
        name: "Rin".to_string(),
    };
    print_intro(&rin);
}
