//! DELIBERATELY BROKEN — expected: E0072
//!
//! A direct translation of "an expression is a number, or the sum/product
//! of two smaller expressions" — each variant holds an `Expr` by value.
//!
//!     cargo run -p p2-06-02-recursive-types-and-trait-objects --example 01-naive-expr-broken --features broken

enum Expr {
    Num(f64),
    Add(Expr, Expr),
    Mul(Expr, Expr),
}

fn main() {
    let two = Expr::Num(2.0);
    let three = Expr::Num(3.0);
    let _sum = Expr::Add(two, three);
}
