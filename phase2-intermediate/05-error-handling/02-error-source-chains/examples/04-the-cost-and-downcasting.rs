//! The cost of `Box<dyn Error>`: once erased, a caller cannot `match` on
//! which concrete error type actually happened (see "Errors you will
//! meet" for the compiler saying so directly). `.downcast_ref::<T>()` can
//! recover the concrete type — but only if the caller already suspects
//! which `T` to ask for, one guess at a time, not as an exhaustive match.
//!
//!     cargo run -p p2-05-02-error-source-chains --example 04-the-cost-and-downcasting

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
struct ConfigError(String);

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "config problem: {}", self.0)
    }
}

impl Error for ConfigError {}

fn might_fail(as_config_error: bool) -> Result<(), Box<dyn Error>> {
    if as_config_error {
        return Err(Box::new(ConfigError("missing field: name".to_string())));
    }
    "not-a-number".parse::<u32>()?;
    Ok(())
}

// `downcast_ref` lives only on `dyn Error + 'static` — one more place the
// `'static` bound from `source()`'s own signature shows back up.
fn describe(err: &(dyn Error + 'static)) -> &'static str {
    if err.downcast_ref::<ConfigError>().is_some() {
        "a ConfigError"
    } else if err.downcast_ref::<ParseIntError>().is_some() {
        "a ParseIntError"
    } else {
        "something else"
    }
}

fn main() {
    let a = might_fail(true).unwrap_err();
    let b = might_fail(false).unwrap_err();
    // `&*a`, not `&a`: one explicit deref through the `Box` first, landing
    // straight on `&(dyn Error + 'static)` with no further coercion needed.
    println!("a: {a} ({})", describe(&*a));
    println!("b: {b} ({})", describe(&*b));
}
