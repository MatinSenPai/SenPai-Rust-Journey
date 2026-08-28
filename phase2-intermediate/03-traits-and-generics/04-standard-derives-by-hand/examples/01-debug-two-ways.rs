//! `#[derive(Debug)]` is not magic — it writes an `impl Debug` for you. This
//! example writes the same thing two more times: once with the
//! `f.debug_struct(...)` builder (what a derive actually expands to), and
//! once with a raw `write!()` call, to see exactly what the builder buys you.
//!
//!     cargo run -p p2-03-04-standard-derives-by-hand --example 01-debug-two-ways

use std::fmt;

#[derive(Debug)]
struct Derived {
    title: String,
    episodes: u32,
    score: u8,
}

struct Builder {
    title: String,
    episodes: u32,
    score: u8,
}
impl fmt::Debug for Builder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Builder")
            .field("title", &self.title)
            .field("episodes", &self.episodes)
            .field("score", &self.score)
            .finish()
    }
}

struct Raw {
    title: String,
    episodes: u32,
    score: u8,
}
impl fmt::Debug for Raw {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Raw {{ title: {:?}, episodes: {:?}, score: {:?} }}",
            self.title, self.episodes, self.score
        )
    }
}

fn main() {
    let derived = Derived {
        title: "Frieren".to_string(),
        episodes: 28,
        score: 96,
    };
    let builder = Builder {
        title: "Frieren".to_string(),
        episodes: 28,
        score: 96,
    };
    let raw = Raw {
        title: "Frieren".to_string(),
        episodes: 28,
        score: 96,
    };

    println!("compact — derived: {derived:?}");
    println!("compact — builder: {builder:?}");
    println!("compact — raw:     {raw:?}");
    println!();
    println!("alternate — derived:\n{derived:#?}");
    println!("alternate — builder:\n{builder:#?}");
    println!("alternate — raw:\n{raw:#?}");
}
