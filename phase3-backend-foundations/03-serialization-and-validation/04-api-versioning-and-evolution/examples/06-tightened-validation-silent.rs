//! No error at all: the server tightens a rule between releases and a request
//! that was valid yesterday is rejected today. The types did not change, so
//! nothing compiles differently. Only a test that replays an old request
//! catches it.
//!
//!     cargo run -p p3-03-04-api-versioning-and-evolution --example 06-tightened-validation-silent

use validator::Validate;

#[derive(Validate)]
struct CreateAnimeV1 {
    #[validate(length(min = 1, max = 200))]
    title: String,
}

#[derive(Validate)]
struct CreateAnimeV2 {
    #[validate(length(min = 1, max = 50))] // "just a cleanup"
    title: String,
}

fn main() {
    let title = "Sousou no Frieren: Beyond Journey's End (the complete season)".to_string();
    println!("{} chars", title.chars().count());
    let v1 = CreateAnimeV1 {
        title: title.clone(),
    };
    println!("v1 accepts: {}", v1.validate().is_ok());
    println!("v2 accepts: {}", CreateAnimeV2 { title }.validate().is_ok());
}
