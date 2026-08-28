//! `Box` breaks the cycle: every recursive field is a `Box<Expr>` instead of
//! a bare `Expr`, so the type itself has one fixed, known size no matter how
//! deep a real tree gets at run time.
//!
//!     cargo run -p p2-06-02-recursive-types-and-trait-objects --example 02-boxed-expr-eval

enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Num(value) => *value,
        Expr::Add(left, right) => eval(left) + eval(right),
        Expr::Mul(left, right) => eval(left) * eval(right),
    }
}

fn main() {
    // (2 + 3) * 4
    let tree = Expr::Mul(
        Box::new(Expr::Add(
            Box::new(Expr::Num(2.0)),
            Box::new(Expr::Num(3.0)),
        )),
        Box::new(Expr::Num(4.0)),
    );
    println!("(2 + 3) * 4 = {}", eval(&tree));
}
