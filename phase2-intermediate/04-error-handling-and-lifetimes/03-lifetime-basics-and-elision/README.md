# 04.3 — Lifetime basics and elision

You've been writing correct lifetime-dependent code since Phase 1 without
naming lifetimes explicitly (`longest_word` in
`phase1-fundamentals/04-strings-and-slices/01-string-vs-str`, `find_by_anime`
in the anime-quote-cli side-quest). This lesson makes explicit what the
compiler has been inferring for you all along.

## What a lifetime actually is

A lifetime isn't a value and doesn't exist at runtime — it's the
compiler's name for **how long a reference stays valid**, used purely to
check your code at compile time. `'a` is just a label, the same way `T` is
just a label for a generic type — `'a` on its own means nothing; it means
something when it appears on *multiple* things and ties their validity
together:

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}
```

This signature says: "given two string slices that are *both* valid for at
least lifetime `'a`, I'll return a string slice that's also valid for `'a`."
It's a constraint on the *relationship* between the inputs and output, not
a lifetime you're inventing — the compiler still infers the actual,
concrete lifetime at each call site; you're just telling it how the inputs
and output relate.

## Elision: the three rules that mean you usually don't write this

Most functions with references never need explicit lifetimes, because the
compiler applies three rules first, and only asks you to be explicit when
they don't produce an unambiguous answer:

1. **Each elided input reference gets its own lifetime.** `fn foo(x: &str,
   y: &str)` is really `fn foo<'a, 'b>(x: &'a str, y: &'b str)`.
2. **If there's exactly one input lifetime, it's assigned to every elided
   output lifetime.** This is why `fn longest_word(text: &str) -> &str`
   (Phase 1) never needed `<'a>` — one input reference, so the compiler
   knows the output must borrow from it.
3. **If one of the inputs is `&self` or `&mut self` (a method), the output
   gets `self`'s lifetime.** This is why methods returning a reference into
   `self` almost never need explicit annotations either.

`longest` above needs `<'a>` explicitly because it has **two** input
references (rule 1 gives them separate lifetimes `'a`/`'b` by default) and
one output reference — nothing in the three rules picks which input's
lifetime the output should get, so the compiler stops and makes you say it.

## When a struct holds a reference

```rust
struct Excerpt<'a> {
    text: &'a str,
}
```

A struct can't outlive data it borrows — `Excerpt<'a>` says "no `Excerpt`
value can outlive the `&str` its `text` field points into." This is always
explicit; there's no elision rule for struct fields.

## `'static`

`'static` means "valid for the entire remainder of the program" — string
literals (`"hello"`) are `&'static str` because they're baked directly into
the compiled binary, not allocated at runtime. Seeing `'static` in a bound
like `T: 'static` doesn't always mean "lives forever" though — on a generic
type parameter it more precisely means "contains no borrowed data with a
shorter lifetime," which includes fully owned data (`String`, `Vec<T>`)
just as much as it includes genuinely `'static` references.

## Your task

Implement everything in `src/lib.rs`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
