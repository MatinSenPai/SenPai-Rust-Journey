//! DELIBERATELY BROKEN — expected: E0277.
//!
//! `dyn Summarize` alone has no fixed size — `AnimeSeries` and
//! `MangaVolume` are different sizes, and "some type implementing
//! `Summarize`" could be anything. A `Vec`'s backing buffer needs to know
//! each element's exact size up front.
//!
//!     cargo run -p p2-03-07-static-vs-dynamic-dispatch --example 06-vec-dyn-no-box-broken --features broken

trait Summarize {
    fn summary(&self) -> String;
}

fn total_summaries(_lineup: &Vec<dyn Summarize>) {}

fn main() {
    println!("{}", total_summaries as usize);
}
