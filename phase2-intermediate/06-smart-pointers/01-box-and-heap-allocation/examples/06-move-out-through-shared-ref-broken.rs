//! DELIBERATELY BROKEN — expected: E0507
//! Run `cargo run --example 06-move-out-through-shared-ref-broken --features broken`
//! and read the error.

struct Profile {
    name: String,
}

fn take_it(boxed: &Box<Profile>) -> Profile {
    **boxed
}

fn main() {
    let boxed = Box::new(Profile {
        name: String::from("Matin"),
    });
    let owned = take_it(&boxed);
    println!("{}", owned.name);
}
