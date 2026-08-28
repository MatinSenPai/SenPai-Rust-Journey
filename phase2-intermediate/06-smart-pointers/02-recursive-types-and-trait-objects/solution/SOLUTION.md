# Solution — 2.6.2 Recursive types and boxed trait objects

```rust
pub fn depth(expr: &Expr) -> usize {
    match expr {
        Expr::Num(_) => 1,
        Expr::Add(left, right) | Expr::Mul(left, right) => 1 + depth(left).max(depth(right)),
    }
}

pub fn count_nums(expr: &Expr) -> usize {
    match expr {
        Expr::Num(_) => 1,
        Expr::Add(left, right) | Expr::Mul(left, right) => count_nums(left) + count_nums(right),
    }
}

pub fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|shape| shape.area()).sum()
}

pub fn count_shapes_over(shapes: &[Box<dyn Shape>], threshold: f64) -> usize {
    shapes
        .iter()
        .filter(|shape| shape.area() > threshold)
        .count()
}
```

## `depth` and `count_nums` — one shared recursive shape

Both functions follow exactly the same shape as `eval`, already given at the top of this file: a base case for `Num`, and a step that combines two children. The only difference is that `eval` has to tell `Add` and `Mul` apart (because `+` and `*` give different answers), while `depth` and `count_nums` don't care which one it is — a node's depth and the leaf count underneath it are the same whether the node is an addition or a multiplication. That's why both functions use an alternative pattern (`Add(left, right) | Mul(left, right)`): one arm covers both variants, because their bodies are identical.

`left` and `right` come out as `&Box<Expr>` here (since `expr` itself is a `&Expr`), and the recursive call `depth(left)` works with no manual dereferencing at all — the same deref coercion the lesson body already showed you on `eval`.

## `total_area` and `count_shapes_over` — again, an almost-identical body

```rust
shapes.iter().map(|shape| shape.area()).sum()
```

```rust
shapes.iter().filter(|shape| shape.area() > threshold).count()
```

Both are a plain iterator pipeline over the same `&[Box<dyn Shape>]` — one sums, the other filters and counts. Worth noticing: `.area()` gets resolved through *two* layers — first `&Box<dyn Shape>` down to `Box<dyn Shape>` (what `.iter()` yields), then `Box<dyn Shape>` down to `dyn Shape` (the `Box` itself) — and neither layer was unwrapped by hand. That's exactly the point "Boxed trait objects" made in the lesson: `Box` here is only about the fixed size, nothing new about dynamic dispatch itself — that story was already told in full in [2.3.7](../../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md).

## What this lesson was really about

- **Recursion at compile time has to bottom out at one final number; `Box` moves that requirement to run time instead of removing it.** `depth` and `count_nums` prove that plain *algorithmic* recursion (a function calling itself) was never the problem — the problem was only ever in the *type* definition.
- **Every recursive path has to pass through a `Box`, not just one of them.** The alternative pattern `Add(left, right) | Mul(left, right)` only compiles cleanly because both variants genuinely share the same shape — two `Box<Expr>` fields.
- **`Box<dyn Trait>` applies the same trick to a different reason a size was unknown.** `total_area` and `count_shapes_over` have no extra code asking "is this shape a `Circle` or a `Rectangle`?" — exactly the indifference dynamic dispatch promises.
