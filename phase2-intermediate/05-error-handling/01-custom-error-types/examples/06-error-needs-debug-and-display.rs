//! DELIBERATELY BROKEN — expected: E0277.
//!
//!     cargo run -p p2-05-01-custom-error-types --example 06-error-needs-debug-and-display --features broken

pub struct ReviewError;

// `Error: Debug + Display` — neither is implemented for `ReviewError` above,
// so this line alone cannot compile.
impl std::error::Error for ReviewError {}

fn main() {
    let _err = ReviewError;
}
