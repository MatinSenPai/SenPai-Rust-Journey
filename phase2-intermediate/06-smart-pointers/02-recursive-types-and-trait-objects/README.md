# 2.6.2 — Recursive types and boxed trait objects

## At a glance

After this lesson you can:

- Explain why a naive recursive type — an `enum` where one variant holds the very same `enum` directly, by value — refuses to compile, and read `E0072` for exactly what the compiler means, not from a memorized one-liner.
- Build a genuinely useful recursive structure — a small arithmetic expression tree — using `Box`, and say why boxing only one of two recursive fields is not enough.
- Recognize `Vec<Box<dyn Trait>>` as the same "fixed size, unknown content" trick applied a second time — this time for different types behind a shared trait, not for a type containing itself.

**Time:** ~65 minutes · **Prerequisites:**
[2.6.1 — `Box` and heap allocation](../01-box-and-heap-allocation/README.md),
[2.3.7 — Static versus dynamic dispatch, and object safety](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md)

---

## Why this matters

Trees, linked lists, ASTs (the syntax trees a parser builds), even a folder structure — they all share one shape: "this thing is either a leaf, or built out of a few smaller copies of the same thing." In Python you write that without thinking twice:

```python
class Node:
    def __init__(self, value, nxt=None):
        self.value = value
        self.next = nxt
```

`self.next` can be another `Node`, whose `next` can be another `Node`, as deep as you like — and Python never complains. Not because Python is smarter, but because every Python variable, always, without exception, is just a reference to an object — one machine word, no matter how big that object is. The interpreter never has to know up front "exactly how many bytes does a `Node` take," because the answer is always the same: the size of a pointer.

Rust does not take that deal. A value, by default, sits exactly where it's held — its actual bytes, not a pointer to its bytes — whether that's on the stack or inside another struct. That one choice is what lets an `i32` sit packed edge-to-edge inside a `Vec<i32>` with zero extra indirection, and it's a big part of why Rust code usually runs with no hidden cost. But it comes with a price: for every type you write, the compiler must be able to hand back one fixed, final number — "this type is exactly this many bytes" — before the program ever runs. Today you see exactly where that requirement hits a wall.

[2.6.1](../01-box-and-heap-allocation/README.md) handed you one central fact — `Box<T>`, no matter what `T` is, is always exactly one pointer wide — and used it to close out the `Box<dyn Trait>` story completely. It left exactly one door still shut: a type that holds itself. Today you open it, with that same one fact doing all the work, and then apply the already-resolved boxed-trait-object story to a real, concrete case — a collection of several different shapes behind one interface. This is lesson two of five in the smart-pointers module: 2.6.1 built the tool, today you see its first real payoff.

---

## The concept

### A type that claims to hold itself

Picture a small arithmetic expression: it's either a number, or the sum of two smaller expressions, or the product of two smaller expressions — exactly the "leaf, or a few smaller copies of itself" shape from above. The direct translation of that sentence into an `enum` is this:

```rust
enum Expr {
    Num(f64),
    Add(Expr, Expr),
    Mul(Expr, Expr),
}
```

Compile it, and you get this:

```text
error[E0072]: recursive type `Expr` has infinite size
  --> phase2-intermediate\06-smart-pointers\02-recursive-types-and-trait-objects\examples\01-naive-expr-broken.rs:8:1
   |
 8 | enum Expr {
   | ^^^^^^^^^
 9 |     Num(f64),
10 |     Add(Expr, Expr),
   |         ---- recursive without indirection
   |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
   |
10 |     Add(Box<Expr>, Expr),
   |         ++++    +

For more information about this error, try `rustc --explain E0072`.
```

Why? To lay `Expr` out on the stack, the compiler has to size its largest variant with one fixed number. The `Add` variant holds two whole `Expr` values, by value — not pointers to them, the values themselves. So to size `Add`, you first need the size of `Expr` — the exact thing you're in the middle of computing. This isn't a computation that eventually bottoms out and returns an answer; it's a circular definition with no finite number that satisfies it. That's why the compiler says "infinite size," not "unknown size" — those are two genuinely different claims, and this lesson gets to the difference later.

```senpai-visual
{"kind":"concept","labels":["Add holds two Expr fields, by value","each of those may itself be an Add","size(Expr) needs size(Expr) first","the count never bottoms out","box the recursive fields instead","Box is always one pointer wide"]}
```

### `Box` breaks the cycle

The fix is exactly what [2.6.1](../01-box-and-heap-allocation/README.md) already taught you: put the recursive field behind a `Box`.

```rust
enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}
```

