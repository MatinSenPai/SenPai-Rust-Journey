//! Two pieces of real logic, each written twice: once as a `match`, once as
//! a combinator chain. Read both pairs and judge for yourself which reads
//! better — the point of this file is that the answer is not always the
//! same.

// --- Case 1: a plain transform with a default. The chain wins. ---------

fn greeting_match(name: Option<&str>) -> String {
    match name {
        Some(n) if !n.is_empty() => format!("Hello, {n}!"),
        _ => "Hello, stranger!".to_string(),
    }
}

fn greeting_combinator(name: Option<&str>) -> String {
    name.filter(|n| !n.is_empty())
        .map(|n| format!("Hello, {n}!"))
        .unwrap_or_else(|| "Hello, stranger!".to_string())
}

// --- Case 2: three different rules, one of which needs an outside value.
// The match wins. ---------------------------------------------------------

fn shipping_cost_match(weight_kg: Option<f64>, express: bool) -> f64 {
    match weight_kg {
        Some(w) if w <= 0.0 => 0.0,
        Some(w) if express => w * 2.5 + 10.0,
        Some(w) => w * 1.2,
        None => 5.0, // unknown weight: flat handling fee
    }
}

fn shipping_cost_combinator(weight_kg: Option<f64>, express: bool) -> f64 {
    weight_kg
        .map(|w| {
            if w <= 0.0 {
                0.0
            } else if express {
                w * 2.5 + 10.0
            } else {
                w * 1.2
            }
        })
        .unwrap_or(5.0)
}

fn main() {
    println!("-- greeting: match vs combinator --");
    for name in [Some("Sam"), Some(""), None] {
        let a = greeting_match(name);
        let b = greeting_combinator(name);
        println!(
            "name={name:?} -> match={a:?} combinator={b:?} (equal: {})",
            a == b
        );
    }

    println!("\n-- shipping cost: match vs combinator --");
    let cases = [
        (Some(0.0), false),
        (Some(2.0), true),
        (Some(2.0), false),
        (None, true),
    ];
    for (weight, express) in cases {
        let a = shipping_cost_match(weight, express);
        let b = shipping_cost_combinator(weight, express);
        println!(
            "weight={weight:?}, express={express} -> match={a} combinator={b} (equal: {})",
            a == b
        );
    }
}
