//! The only way to get the value out of an `Option<T>` is to handle both
//! cases. `match` and `if let` are the two ways to do that.
//!
//!     cargo run -p p1-06-01-option-and-null-safety --example 02-match-and-if-let

fn describe(rating: Option<u8>) -> String {
    match rating {
        Some(score) => format!("rated {score}/10"),
        None => "not rated yet".to_string(),
    }
}

fn main() {
    println!("{}", describe(Some(9)));
    println!("{}", describe(None));

    // `if let` is the same match with the "do nothing" arm left out — from
    // 1.5.5. It reads well when only one case has work to do.
    let maybe_name: Option<&str> = Some("Matin");
    if let Some(name) = maybe_name {
        println!();
        println!("if let:   hello, {name}");
    }

    let nobody: Option<&str> = None;
    if let Some(name) = nobody {
        println!("if let:   hello, {name}");
    } else {
        println!("if let:   nobody to greet");
    }

    // The exhaustiveness `match` gives you is not free with `if let` — that
    // trade is exactly what 1.5.5 warned you to make on purpose. Here it's
    // fine: "no rating" really is the one other case.
}
