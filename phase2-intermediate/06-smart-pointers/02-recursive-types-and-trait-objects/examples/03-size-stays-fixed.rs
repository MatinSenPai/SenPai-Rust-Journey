//! Proof, not just an assertion: build a 1000-level-deep `Expr` and show its
//! own, direct size on the stack never moved from the one-node size — every
//! extra level lives behind one more heap allocation, not inline.
//!
//!     cargo run -p p2-06-02-recursive-types-and-trait-objects --example 03-size-stays-fixed

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
    let shallow = Expr::Num(1.0);

    let mut deep = Expr::Num(0.0);
    let mut depth = 1;
    for _ in 0..1_000 {
        deep = Expr::Add(Box::new(deep), Box::new(Expr::Num(1.0)));
        depth += 1;
    }

    println!(
        "size_of::<Expr>()     = {} bytes",
        std::mem::size_of::<Expr>()
    );
    println!(
        "size_of_val(&shallow) = {} bytes",
        std::mem::size_of_val(&shallow)
    );
    println!(
        "size_of_val(&deep)    = {} bytes",
        std::mem::size_of_val(&deep)
    );
    println!("deep tree depth       = {depth}");
    println!("deep tree evaluates to = {}", eval(&deep));
}
