//! Compiles, runs, and quietly lets bad data through: the inner struct has
//! rules, but the outer field is not marked `#[validate(nested)]`.
//! Run: `cargo run -p p3-03-02-validation --example 06-forgot-nested-trap`

use validator::Validate;

#[derive(Validate)]
struct Reviewer {
    #[validate(email)]
    email: String,
}

#[derive(Validate)]
struct Review {
    #[validate(length(min = 1))]
    title: String,
    reviewer: Reviewer,
}

fn main() {
    let review = Review {
        title: "Frieren".into(),
        reviewer: Reviewer {
            email: "not-an-email".into(),
        },
    };
    println!("outer validate: {:?}", review.validate());
    println!("inner has errors: {}", review.reviewer.validate().is_err());
}
