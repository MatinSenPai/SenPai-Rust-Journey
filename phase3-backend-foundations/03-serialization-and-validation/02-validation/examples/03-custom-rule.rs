//! A custom rule is a plain function returning `Result<(), ValidationError>`.
//! Run: `cargo run -p p3-03-02-validation --example 03-custom-rule`

use validator::{Validate, ValidationError};

fn no_spaces(value: &str) -> Result<(), ValidationError> {
    if value.contains(' ') {
        return Err(ValidationError::new("no_spaces").with_message("no spaces allowed".into()));
    }
    Ok(())
}

#[derive(Validate)]
struct Handle {
    #[validate(length(min = 3), custom(function = "no_spaces"))]
    name: String,
}

fn main() {
    for name in ["matin", "ma tin", "m "] {
        let result = Handle { name: name.into() }.validate();
        println!("{name:?} -> {result:?}");
    }
}
