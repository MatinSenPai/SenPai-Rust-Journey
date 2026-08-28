//! DELIBERATELY BROKEN — expected: E0072
//!
//! Boxing only one side looks like progress, but it is not enough: the
//! other side is still a bare `Expr`, and that one path alone still
//! recurses forever.
//!
//!     cargo run -p p2-06-02-recursive-types-and-trait-objects --example 04-half-boxed-broken --features broken

enum Expr {
    Num(f64),
    Add(Box<Expr>, Expr),
}

fn main() {
    let _tree = Expr::Add(Box::new(Expr::Num(2.0)), Expr::Num(3.0));
}
