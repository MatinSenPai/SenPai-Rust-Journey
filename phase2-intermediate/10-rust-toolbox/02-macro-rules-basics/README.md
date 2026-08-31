# 2.10.2 — `macro_rules!` basics

## At a glance

After this lesson you can:

- Write a matcher/transcriber pair with `macro_rules!`, use fragment specifiers (`expr`, `ident`, `ty`, `pat`, `literal`, `tt`) correctly, and predict exactly what tokens a given call expands to.
- Use repetition (`$(...)*`, `+`, `?`) to accept a variable number of arguments, and explain why macro hygiene keeps a macro's own `let` from colliding with a caller's identically-named variable.
- Read and fix the real compiler error a wrong fragment specifier produces, and decide when a macro is genuinely the right tool versus when a plain function or generic is enough.

**Time:** ~70 minutes · **Prerequisites:** [2.10.1 — Pattern matching in depth](../01-pattern-matching-depth/README.md)

---

## Why this matters

Ever since your first `println!`, you've been *calling* macros — the `!` was always the tell. This lesson is about writing your own, with `macro_rules!`, Rust's declarative macro system. Procedural macros — things like `#[derive(Serialize)]` — are a separate, heavier mechanism this course never goes near; that fact alone is enough to know for now.

Python has no real equivalent. A decorator or a metaclass transforms an *object at run time*; a macro transforms *source code at compile time* — it takes the tokens you wrote and expands them into new tokens before the compiler proper ever lays eyes on them.

The reason Rust needs this and Python doesn't comes down to one thing: Python functions take `*args`, and their input can be any type, so `max(a, b, c)` is genuinely just an ordinary function. Rust functions have fixed arity and fixed types — so anything variadic, like `println!`, `vec!`, and `format!`, *has to* be a macro. No function signature can accept "any number of arguments, of any type."

---

## The concept

### Matcher and transcriber

A `macro_rules!` definition is a list of arms, each shaped `(matcher) => { transcriber }`. The matcher is a pattern over *tokens* — exactly like `match`, only this time over source code; the transcriber is a template the captured tokens get pasted into:

```rust
macro_rules! square {
    ($x:expr) => {
        $x * $x
    };
}
```

`$x:expr` is a **fragment specifier**: it says "capture one full *expression*, and call it `$x`." A captured `expr` stays atomic in the expansion — it keeps whatever grouping it had where it was captured:

```rust
let a = square!(1 + 2);
println!("square!(1 + 2) = {a}");
```

```text
square!(1 + 2) = 9
```

`square!(1 + 2)` expands, conceptually, to `(1 + 2) * (1 + 2)`, not to `1 + 2 * 1 + 2` — otherwise the answer would be 5, not 9.

```senpai-visual
{"kind":"concept","labels":["your tokens: square!(1 + 2)","matcher: $x:expr","transcriber template: $x * $x","expansion: (1 + 2) * (1 + 2)","compiler compiles the expansion"]}
```

A detail few people catch the first time: because `$x` is pasted twice in the transcriber, whatever expression you pass gets evaluated twice — not once:

```rust
let mut calls = 0;
let mut next = || {
    calls += 1;
    calls
};
let b = square!(next());
println!("square!(next()) = {b}, next() ran {calls} times");
```

```text
square!(next()) = 2, next() ran 2 times
```

`next()` runs once for the first `$x` and once for the second; the first call returns 1, the second returns 2, and their product is 2 — not either number squared. For real work you need to bind `$x` into a `let` first, so it's only evaluated once; `timed!` in the exercises asks you to do exactly that.

### The other fragment specifiers

You've seen `expr`. The rest you'll actually reach for:

- `ident` — a name; for generating a fresh variable or function, somewhere `expr` won't let you.
- `ty` — a type.
- `pat` — a pattern; what 2.10.1 spent its time on — a `match` arm, or the left side of a `let`.
- `literal` — a raw literal.
- `tt` — a single "token tree": one token, or everything inside a matching pair of `()`/`[]`/`{}`. The escape hatch, for when no other specifier fits.

