//! DELIBERATELY BROKEN — expected: E0308
//! Run `cargo run -p p3-03-02-validation --example 05-custom-wrong-signature-broken --features broken`
//! and read the error. The custom rule returns `bool`, not a `Result`.

use validator::Validate;

fn no_spaces(value: &str) -> bool {
    !value.contains(' ')
}

#[derive(Validate)]
struct Handle {
    #[validate(custom(function = "no_spaces"))]
    name: String,
}

fn main() {
    let h = Handle {
        name: "ma tin".into(),
    };
    println!("{:?}", h.validate());
}
