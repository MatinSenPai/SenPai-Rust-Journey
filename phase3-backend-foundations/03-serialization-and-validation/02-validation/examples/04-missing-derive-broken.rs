//! DELIBERATELY BROKEN — expected: E0599
//! Run `cargo run -p p3-03-02-validation --example 04-missing-derive-broken --features broken`
//! and read the error. `Rating` never derives `Validate`.

use validator::Validate;

struct Rating {
    score: u8,
}

fn main() {
    let r = Rating { score: 15 };
    println!("{:?}", r.validate());
}