The important part: once a fragment is captured, it stays **opaque** — an `expr` can never later be dropped somewhere an `ident` or a `pat` is expected; the specifier decides what role the fragment can play, not what its actual contents look like. "Errors you will meet" shows you exactly this.

### Repetition

`$( ... )` followed by one of three markers — `*` (zero or more), `+` (one or more), or `?` (zero or one) — matches a group as many times as it repeats in the input. The same syntax replays the captured values in the transcriber:

```rust
macro_rules! string_vec {
    () => {
        Vec::<String>::new()
    };
    ( $( $s:expr ),+ $(,)? ) => {{
        let mut v = Vec::new();
        $( v.push($s.to_string()); )+
        v
    }};
}
```

The second arm says: one or more `expr`s, comma-separated (`$( $s:expr ),+`), plus an optional trailing comma (`$(,)?` — the same courtesy every built-in macro extends). Its transcriber is a block — double braces, the outer pair belonging to the arm and the inner pair to the block itself — that builds a `Vec` and, with `$( v.push($s.to_string()); )+`, pushes once per element:

```rust
let empty: Vec<String> = string_vec![];
let names = string_vec!["hero", "mage", "sage"];
let trailing = string_vec!["one", "two",];
println!("empty:    {empty:?}");
println!("names:    {names:?}");
println!("trailing: {trailing:?}");
```

```text
empty:    []
names:    ["hero", "mage", "sage"]
trailing: ["one", "two"]
```

Arms are always tried top to bottom — the first matcher that fits wins. Two arms were needed here, not one: if the only arm had been written `$( $s:expr ),*` (a star, not a plus), the empty case would still match, but its transcriber would still emit `let mut v = Vec::new();` — and `mut` on a variable that's never mutated on that path is a warning. Splitting the empty case into its own arm keeps both expansions warning-free.

An arm can also call the macro itself again — recursion, exactly the way a function can call itself. Like any recursion, it needs a **base case** that answers the smallest input and stops the recursion from continuing — "Errors you will meet" shows what happens when you forget it. When a macro you're defining calls itself (or another macro from the same crate) again inside its own transcriber, it's better to call it as `$crate::` rather than bare — that's up next, under hygiene.

### Hygiene, in one paragraph

Names introduced *inside* a macro's transcriber live in their own scope: if a macro's transcriber builds `let start = ...;`, and the caller already has a variable named `start`, they don't collide — as far as the compiler is concerned, the macro's `start` and the caller's `start` are two completely different names, even though they're spelled identically. This is called **hygiene**, and it's exactly what keeps Rust macros away from the classic `#define` disasters of C:

```rust
macro_rules! double_it {
    ($x:expr) => {{
        let start = $x;
        start + start
    }};
}
```

```rust
let start = 100;
let doubled = double_it!(7);
println!("caller's start: {start}");
println!("double_it!(7):  {doubled}");
```

```text
caller's start: 100
double_it!(7):  14
```

The caller of `double_it!(7)` never notices the transcriber has its own `start` — and its own `start` stays untouched. The flip side: a macro *can't* quietly define a variable for the caller to later reach — which is itself a feature, not a gap.

This hygiene only covers local names — paths and items still resolve at the call site. That's exactly why `$crate` exists: inside a transcriber, it always names the crate the macro was *defined* in, no matter which crate is calling it — the exercise skeleton shows this exact pattern in every `$crate::unsolved(...)` call.

### When *not* to write a macro (honest guidance)

Reach for a macro **last**. A plain function is checked as written, shows real types in its errors, and gets full rust-analyzer support; a macro is checked per-expansion, its errors point at generated code rather than what you actually typed, and autocomplete inside one is weak. For "same logic, several types," generics already cover it.

The legitimate reasons are narrow: variadic input (`max_of!`), syntax a function simply can't accept (`"k" => "v"` isn't a valid expression, so `string_map!` has to be a macro), wrapping an expression with before/after code without forcing a closure (`timed!`), and generating repetitive items. If a plain `fn` compiles for your problem, the macro version of it is worse. Full stop.

