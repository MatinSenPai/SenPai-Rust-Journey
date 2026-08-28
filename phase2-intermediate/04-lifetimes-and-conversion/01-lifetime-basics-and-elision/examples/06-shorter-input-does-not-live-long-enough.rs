//! DELIBERATELY BROKEN — expected: E0597
//!
//! `longest`'s signature promises its return value is valid for `'a` — the
//! lifetime shared by *both* inputs. `short_lived` stops existing at the
//! closing `}` below, so nothing the compiler can prove satisfies `'a` past
//! that point, even though this particular call happened to return the
//! *other* argument.
//!
//!     cargo run -p p2-04-01-lifetime-basics-and-elision --example 06-shorter-input-does-not-live-long-enough --features broken

fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}

fn main() {
    let long_lived = String::from("a very long-lived string");
    let result;
    {
        let short_lived = String::from("short");
        result = longest(long_lived.as_str(), short_lived.as_str());
    }
    println!("longest: {result}");
}
