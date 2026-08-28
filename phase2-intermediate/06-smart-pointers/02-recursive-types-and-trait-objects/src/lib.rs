//! Exercises for 2.6.2 — recursive types and boxed trait objects.
//!
//! `Expr`, `eval`, `Shape`, `Circle`, and `Rectangle` are given below, fully
//! written — building them is exactly what the lesson body already walked
//! through. What you write here is new code that walks the same two shapes
//! for a different reason: how deep a tree really is, and how many shapes
//! clear a threshold.

/// A tiny arithmetic expression: a number, or the sum/product of two
/// smaller expressions. Every recursive field is boxed — see the README for
/// why `Add(Expr, Expr)` (no `Box`) refuses to compile.
pub enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

/// Evaluates an expression tree to its numeric result.
pub fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Num(value) => *value,
        Expr::Add(left, right) => eval(left) + eval(right),
        Expr::Mul(left, right) => eval(left) * eval(right),
    }
}

/// The number of levels from the root to the deepest leaf. A bare `Num` has
/// depth `1`. An `Add` or `Mul` has depth `1 +` the larger of its two
/// children's depths.
///
/// # Examples
///
/// `depth(&Expr::Num(5.0))` is `1`.
///
/// `depth(&Expr::Add(Box::new(Expr::Num(1.0)), Box::new(Expr::Num(2.0))))`
/// is `2`.
pub fn depth(expr: &Expr) -> usize {
    todo!("1 for a Num; for Add/Mul, 1 plus whichever child is deeper")
}

/// The total count of `Num` leaves anywhere in the tree.
///
/// # Examples
///
/// `count_nums(&Expr::Num(5.0))` is `1`.
///
/// For `Expr::Mul(Box::new(Expr::Add(Box::new(Expr::Num(2.0)),
/// Box::new(Expr::Num(3.0)))), Box::new(Expr::Num(4.0)))` — that is,
/// `(2 + 3) * 4` — this is `3`.
pub fn count_nums(expr: &Expr) -> usize {
    todo!("1 for a Num; for Add/Mul, the total of both children's counts")
}

/// Something with a computable area.
pub trait Shape {
    fn area(&self) -> f64;
}

pub struct Circle {
    pub radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}

pub struct Rectangle {
    pub width: f64,
    pub height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}

/// The combined area of every shape in `shapes`, added together. `0.0` for
/// an empty slice.
pub fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    todo!("add together the area of every shape in the slice")
}

/// How many shapes in `shapes` have an area strictly greater than
/// `threshold` — a shape whose area exactly equals `threshold` does not
/// count.
pub fn count_shapes_over(shapes: &[Box<dyn Shape>], threshold: f64) -> usize {
    todo!("count the shapes whose area is strictly greater than threshold")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    // ---------------------------------------------------------- depth

    #[test]
    fn a_bare_num_has_depth_one() {
        assert_eq!(depth(&Expr::Num(5.0)), 1);
    }

    #[test]
    fn one_level_of_add_has_depth_two() {
        let tree = Expr::Add(Box::new(Expr::Num(1.0)), Box::new(Expr::Num(2.0)));
        assert_eq!(depth(&tree), 2);
    }

    #[test]
    fn depth_follows_the_deeper_branch() {
        // (2 + 3) * 4 — the Add branch is deeper than the lone Num.
        let tree = Expr::Mul(
            Box::new(Expr::Add(
                Box::new(Expr::Num(2.0)),
                Box::new(Expr::Num(3.0)),
            )),
            Box::new(Expr::Num(4.0)),
        );
        assert_eq!(depth(&tree), 3);
    }

    #[test]
    fn depth_grows_one_level_per_nesting() {
        let mut tree = Expr::Num(0.0);
        for level in 1..=6 {
            tree = Expr::Add(Box::new(tree), Box::new(Expr::Num(1.0)));
            assert_eq!(depth(&tree), level + 1);
        }
    }

    // ------------------------------------------------------ count_nums

    #[test]
    fn a_bare_num_counts_as_one() {
        assert_eq!(count_nums(&Expr::Num(5.0)), 1);
    }

    #[test]
    fn count_nums_adds_up_every_leaf() {
        // (2 + 3) * 4 has three Num leaves: 2, 3, and 4.
        let tree = Expr::Mul(
            Box::new(Expr::Add(
                Box::new(Expr::Num(2.0)),
                Box::new(Expr::Num(3.0)),
            )),
            Box::new(Expr::Num(4.0)),
        );
        assert_eq!(count_nums(&tree), 3);
    }

    // ------------------------------------------------------- total_area

    #[test]
    fn total_area_of_empty_slice_is_zero() {
        let shapes: Vec<Box<dyn Shape>> = Vec::new();
        assert_eq!(total_area(&shapes), 0.0);
    }

    #[test]
    fn total_area_sums_two_rectangles_exactly() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Rectangle {
                width: 2.0,
                height: 3.0,
            }),
            Box::new(Rectangle {
                width: 4.0,
                height: 5.0,
            }),
        ];
        assert_eq!(total_area(&shapes), 26.0);
    }

    #[test]
    fn total_area_mixes_concrete_types_behind_one_vec() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Circle { radius: 1.0 }),
            Box::new(Rectangle {
                width: 1.0,
                height: 1.0,
            }),
        ];
        let expected = std::f64::consts::PI + 1.0;
        assert!(approx_eq(total_area(&shapes), expected));
    }

    // ------------------------------------------------- count_shapes_over

    #[test]
    fn count_shapes_over_counts_only_the_larger_ones() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Circle { radius: 1.0 }), // area ~3.14
            Box::new(Rectangle {
                width: 2.0,
                height: 3.0,
            }), // area 6.0
            Box::new(Rectangle {
                width: 1.0,
                height: 1.0,
            }), // area 1.0
        ];
        assert_eq!(count_shapes_over(&shapes, 2.0), 2);
        assert_eq!(count_shapes_over(&shapes, 100.0), 0);
    }

    #[test]
    fn count_shapes_over_excludes_an_exact_match() {
        let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Rectangle {
            width: 2.0,
            height: 3.0,
        })];
        assert_eq!(count_shapes_over(&shapes, 6.0), 0);
        assert_eq!(count_shapes_over(&shapes, 5.999), 1);
    }
}
