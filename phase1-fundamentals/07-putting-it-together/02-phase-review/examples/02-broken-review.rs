//! DELIBERATELY BROKEN — expected: a run-time panic, not a compiler error.
//! Run `cargo run --example 02-broken-review --features broken` and read the
//! panic.
//!
//! This file compiles. Five separate Phase 1 mistakes live in it anyway —
//! the panic below only points at one of them. Find the other four by
//! reading, not by running: the description is in `## Repair`, not here.

fn greeting_for(name: &String) -> String {
    let copy = name.clone();
    format!("خوش آمدی, {copy}!")
}

fn find_member(id: u32) -> i32 {
    match id {
        1 => 100,
        2 => 250,
        _ => -1,
    }
}

fn first_three_letters(word: &str) -> &str {
    &word[0..3]
}

fn report(balance: i32) {
    match balance {
        0 => println!("balance is zero"),
        _ => {}
    }
}

fn main() {
    let name = String::from("سن‌پای");
    println!("{}", greeting_for(&name));

    let points = find_member(3);
    println!("member points: {points}");

    report(0);

    println!("{}", first_three_letters("سلام"));
}
