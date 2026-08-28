//! DELIBERATELY BROKEN — expected: a thiserror macro error, no rustc code
//!
//! `#[from]` promises `impl From<FieldType> for RatingsError` — a function
//! that only ever receives one `FieldType` value and has to build the whole
//! variant out of it alone. `InvalidScore` also needs a `line` number that
//! no `ParseIntError` carries, so `#[from]` cannot honestly generate that
//! `From` impl.
//!
//!     cargo run -p p2-05-03-thiserror-and-anyhow --example 09-from-needs-single-field --features broken

#[derive(Debug, thiserror::Error)]
pub enum RatingsError {
    #[error("line {line}: invalid score: {source}")]
    InvalidScore {
        line: usize,
        #[from]
        source: std::num::ParseIntError,
    },
}

fn main() {}