That one change is enough, because the `Add` variant no longer holds two whole `Expr` values — it holds two `Box<Expr>` values, and a `Box<Expr>`, no matter what's inside it or how deep it goes, is *always* exactly one pointer wide (8 bytes on a 64-bit machine). The math stops being circular: `size(Add)` = one pointer + one pointer, and you already know both of those numbers without first needing `size(Expr)`. The real recursion didn't go away — it moved. It moved out of the type definition (which has to be bounded at compile time) and into the heap (which can go as deep as it likes at run time, at the cost of one allocation per level).

Now build a real expression — `(2 + 3) * 4` — and evaluate it:

```rust
fn eval(expr: &Expr) -> f64 {
    match expr {
        Expr::Num(value) => *value,
        Expr::Add(left, right) => eval(left) + eval(right),
        Expr::Mul(left, right) => eval(left) * eval(right),
    }
}
```

One subtlety worth naming: `expr` is a `&Expr`, so inside the `match` arms, `left` and `right` come out as `&Box<Expr>`, not `Box<Expr>` — Rust works that out for you, nothing you have to write by hand. And calling `eval(left)` works with no manual dereferencing at all: `Box<Expr>` implements `Deref`, so a `&Box<Expr>` slots in wherever a `&Expr` is expected — the same deref coercion [2.6.1](../01-box-and-heap-allocation/README.md) introduced.

```rust
// (2 + 3) * 4
let tree = Expr::Mul(
    Box::new(Expr::Add(
        Box::new(Expr::Num(2.0)),
        Box::new(Expr::Num(3.0)),
    )),
    Box::new(Expr::Num(4.0)),
);
println!("(2 + 3) * 4 = {}", eval(&tree));
```

```text
(2 + 3) * 4 = 20
```

```senpai-visual
{"kind":"ownership","labels":["Mul owns two Box<Expr> fields","the left Box points to an Add node on the heap","the right Box points to Num(4.0) on the heap","that Add node owns two more boxes","every Box is the same one-pointer size","dropping Mul drops the whole chain"]}
```

### Proof: the size never moves

Back the claim with a number, not just an argument. Build a one-node `Expr`, then a 1000-level chain, and compare the size of each value itself — not what its pointers lead to, the value that actually sits on the stack:

```rust
let shallow = Expr::Num(1.0);

let mut deep = Expr::Num(0.0);
let mut depth = 1;
for _ in 0..1_000 {
    deep = Expr::Add(Box::new(deep), Box::new(Expr::Num(1.0)));
    depth += 1;
}
```

```rust
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
```

```text
size_of::<Expr>()     = 24 bytes
size_of_val(&shallow) = 24 bytes
size_of_val(&deep)    = 24 bytes
deep tree depth       = 1001
deep tree evaluates to = 1000
```

`shallow` and the root of `deep` are exactly the same size, even though one has nothing underneath it and the other has 1000 more levels underneath it. That's the whole content of "fixed size": `Expr` itself, wherever it sits, is always the same number of bytes; the tree's real depth is a run-time property that lives behind the pointers, not a property of the type.

One thing worth noticing: that `24` is exactly two `Box`es (`8 + 8`) plus 8 more bytes for an explicit tag saying which of the three variants this is — `Num`, `Add`, or `Mul`. [2.6.1](../01-box-and-heap-allocation/README.md) and [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md) showed you a case where a tag comes *free*: `Option<Box<T>>` needs no separate byte at all, because a `Box` can never be null, so the compiler borrows that one impossible bit pattern as the `None` tag. That trick only works for choosing between exactly two things, though — packing a *three*-way choice into spare bit patterns is a harder ask, and here the compiler doesn't find one; it pays for an honest tag instead. `Expr` still comes in at a fixed size either way — the fixed-size guarantee never depended on the tag being free, only on `Box` always being one pointer wide.

### Half-boxing is still infinite

Look again at the compiler's own suggestion — it only boxed the first field of `Add` (`Box<Expr>, Expr`), not both. Apply exactly that suggestion, no more:

```rust
enum Expr {
    Num(f64),
    Add(Box<Expr>, Expr),
}
```

```text
error[E0072]: recursive type `Expr` has infinite size
  --> phase2-intermediate\06-smart-pointers\02-recursive-types-and-trait-objects\examples\04-half-boxed-broken.rs:9:1
   |
 9 | enum Expr {
   | ^^^^^^^^^
10 |     Num(f64),
11 |     Add(Box<Expr>, Expr),
   |                    ---- recursive without indirection
   |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
   |
11 |     Add(Box<Expr>, Box<Expr>),
   |                    ++++    +

For more information about this error, try `rustc --explain E0072`.
```

