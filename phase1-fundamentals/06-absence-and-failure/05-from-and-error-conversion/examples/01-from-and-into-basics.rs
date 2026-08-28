//! `From` is a conversion you implement once. `Into` is what you get for
//! writing it — nobody implements `Into` by hand.
//!
//!     cargo run -p p1-06-05-from-and-error-conversion --example 01-from-and-into-basics

struct Kilograms(f64);
struct Grams(f64);
struct Pounds(f64);

// One `impl From<Source> for Target` per direction you actually need.
impl From<Kilograms> for Grams {
    fn from(value: Kilograms) -> Self {
        Grams(value.0 * 1000.0)
    }
}

impl From<Kilograms> for Pounds {
    fn from(value: Kilograms) -> Self {
        Pounds(value.0 * 2.20462)
    }
}

fn print_grams(amount: Grams) {
    println!("as grams:   {}", amount.0);
}

fn main() {
    // You have called this since Phase 0 without a name for it: this *is*
    // `<String as From<&str>>::from`.
    let name = String::from("Matin");
    println!("String::from: {name}");

    // A widening numeric conversion can never lose information, so the
    // standard library ships it as `From` too.
    let wide: u64 = u64::from(42_u32);
    println!("u64::from:  {wide}");

    println!();

    // `Grams::from(Kilograms(5.0))` — the explicit form, spelled out.
    let explicit = Grams::from(Kilograms(5.0));
    println!("Grams::from:  {}", explicit.0);

    // `.into()` is the same conversion read the other way. It exists only
    // because `impl From<Kilograms> for Grams` exists above — this file
    // never writes `impl Into` anywhere.
    let box_weight = Kilograms(5.0);
    let in_grams: Grams = box_weight.into();
    println!(".into() (typed let): {}", in_grams.0);

    // A function parameter pins the target just as well as a `let` type
    // does — the compiler reads `print_grams`'s signature and knows the
    // `.into()` must produce a `Grams`.
    print_grams(Kilograms(2.0).into());

    // The same source type, a different target: `Kilograms` also converts
    // to `Pounds`, because a *second* `From` impl exists for it.
    let in_pounds: Pounds = Kilograms(5.0).into();
    println!(".into() (as Pounds): {}", in_pounds.0);
}