---

## Hands on

```sh
cargo run -p p2-10-02-macro-rules-basics --example 01-matcher-and-transcriber
cargo run -p p2-10-02-macro-rules-basics --example 02-repetition-and-trailing-comma
cargo run -p p2-10-02-macro-rules-basics --example 03-hygiene
```

Then the two broken ones:

```sh
cargo run -p p2-10-02-macro-rules-basics --example 04-wrong-fragment-specifier-broken --features broken
cargo run -p p2-10-02-macro-rules-basics --example 05-missing-base-case-broken --features broken
```

Then try these:

1. In `01-matcher-and-transcriber.rs`, rewrite `square!` so `$x` is evaluated exactly once (hint: pour it into a `let` first) — then call `next()` again and see what `calls` ends up being.
2. In `02-repetition-and-trailing-comma.rs`, add a fourth argument to `string_vec!`, with no trailing comma — does it still compile?
3. In `03-hygiene.rs`, declare another variable named `doubled` with a different value before calling `double_it!` — does the result of `double_it!(7)` change?

---

## Errors you will meet

### `E0425` — a wrong fragment specifier: an `expr` can't stand in for a name

```text
error[E0425]: cannot find value `x` in this scope
  --> phase2-intermediate\10-rust-toolbox\02-macro-rules-basics\examples\04-wrong-fragment-specifier-broken.rs:12:15
   |
12 |     make_var!(x);
   |               ^ not found in this scope

error[E0425]: cannot find value `x` in this scope
  --> phase2-intermediate\10-rust-toolbox\02-macro-rules-basics\examples\04-wrong-fragment-specifier-broken.rs:13:16
   |
13 |     println!("{x}");
   |                ^ not found in this scope

error: arbitrary expressions aren't allowed in patterns
  --> phase2-intermediate\10-rust-toolbox\02-macro-rules-basics\examples\04-wrong-fragment-specifier-broken.rs:12:15
   |
12 |     make_var!(x);
   |               ^
   |
   = note: the `expr` fragment specifier forces the metavariable's content to be an expression

For more information about this error, try `rustc --explain E0425`.
```

**What the compiler is objecting to:** `make_var!`'s matcher reads `($name:expr) => { let $name = 5; };` — it captures `$name` as an **expression**. But in the transcriber, `$name` sits on the left of a `let`, where the compiler wants a **pattern** (the name of a fresh variable), not an expression. As "The concept" said, an `expr` fragment stays opaque once captured — it can't be reinterpreted elsewhere, in a different role. The compiler's own note says exactly this.

**The fix:** change the specifier, not the shape of the transcriber:

```rust
macro_rules! make_var {
    ($name:ident) => {
        let $name = 5;
    };
}
```

**Why this is the fix:** `ident` says "capture a name," not "evaluate an expression." Now `$name` has exactly the role `let` wants, and `make_var!(x)` genuinely builds a fresh variable named `x`.

### Recursion with no base case — "unexpected end of macro invocation"

```text
error: unexpected end of macro invocation
 --> phase2-intermediate\10-rust-toolbox\02-macro-rules-basics\examples\05-missing-base-case-broken.rs:7:52
  |
5 | macro_rules! largest_of {
  | ----------------------- when calling this macro
6 |     ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
7 |         std::cmp::max($first, largest_of!( $( $rest ),+ ))
  |                                                    ^ missing tokens in macro arguments
  |
note: while trying to match `,`
 --> phase2-intermediate\10-rust-toolbox\02-macro-rules-basics\examples\05-missing-base-case-broken.rs:6:18
  |
6 |     ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
  |                  ^
```

**What the compiler is objecting to:** `largest_of!` has exactly one arm: `( $first:expr, $( $rest:expr ),+ $(,)? )` — "at least two arguments: one on its own, and at least one more after a comma." The recursive call `largest_of!( $( $rest ),+ )` eventually reaches a single leftover argument (here, `7`), and there's no comma left for that one arm to match against — and since no other arm accepts a lone argument, the expansion simply stops, mid-invocation. This isn't even a type or borrow error; the compiler is saying the macro call itself never finished.