Still `E0072`, now pointing at the second field. Which makes complete sense: boxing the first field took *that* field's path out of the recursive cycle, but the second field — a bare `Expr` — still sits directly inside `Add`, by value. The math is still just as circular, only through the other door: `size(Add)` = `size(Box<Expr>)` + `size(Expr)`, and that second term is still the very thing you're computing. **The rule isn't "box a field so the size shrinks"; it's "every recursive path has to pass through a box, all of them, not just one."**

The compiler's suggestion is a hint, not a guaranteed complete fix — recompile after applying it and check whether another error is waiting. (And while we're here: the compiler's help text also names `Rc` and `&` as possible fixes. `&` doesn't work here because a reference doesn't own what it points to — something else has to keep that data alive for the whole life of the reference, and a tree you just built has nowhere else for that to live. `Rc` is a different kind of box — one that allows more than one owner — and it's [2.6.3](../03-rc-and-arc/README.md)'s subject.)

### Boxed trait objects: the same idea, elsewhere

You already hit this exact wall in [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md), from a different angle: `dyn Shape` also has no size the compiler can know at compile time — not because it's recursive, but because *the real concrete type underneath it could be anything*, and different concrete types have different sizes. This is exactly what [1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) called an **unsized type** — the same family `str` and `[T]` belong to. Two genuinely different problems — "might be infinitely deep" versus "might be any type at all" — and both get the same fix, because from the compiler's point of view they're the same shape: "I cannot say up front how much room this needs." `Box` answers that with "then let it always be exactly one pointer" — whichever reason the size was unknown for.

This time it isn't about recursion — a `Circle` never holds itself — it's that you want several genuinely different concrete types sitting side by side in one `Vec`:

```rust
trait Shape {
    fn area(&self) -> f64;
}

struct Circle {
    radius: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
}
```

```rust
struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
}
```

```rust
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle { radius: 2.0 }),
    Box::new(Rectangle {
        width: 3.0,
        height: 4.0,
    }),
];

let total: f64 = shapes.iter().map(|shape| shape.area()).sum();
println!("total area = {total:.2}");
```

```text
total area = 24.57
```

`Circle` and `Rectangle` are two completely different structs, two different sizes. `Box<dyn Shape>` turns both into the same element type — a fat pointer, always the same size — exactly what [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) already showed you with `Summarize`. There's nothing new here about dynamic dispatch or vtables — that story is fully told there. Today's point is just this: same `Box`, same reason — "turn an unknown size into a fixed one" — applied to different types this time, instead of unknown depth.

```senpai-visual
{"kind":"ownership","labels":["Vec owns a list of Box<dyn Shape>","every Box is the same one-pointer size","one Box points to a heap Circle","the next Box points to a heap Rectangle","same element type, different real sizes underneath"]}
```

---

## Hands on

```sh
cargo run -p p2-06-02-recursive-types-and-trait-objects --example 02-boxed-expr-eval
cargo run -p p2-06-02-recursive-types-and-trait-objects --example 03-size-stays-fixed
cargo run -p p2-06-02-recursive-types-and-trait-objects --example 05-vec-box-dyn-shape
```

Then the two broken ones:

```sh
cargo run -p p2-06-02-recursive-types-and-trait-objects --example 01-naive-expr-broken --features broken
cargo run -p p2-06-02-recursive-types-and-trait-objects --example 04-half-boxed-broken --features broken
```

Then try these:

1. In `03-size-stays-fixed.rs`, change `1_000` to `3_000`. Does `size_of::<Expr>()` change? What about `deep tree depth`?
2. In `05-vec-box-dyn-shape.rs`, add a third `Circle` to `shapes`. Does `total` come out right without changing anything else in the file?
3. In `02-boxed-expr-eval.rs`, swap `Mul` for `Add` and see what `eval` returns for `(2 + 3) + 4`.

---

## Errors you will meet

### `E0072` — recursive type has infinite size

```text
error[E0072]: recursive type `Expr` has infinite size
  --> phase2-intermediate\06-smart-pointers\02-recursive-types-and-trait-objects\examples\01-naive-expr-broken.rs:8:1
   |
 8 | enum Expr {
   | ^^^^^^^^^
 9 |     Num(f64),
10 |     Add(Expr, Expr),
   |         ---- recursive without indirection
   |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
   |
10 |     Add(Box<Expr>, Expr),
   |         ++++    +

For more information about this error, try `rustc --explain E0072`.
```

**What the compiler is objecting to:** to lay `Expr` out on the stack, it has to size the largest variant with one fixed number. The `Add` variant holds two whole `Expr` values by value — not pointers to them; sizing each one needs the size of `Expr` again — the exact thing you haven't finished computing yet. That calculation has no final number.

