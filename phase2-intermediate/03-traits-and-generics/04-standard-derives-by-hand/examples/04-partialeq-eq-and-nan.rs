//! `#[derive(PartialEq)]` compares every field with `==`. `Eq` adds nothing
//! callable — it is a marker saying "and also, every value equals itself."
//! `f64` cannot make that promise, which is why it has `PartialEq` and never
//! `Eq`.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 04-partialeq-eq-and-nan

#[derive(Debug, PartialEq, Eq)]
struct Anime {
    title: String,
    episodes: u32,
    score: u8,
}

fn main() {
    let a = Anime {
        title: "Frieren".to_string(),
        episodes: 28,
        score: 96,
    };
    let b = Anime {
        title: "Frieren".to_string(),
        episodes: 28,
        score: 96,
    };
    let c = Anime {
        title: "Frieren".to_string(),
        episodes: 28,
        score: 95,
    };
    println!("a == b (every field matches): {}", a == b);
    println!("a == c (score differs):       {}", a == c);

    let nan = f64::NAN;
    println!("f64::NAN == f64::NAN:         {}", nan == nan);
    println!("1.0_f64 == 1.0_f64:           {}", 1.0_f64 == 1.0_f64);
}
