//! `biased;` makes `select!` poll branches in the order they're written,
//! instead of picking among already-ready branches at random.

#[tokio::main]
async fn main() {
    // Both branches are ready before select! even starts polling. Without
    // `biased;`, tokio picks among ready branches uniformly at random - run
    // this program a few times and the winner will not always be the same.
    tokio::select! {
        _ = std::future::ready(()) => println!("unbiased: branch A won"),
        _ = std::future::ready(()) => println!("unbiased: branch B won"),
    }

    // `biased;` as the first line removes the randomness: branches are tried
    // in written order, so branch A wins every single run.
    tokio::select! {
        biased;
        _ = std::future::ready(()) => println!("biased: branch A won"),
        _ = std::future::ready(()) => println!("biased: branch B won"),
    }
}
