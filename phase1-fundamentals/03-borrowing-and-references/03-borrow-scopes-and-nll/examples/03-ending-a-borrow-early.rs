//! 1.3.3 — three ways to stop holding a borrow, and what each one is worth.

fn main() {
    // 1. An explicit block. The borrow cannot survive the closing brace.
    let mut totals = vec![10, 20, 30];
    let counted = {
        let view = &totals;
        view.len()
    };
    totals.push(40);
    println!(
        "with a block: counted {counted}, {} items now",
        totals.len()
    );

    // 2. The same code without the block. It compiles too — the borrow ended
    //    at `view.len()` either way, so the braces bought nothing here.
    let mut totals = vec![10, 20, 30];
    let view = &totals;
    let counted = view.len();
    totals.push(40);
    println!(
        "without one:  counted {counted}, {} items now",
        totals.len()
    );

    // 3. No borrow held at all. `i32` is `Copy`, so take the value and there
    //    is nothing left to end.
    let mut scores = vec![90, 80];
    let first = scores[0];
    scores.push(first);
    println!("no borrow:    {scores:?}");
}