**The fix:** put every recursive field behind a `Box`:

```rust
enum Expr {
    Num(f64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}
```

**Why this is the fix:** a `Box<Expr>`, no matter what's inside it or how deep it goes, is always exactly one pointer wide. Now `Add`'s size depends on two numbers you already know, not on the size of `Expr` itself. The real recursion moved out of the type definition (which has to be bounded at compile time) and into the heap (which can go as deep as it likes at run time).

### `E0072` — again, when only one field is boxed

```text
error[E0072]: recursive type `Expr` has infinite size
  --> phase2-intermediate\06-smart-pointers\02-recursive-types-and-trait-objects\examples\04-half-boxed-broken.rs:9:1
   |
 9 | enum Expr {
   | ^^^^^^^^^
10 |     Num(f64),
11 |     Add(Box<Expr>, Expr),
   |                    ---- recursive without indirection
   |
help: insert some indirection (e.g., a `Box`, `Rc`, or `&`) to break the cycle
   |
11 |     Add(Box<Expr>, Box<Expr>),
   |                    ++++    +

For more information about this error, try `rustc --explain E0072`.
```

**What the compiler is objecting to:** the same error, the same reason — just underlining the second field this time, not the first. The first field (`Box<Expr>`) is no longer a problem; the second field, a bare `Expr`, still sits directly inside `Add` by value, and that alone is enough to keep the calculation circular.

**The fix:** box the remaining field too — exactly what "`Box` breaks the cycle" above already did.

