//! `?` inside a function returning `anyhow::Result<T>` converts *any*
//! `std::error::Error` type into `anyhow::Error` automatically — no
//! `From` impl required anywhere, not even for a foreign type like
//! `std::num::ParseIntError` that this crate does not own.

fn parse_score(raw: &str) -> Result<u8, std::num::ParseIntError> {
    raw.trim().parse()
}

fn run() -> anyhow::Result<()> {
    let score = parse_score("87")?;
    println!("parsed score: {score}");
    let score = parse_score("oops")?; // ParseIntError -> anyhow::Error, no From needed
    println!("parsed score: {score}");
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("Display : {err}");
        println!("Debug   : {err:?}");
    }
}
