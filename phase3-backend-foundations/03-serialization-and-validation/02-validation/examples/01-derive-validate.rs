//! `#[derive(Validate)]` on a flat struct: what `validate()` returns.
//! Run: `cargo run -p p3-03-02-validation --example 01-derive-validate`

use validator::Validate;

#[derive(Debug, Validate)]
struct Rating {
    #[validate(range(min = 1, max = 10, message = "rating must be between 1 and 10"))]
    score: u8,
    #[validate(length(min = 1, max = 20))]
    title: String,
    #[validate(email)]
    contact: String,
    #[validate(length(max = 5))]
    note: Option<String>,
}

fn main() {
    let good = Rating {
        score: 9,
        title: "Frieren".into(),
        contact: "matin@example.com".into(),
        note: None,
    };
    println!("good: {:?}", good.validate());
    let long = Rating {
        note: Some("far too long".into()),
        ..good
    };
    println!("long note: {:?}", long.validate().is_err());

    let bad = Rating {
        score: 15,
        title: String::new(),
        contact: "nope".into(),
        note: None,
    };
    let errors = bad.validate().unwrap_err();
    let fields = errors.field_errors();
    let mut names: Vec<_> = fields.keys().collect();
    names.sort();
    for name in names {
        for e in fields[name] {
            println!("{name}: code={} message={:?}", e.code, e.message);
        }
    }
}