**Why this is the fix:** the rule isn't "shrink `Add`'s size"; it's that *every* recursive path has to pass through a `Box`. One remaining raw field, whatever else next to it is already boxed, is enough on its own to keep the size at "infinite."

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
enum Number {
    Value(i32),
    Wrapped(Number),
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0072`, for exactly the same reason as `Expr`. The `Wrapped` variant holds a `Number` by value; sizing it needs the size of `Number` again.

</details>

<details>
<summary>What about this one?</summary>

```rust
enum Number {
    Value(i32),
    Wrapped(Box<Number>),
}
```

</details>

<details>
<summary>Answer</summary>

Yes. `Box<Number>` is always exactly one pointer wide, so `Number`'s size no longer depends on itself.

</details>

<details>
<summary>What does this print?</summary>

```rust
let tree = Expr::Add(Box::new(Expr::Num(10.0)), Box::new(Expr::Num(5.0)));
println!("{}", eval(&tree));
```

</details>

<details>
<summary>Answer</summary>

```text
15
```

`eval` adds the two `Num` values: `10.0 + 5.0`. (Notice `f64` prints with `{}` with no trailing `.0` when the result is a whole number — the same thing `02-boxed-expr-eval.rs` showed you with `20`.)

</details>

<details>
<summary>Does this compile? If so, what does <code>None</code> mean here?</summary>

```rust
struct Node {
    value: i32,
    next: Option<Box<Node>>,
}
```

</details>

<details>
<summary>Answer</summary>

Yes. `next: Option<Box<Node>>` means "maybe there's a next node, maybe not" — `None` means this is the last node. And thanks to the null-pointer optimization [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md) showed you, `Option<Box<Node>>` takes no extra bytes over a bare `Box<Node>`.

</details>

Between `Vec<Box<dyn Shape>>` and `Vec<dyn Shape>`, one compiles and one doesn't. Which is which, and why, in one sentence?

<details>
<summary>Answer</summary>

`Vec<Box<dyn Shape>>` compiles. `dyn Shape` alone is an unsized type — different concrete types like `Circle` and `Rectangle` have different sizes — and a `Vec` has to know each element's size up front; `Box` gives it that fixed size.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/01-naive-expr-broken.rs` so it compiles — box both of `Add` and `Mul`'s recursive fields.
2. Fix `examples/04-half-boxed-broken.rs` too. This one already has one field boxed — find the remaining field and box it as well. Afterward, build a small tree and `eval` it to confirm it actually works.

### Implement

Four functions in `src/lib.rs`:

```sh
cargo test -p p2-06-02-recursive-types-and-trait-objects
```

- `depth` — how many levels deep an `Expr` goes.
- `count_nums` — how many `Num` leaves are in the tree.
- `total_area` — the combined area of every shape in a `&[Box<dyn Shape>]`.
- `count_shapes_over` — how many shapes have an area above a threshold.

`Expr`, `eval`, `Shape`, `Circle`, and `Rectangle` are already fully written — the exact things you built above. The precise specification for each of the four functions, with examples, is in the doc comment above each one.

### Build

Pick one of these (or both):

- Add a new variant to `Expr`, such as `Neg(Box<Expr>)` for negating an expression. Update `eval` (and `depth`/`count_nums`, if you wrote them) to handle it, and show in a test or a small `fn main` that `Neg(Num(5.0))` evaluates to `-5.0`.
- Add a third `Shape`, such as `Triangle`. Put it in a `Vec<Box<dyn Shape>>` alongside a `Circle` and a `Rectangle`, and confirm `total_area` still works correctly with no changes anywhere else.

### Challenge (optional)

**Part one.** Generalize `Expr` into a generic `enum` — something like `Tree<T> { Leaf(T), Node(Box<Tree<T>>, Box<Tree<T>>) }` — and write a `count_leaves` that counts the leaves regardless of what `T` is.

**Part two.** (This one reaches forward.) Imagine two different places in a large expression need to share *the exact same* sub-expression, rather than each keeping its own copy. `Box` cannot do that — a `Box` has exactly one owner. In a scratch file (not part of this lesson's tests), swap `Expr`'s `Box<Expr>` fields for `Rc<Expr>` and see exactly what changes about building and evaluating a tree, and what doesn't. The full story is [2.6.3](../03-rc-and-arc/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Recursive type | a type where a variant or field holds, directly or indirectly, the very same type | trees, linked lists, ASTs |
| `E0072` | the "infinite size" error; the compiler cannot compute one fixed number for the type | any recursive type with no `Box` |
| Expression tree | representing a math expression as a tree — numbers as leaves, operators as nodes | evaluation, parsing, small compilers |
| Boxed trait object (`Box<dyn Trait>`) | the same fixed-size trick, applied to an erased type instead of a recursive one | `Vec<Box<dyn Trait>>` for heterogeneous collections |

### What you now know

- Why an `enum` where a variant holds that same `enum` by value refuses to compile: the compiler needs one fixed number to lay it out on the stack, and this definition never resolves to one.
- Why boxing *every* recursive field — not just one of them — is what actually fixes it: a `Box<T>` is always exactly one pointer wide, no matter how deep what's underneath actually goes.
- `size_of::<Expr>()` is a property of the type itself, not of any particular value; a tree's real depth is a run-time property that lives behind the pointers.
- `Vec<Box<dyn Trait>>` applies the same trick to a different reason a size was unknown (the concrete type underneath could be anything) instead of recursion — heterogeneity, not self-reference.

### What comes back later

- **`Rc` and `Arc` for shared ownership** — for when one `Box` (one single owner) isn't enough and several parts of a program genuinely need to share the same sub-tree or node — [2.6.3 — `Rc` and `Arc`](../03-rc-and-arc/README.md).
- **`Weak` and reference cycles** — a tree where one node suddenly points back at its own parent stops being a tree and becomes a graph with a cycle; `Box` alone doesn't handle that without leaking memory forever — [2.6.4 — `Weak` and reference cycles](../04-weak-and-reference-cycles/README.md).
- **`RefCell` and interior mutability** — for when you need to mutate a shared node, not just read it — [2.6.5 — `RefCell` and interior mutability](../05-refcell-and-interior-mutability/README.md).

### Can you explain?

- Why does `enum Expr { Num(f64), Add(Expr, Expr) }` have infinite size, while the same thing with `Add(Box<Expr>, Box<Expr>)` has a fixed size?
- Why isn't boxing just one of `Add`'s two fields enough? Answer in your own words, not by repeating the compiler's message.
- Why is `size_of::<Expr>()` for a 1000-level tree the same number as for a single `Num`?
- `dyn Shape` and an unboxed `Expr` both cause a size-related error — but not for the same reason. Explain each in one sentence.
- The compiler suggests `&` as a fix for `E0072`, alongside `Box`. Why doesn't `&` actually work here?

---

## Going further

- [The Rust Book, ch. 15.1 — Using `Box<T>` to Point to Data on the Heap](https://doc.rust-lang.org/book/ch15-01-box.html) — this same subject, with this same classic recursive-list example, from the Rust team itself.
- [The Rustonomicon — Exotic Sizes](https://doc.rust-lang.org/nomicon/exotic-sizes.html) — unsized types and zero-sized types, in more depth than you needed today.
- [The Rustonomicon — layout optimizations](https://doc.rust-lang.org/nomicon/repr-rust.html) — the same null-pointer optimization [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md) introduced, this time with the technical detail.
- [`std::boxed::Box`](https://doc.rust-lang.org/std/boxed/struct.Box.html) — the official documentation for the one tool you reached for twice today.