**The fix:** add a base-case arm for the single-argument case:

```rust
macro_rules! largest_of {
    ( $only:expr ) => {
        $only
    };
    ( $first:expr, $( $rest:expr ),+ $(,)? ) => {
        std::cmp::max($first, largest_of!( $( $rest ),+ ))
    };
}
```

**Why this is the fix:** now, once the recursion reaches a single argument, the first arm (`$only:expr`) is tried before the recursive one and matches — arms are always tried top to bottom. The recursion stops exactly where it should: `largest_of!(3, 9, 7)` becomes `max(3, largest_of!(9, 7))`, the same pattern repeats for `largest_of!(9, 7)`, and `largest_of!(7)` finally reaches the base arm.

---

## Exercises

### Warm up

<details>
<summary>Given <code>macro_rules! square { ($x:expr) => { $x * $x }; }</code>, does <code>square!(1 + 2)</code> expand to <code>1 + 2 * 1 + 2</code> or to <code>(1 + 2) * (1 + 2)</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`(1 + 2) * (1 + 2)`. A captured `expr` stays atomic in the expansion — it keeps the grouping it had where it was captured, so `$x * $x` can't accidentally merge with the caller's own operators. Check it yourself: `square!(1 + 2)` is 9, not 5.

</details>

<details>
<summary><code>timed!</code>'s transcriber builds <code>let start = Instant::now();</code>. If the code calling <code>timed!</code> already has its own variable named <code>start</code>, do the two collide?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. Names introduced *inside* a macro's transcriber live in their own hygiene scope — the macro's `start` and the caller's `start` are different names as far as the compiler is concerned, even though they're spelled identically.

</details>

<details>
<summary>A macro has two arms: <code>($only:expr)</code> and <code>($first:expr, $($rest:expr),+)</code>. Which arm does <code>max_of!(41)</code> match — and which does <code>max_of!(3, 9, 7)</code> match?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`max_of!(41)` — one argument, no comma — matches only the first arm. `max_of!(3, 9, 7)` matches the second arm, since it needs the comma-separated `$first, $rest` shape the first arm can't provide. Arms are always tried top to bottom; if the first arm could also have matched `3, 9, 7`, it would have won.

</details>

