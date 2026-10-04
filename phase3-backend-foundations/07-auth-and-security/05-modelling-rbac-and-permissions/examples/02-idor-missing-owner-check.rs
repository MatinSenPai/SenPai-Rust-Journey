//! A silent logic bug: this compiles and runs, and it is wrong.
//! `edit_review` checks that the caller is logged in, never whose review it is.
//! Run: cargo run -p p3-07-05-modelling-rbac-and-permissions --example 02-idor-missing-owner-check

use std::collections::HashMap;

struct Review {
    author: u64,
    text: String,
}

/// Only asks "is somebody logged in?", never "is it their review?".
fn edit_review(
    reviews: &mut HashMap<u64, Review>,
    logged_in: Option<u64>,
    id: u64,
    text: &str,
) -> u16 {
    if logged_in.is_none() {
        return 401;
    }
    match reviews.get_mut(&id) {
        Some(review) => {
            review.text = text.to_string();
            200
        }
        None => 404,
    }
}

fn main() {
    let mut reviews = HashMap::from([(
        1,
        Review {
            author: 1,
            text: "Slow, quiet, perfect.".to_string(),
        },
    )]);
    let bob = 2;
    let status = edit_review(&mut reviews, Some(bob), 1, "this anime is trash");
    println!("bob (id {bob}) edits review 1: {status}");
    println!(
        "review 1 now says {:?}, author still {}",
        reviews[&1].text, reviews[&1].author
    );
}
