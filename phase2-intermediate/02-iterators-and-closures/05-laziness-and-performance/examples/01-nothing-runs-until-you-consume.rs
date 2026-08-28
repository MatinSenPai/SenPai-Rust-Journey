//! Building an iterator chain does nothing by itself — the closures inside
//! `.map()` and `.filter()` do not run until something actually pulls
//! values through them.
//!
//!     cargo run -p p2-02-05-laziness-and-performance --example 01-nothing-runs-until-you-consume

fn main() {
    println!("building the chain...");
    let chain = vec![1, 2, 3, 4, 5]
        .into_iter()
        .map(|n| {
            println!("  map saw {n}");
            n * 2
        })
        .filter(|n| {
            println!("  filter saw {n}");
            n % 4 == 0
        });
    println!("chain built — nothing printed above this line from map/filter");

    println!("now calling .collect()...");
    let result: Vec<i32> = chain.collect();
    println!("result: {result:?}");
}