<details>
<summary>Given the matcher <code>( $( $key:expr => $value:expr ),+ $(,)? )</code>, does <code>string_map! { "k" => "v", }</code> (note the trailing comma) match?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Yes. `$(,)?` accepts exactly zero or one trailing comma after the last repeated item — the same convenience every built-in macro (`vec!`, `println!`) already gives you.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/04-wrong-fragment-specifier-broken.rs` so `make_var!(x)` genuinely expands `x` into a variable — change the fragment specifier, not the shape of the transcriber.
2. Fix `examples/05-missing-base-case-broken.rs` so `largest_of!(3, 9, 7)` actually compiles and runs — add a base-case arm for the single-argument case.

### Implement

Three macros in `src/lib.rs` — the full specification for each is its own doc comment:

```sh
cargo test -p p2-10-02-macro-rules-basics
```

- `string_map! { "k" => "v", ... }` — builds a `HashMap<String, String>`; repetition, plus the `=>` token in the matcher itself.
- `max_of!(a, b, c, ...)` — variadic maximum, via recursion and a base case.
- `timed!(expr)` — evaluates the expression exactly once, and expands to a `(result, elapsed)` pair.

All three currently expand to a panicking helper called `unsolved` until you rewrite them — the crate builds, but `cargo test` fails. Delete the `unsolved` calls one by one.

### Build

Design and write `min_of!(a, b, c, ...)` yourself — `max_of!`'s small mirror: any number of arguments ≥ 1, any type that's `Ord`, and its result is the smallest of them. No skeleton in `src/lib.rs` is waiting for it, and no hidden test checks it — check yourself: `min_of!(3, 9, 7)` should be `3`, `min_of!(41)` should be `41`.

If you'd rather build something else: write a small `debug_print!(expr)` that prints something like `[file:line] expr = value` — using `file!()`, `line!()`, and `stringify!()`. If you do, congratulations: you're building your own miniature version of the standard library's `dbg!`.

### Challenge (optional)

**Part one.** Write a `list_of!` that accepts either a comma-separated or a semicolon-separated list:

```rust
let a: Vec<i32> = list_of!(1, 2, 3);
let b: Vec<i32> = list_of!(1; 2; 3);
```

Both should produce `[1, 2, 3]`. Hint: write two separate arms, one per separator; since matchers are always tried top to bottom and an `$x:expr` won't stop at a comma or a semicolon on its own, each input matches only one of the two arms.

**Part two** (this one looks ahead). Every macro you wrote in this lesson had at most one layer of repetition. A **tt muncher** — a technique that chews through an arbitrary list of tokens one at a time, from the front, using recursion — can accept far more complicated shapes than that. This lesson doesn't go there; if you're curious, "Going further" below tells you where to look.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `macro_rules!` | Rust's macro-writing system: defines a macro as matcher/transcriber arms | any time the syntax itself must be variadic or novel |
| Matcher | the pattern half of an arm — tried against the caller's tokens | every `macro_rules!` arm |
| Transcriber | the template half of an arm — the tokens a call expands into | every `macro_rules!` arm |
| Fragment specifier | `expr`, `ident`, `ty`, `pat`, `literal`, `tt` — declares what kind of token a capture may be | every `$name:specifier` in a matcher |
| Repetition (`$(...)*`/`+`/`?`) | matches and replays a group zero-or-more, one-or-more, or zero-or-one times | variadic macros |
| Hygiene | names inside a macro can't collide with the caller's own names | any `let` a transcriber builds |
| `$crate` | always names the crate the macro was defined in, no matter who calls it | recursive or self-referencing macros |

### What you now know

- A `macro_rules!` definition is a list of arms, each "matcher => transcriber"; the first arm that matches the call's tokens wins.
- A captured `expr` stays atomic and opaque — it keeps its grouping, and can never later be reinterpreted in a different role, such as a variable name.
- `$(...)*`/`+`/`?` repeats a captured group zero-or-more, one-or-more, or zero-or-one times, in both the matcher and the transcriber.
- Hygiene guarantees a name a macro builds can't collide with an identically-spelled name at the call site — but that same rule means a macro also can't quietly build a variable for the caller; only local names are hygienic this way, not paths, which is what `$crate` is for.
- A recursive macro needs a base case, or the expansion has nowhere to stop.
- Reach for a macro only when a plain function or generic genuinely can't solve the problem — variadic input, non-expression syntax, or wrapping an expression with before/after code.

### What comes back later

- **Cargo features and conditional compilation** — you just used a feature (`broken`) to keep the broken examples behind a flag; 2.10.3 finishes that exact mechanism: [2.10.3 — Cargo features](../03-cargo-features/README.md)

### Can you explain?

- Why does `square!(1 + 2)` expand to `(1 + 2) * (1 + 2)`, not to `1 + 2 * 1 + 2`?
- What does hygiene actually guarantee, and what does it not guarantee?
- Why does a recursive macro like `max_of!` need a base-case arm? What happens if it doesn't have one?
- What's the difference between `$( ... )*`, `$( ... )+`, and `$( ... )?`?
- Give a real example, one you've actually seen, where a plain function did the job a macro was written for instead — or the reverse, a case where only a macro could actually work.

---

## Going further

- [The Rust Book — Macros](https://doc.rust-lang.org/book/ch20-05-macros.html) — the same ground, from the language team itself; includes a brief look at procedural macros too.
- [The Rust Reference — Macros by Example](https://doc.rust-lang.org/reference/macros-by-example.html) — the full list of fragment specifiers, and the formal description of hygiene.
- [The Little Book of Rust Macros](https://lukaswirth.dev/tlborm/) — where you'll learn tt munchers and the genuinely gnarly edges of repetition, whenever you're ready for them.
