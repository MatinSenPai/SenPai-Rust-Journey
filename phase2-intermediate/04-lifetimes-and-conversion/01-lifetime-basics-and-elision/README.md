# 2.4.1 — Lifetime basics and elision

## At a glance

After this lesson you can:

- Explain, in your own words, what a lifetime actually is and is not — and say why `fn longest(x: &str, y: &str) -> &str` refuses to compile without your help.
- Read `E0106` yourself and fix it by writing the right lifetime parameter — and say exactly what that parameter promises.
- Apply the three elision rules to a fresh signature and know, before running the compiler, whether it needs an explicit lifetime.
- Say exactly what `'static` promises, and why a string literal always gets it for free.

**Time:** ~55 minutes · **Prerequisites:** [2.3.7 — Static versus dynamic dispatch, and object safety](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md), and specifically [2.3.2 — Generic functions and structs, bounds, `where`](../../03-traits-and-generics/02-generic-functions-and-structs/README.md) for the promise this lesson finishes

---

## Why this matters

You've been working with references since Phase 1, and behind every borrow the compiler has always been asking itself the same question: "how long is this reference guaranteed to stay valid?" [1.3.3](../../../phase1-fundamentals/03-borrowing-and-references/03-borrow-scopes-and-nll/README.md) answered exactly that question for a single, lone borrow. Today we answer the same question for a function's whole signature.

You've even seen the syntax already — unexplained, on purpose. [1.6.4](../../../phase1-fundamentals/06-absence-and-failure/04-panic-vs-result/README.md) was about panicking versus `Result`, not about lifetimes; but a helper function inside one of that lesson's broken examples had exactly this signature:

```rust
fn find_config_path<'a>(candidates: &[&'a str], wanted: &str) -> Option<&'a str>
```

That `<'a>` was only there so the helper would pass the borrow checker at all; the lesson never once said what those marks mean. If that line looked strange to you back then, today you get the answer.

[1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) showed you the same idea from the other side, without ever using the word "lifetime": a function like `fn shout(text: &str) -> &str` that tried to hand back a reference into a `String` the function itself had just built, moments earlier. The compiler stopped it with `E0515`, and explained itself entirely in ownership vocabulary: `returns a reference to data owned by the current function`. That explanation was completely sufficient — because there was only one possible source for that reference in the first place (not even `text`; `loud`, whose real owner never lived long enough).

Today's problem starts where there is *more than one* possible source. The classic case:

```rust
fn longest(x: &str, y: &str) -> &str
```

Here `x` and `y` are both borrowed, both valid, and the output could plausibly, by its type alone, come from either one. The compiler cannot guess which — and it is right not to. [2.3.2](../../03-traits-and-generics/02-generic-functions-and-structs/README.md) stopped on exactly this signature too, and closed with a promise: it named "the full story of the lifetime on that `&T` `largest` returns" as something this lesson finishes. Let's start.

---

## The concept

### What a lifetime actually is

Before any syntax, pin this definition down, because everything else in this lesson rests on this one sentence:

> **A lifetime is not a value. It never exists anywhere inside the running program. A lifetime is only the name for something the compiler works out, purely at compile time, about a reference: how far it is guaranteed to stay valid.**

A perfectly ordinary example, with no explicit `'a` anywhere:

```rust
fn main() {
    let sentence = String::from("senpai teaches rust");
    let word = &sentence[..6];
    println!("{word}");
}
```

```text
senpai
```

You never typed anything called a lifetime here, but the moment you built `word`, the compiler worked it out: "`word` is only valid for as long as `sentence` is still alive." [1.3.3](../../../phase1-fundamentals/03-borrowing-and-references/03-borrow-scopes-and-nll/README.md) showed you this same accounting under a different name — borrow scope, from where a borrow is taken to its last use. A lifetime is exactly that span, just now with a label you can write down and use to tie *several* different references together — something a single, lone borrow never needed.

If you later see something described as "`x`'s lifetime is three lines" or similar, translate it as: "up to this point, the compiler has proven the reference `x` is still valid." Nothing is stored on disk or in the running program's memory called a lifetime. The compiler throws this accounting away completely once compilation is done.

### Why explicit syntax exists at all

In the example above, the compiler figured out on its own that `word` was tied to `sentence`, because there was only one input reference — no ambiguity at all. The problem starts once there is more than one input reference and the output could, by its type, come from either one.

Take `fn longest(x: &str, y: &str) -> &str` — a function that returns whichever of `x`, `y` is longer. `x` is a reference, `y` is a completely separate reference, and the output is a reference too. Which of the two keeps the output alive? From the signature alone, the answer could be "`x`", could be "`y`" — depending on what the body does, and that body could be an `if` that, depending on run-time data, sometimes returns `x` and sometimes `y`. The compiler has to get this right for *every* possible call, not just one particular run. So when none of the mechanical rules (next section) settle the answer on their own, it does not stay silent and it does not guess — it refuses, and asks you to write the relationship yourself. Run `examples/05-two-inputs-one-output.rs` with `--features broken` right now and that is exactly what happens; the full error is in "Errors you will meet".

### The syntax: `<'a>`, and the promise it makes

Rust's answer is a new kind of parameter — not over types, but over lifetimes themselves, exactly the way `<T>` is a generic parameter over a *type*:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}
```

```rust
let a = String::from("senpai");
let b = String::from("kouhai teaches back");
println!("longest: {}", longest(&a, &b));
```

```text
longest: kouhai teaches back
```

`<'a>` says: "this function is generic over some unknown lifetime called `'a`." That same `'a` then repeats three times — on `x`, on `y`, on the output — and that repetition *is* the promise: whatever `'a` turns out to be at any particular call, `x`, `y`, and the returned value are all guaranteed valid for at least that span. You are not "creating" a lifetime or "making it longer" — the compiler still works out the real, concrete lifetime for `'a` at every call site, on its own; you are only telling it how these three relate to each other.

And because `'a` has to be valid for `x` and `y` at the same time, the only span that works is the overlap of the two — whichever of `x`, `y` ends first. In other words: **the returned value is valid for at most the shorter-lived of the two inputs, never more.**

```senpai-visual
{"kind":"lifetime","labels":["x with its own lifetime","y with its own lifetime","'a ties both together","the output takes 'a's lifetime","'a lasts only as long as the shorter one"]}
```

This promise is not a slogan — the compiler genuinely enforces it at every call. Call the same `longest` above with one input that stops existing too soon:

```rust
let long_lived = String::from("a very long-lived string");
let result;
{
    let short_lived = String::from("short");
    result = longest(long_lived.as_str(), short_lived.as_str());
}
println!("longest: {result}");
```

```text
error[E0597]: `short_lived` does not live long enough
  --> phase2-intermediate\04-lifetimes-and-conversion\01-lifetime-basics-and-elision\examples\06-shorter-input-does-not-live-long-enough.rs:24:47
   |
23 |         let short_lived = String::from("short");
   |             ----------- binding `short_lived` declared here
24 |         result = longest(long_lived.as_str(), short_lived.as_str());
   |                                               ^^^^^^^^^^^ borrowed value does not live long enough
25 |     }
   |     - `short_lived` dropped here while still borrowed
26 |     println!("longest: {result}");
   |                         ------ borrow later used here

For more information about this error, try `rustc --explain E0597`.
```

Read it this way: because `x` and `y` are both tied to `'a`, the compiler must pick an `'a` that works for both `long_lived` and `short_lived` — which means `'a` can be at most as long as `short_lived` lives. `short_lived` stops existing at that closing brace; using `result` afterward means using something that might have been judged by `short_lived`'s lifetime, past the end of that very `'a`. The compiler rejects this — not because `y` was actually returned this time (`x` was longer, so it wasn't), but because the signature guarantees this for *every* possible call, not just this one. This same error, with its fix, is back in "Errors you will meet".

### Elision: three rules that mean you usually don't write this

If you had to write `<'a>` on every function that returns a reference, Rust would be unbearably explicit. In practice you almost never have to — because before anything else, the compiler tries three completely mechanical rules, and only asks you to be explicit when all three come up empty:

1. **Every elided input reference gets its own lifetime.** `fn f(x: &str, y: &str)` is really `fn f<'a, 'b>(x: &'a str, y: &'b str)` — exactly what caught `longest` above without `<'a>`: `x` and `y` got two *separate* lifetimes, not one shared one.
2. **If there is exactly one input lifetime, it is assigned to every elided output lifetime.** That's why `word` in the lesson's first example never needed `<'a>` — there was only one input reference.
3. **If one of the parameters is `&self` or `&mut self`, the output gets that lifetime.** This rule belongs to methods specifically.

See rule 2 concretely — these two functions are, to the compiler, word for word identical:

```rust
fn first_word(s: &str) -> &str {
    s.split_whitespace().next().unwrap_or("")
}

fn first_word_explicit<'a>(s: &'a str) -> &'a str {
    s.split_whitespace().next().unwrap_or("")
}
```

```rust
let sentence = "senpai teaches rust";
println!("elided:   {}", first_word(sentence));
println!("explicit: {}", first_word_explicit(sentence));
```

```text
elided:   senpai
explicit: senpai
```

Rule 1 by itself isn't really "elision" at all — it just means "each gets its own lifetime, and those lifetimes have no relation to each other." That's exactly why rule 2, which only fires on *one* input lifetime, simply does not activate when there are two — it does not guess which one to use.

Rule 3, just as concretely: `Ticket` itself only holds an owned `String` — there is no lifetime on the struct at all. Only the method that borrows from `&self` gets this rule:

```rust
struct Ticket {
    subject: String,
}

impl Ticket {
    fn subject(&self) -> &str {
        &self.subject
    }

    fn subject_explicit<'a>(&'a self) -> &'a str {
        &self.subject
    }
}
```

```rust
let ticket = Ticket {
    subject: String::from("printer is on fire"),
};
println!("elided:   {}", ticket.subject());
println!("explicit: {}", ticket.subject_explicit());
```

```text
elided:   printer is on fire
explicit: printer is on fire
```

(Having a struct hold a reference itself — something like `struct Ticket<'a> { subject: &'a str }` — is a completely separate story; [2.4.2](../02-lifetimes-in-structs-and-methods/README.md) picks it up.)

All three rules together cover nearly every real signature you actually write. `longest` was the exception, not the rule — precisely because it had two inputs and the output could, by its type, have come from either one.

### `'static`: the one special, whole-program lifetime

One particular lifetime has a fixed name: `'static`, meaning "valid for the rest of the running program." String literals always get this for free, because they are baked directly into the compiled binary itself, not allocated anywhere at run time:

```rust
fn announce(message: &'static str) {
    println!("{message}");
}

fn main() {
    let literal: &'static str = "senpai never goes out of scope";
    announce(literal);
    announce("neither does this one");
}
```

```text
senpai never goes out of scope
neither does this one
```

`'static` is really just one more lifetime, not a different kind of thing or a special exception — this one just happens to be tied to the whole program's lifetime instead of some local variable's. One note for later, for whenever you see `T: 'static` on a generic type parameter: there it doesn't mean "lives forever," but more precisely "contains no borrowed data with a shorter lifetime" — a promise that a fully owned `String` or `Vec<T>`, with no reference inside it at all, satisfies just as well as a genuine `&'static str` does.

---

## Hands on

```sh
cargo run -p p2-04-01-lifetime-basics-and-elision --example 01-longest-with-lifetime
cargo run -p p2-04-01-lifetime-basics-and-elision --example 02-elision-rule-two
cargo run -p p2-04-01-lifetime-basics-and-elision --example 03-elision-rule-three-method
cargo run -p p2-04-01-lifetime-basics-and-elision --example 04-static-lifetime
```

Then the two that are broken:

```sh
cargo run -p p2-04-01-lifetime-basics-and-elision --example 05-two-inputs-one-output --features broken
cargo run -p p2-04-01-lifetime-basics-and-elision --example 06-shorter-input-does-not-live-long-enough --features broken
```

Then try:

1. In `01-longest-with-lifetime`, swap the order of `a` and `b` in the call to `longest`. Does the output change? Why does the `>=` (rather than `>`) matter on a tie?
2. In `02-elision-rule-two`, add a third `println!` that prints `first_word("")`. What does it print, and which part of the function is responsible?
3. In `06-shorter-input-does-not-live-long-enough --features broken`, move the `println!` inside the inner block, right after the `result = ...` line. Does it compile now? Why?

---

## Errors you will meet

### `E0106` — missing lifetime specifier

```text
error[E0106]: missing lifetime specifier
  --> phase2-intermediate\04-lifetimes-and-conversion\01-lifetime-basics-and-elision\examples\05-two-inputs-one-output.rs:10:33
   |
10 | fn longest(x: &str, y: &str) -> &str {
   |               ----     ----     ^ expected named lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but the signature does not say whether it is borrowed from `x` or `y`
help: consider introducing a named lifetime parameter
   |
10 | fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
   |           ++++     ++          ++          ++

For more information about this error, try `rustc --explain E0106`.
```

**What the compiler is objecting to:** `x` and `y` are both references, and the output is a reference too. The compiler says exactly this: it does not know whether the output is borrowed from `x` or from `y` — and none of the three elision rules answer that either, because there are two input references, not one.

**The fix:** exactly what the compiler itself suggested — add a lifetime parameter and tie all three spots (both inputs, and the output) to it:

```rust
fn longest<'a>(x: &'a str, y: &'a str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}
```

**Why that's the fix:** the signature now explicitly says the output is tied to *both* inputs, not just one of them. The compiler no longer has to guess — you wrote the relationship, and it only has to check, at every call, that the relationship genuinely holds.

### `E0597` — the shorter input does not live long enough

```text
error[E0597]: `short_lived` does not live long enough
  --> phase2-intermediate\04-lifetimes-and-conversion\01-lifetime-basics-and-elision\examples\06-shorter-input-does-not-live-long-enough.rs:24:47
   |
23 |         let short_lived = String::from("short");
   |             ----------- binding `short_lived` declared here
24 |         result = longest(long_lived.as_str(), short_lived.as_str());
   |                                               ^^^^^^^^^^^ borrowed value does not live long enough
25 |     }
   |     - `short_lived` dropped here while still borrowed
26 |     println!("longest: {result}");
   |                         ------ borrow later used here

For more information about this error, try `rustc --explain E0597`.
```

**What the compiler is objecting to:** `longest<'a>` promised its output is valid for the same `'a` shared by `x` and `y`. `'a` cannot outlast `short_lived`'s own lifetime, because `short_lived` itself has to fit inside `'a` too. `short_lived` stops existing at that `}`; using `result` afterward means using something that might reach beyond `'a`.

**The fix:** either of two things — move the use of `result` before the end of the inner block:

```rust
let long_lived = String::from("a very long-lived string");
{
    let short_lived = String::from("short");
    let result = longest(long_lived.as_str(), short_lived.as_str());
    println!("longest: {result}");
}
```

or keep `short_lived` alive for as long as `long_lived` from the start — that is, declare it outside that block.

**Why that's the fix:** neither fix "extends `short_lived`'s lifetime" — that's not even a thing you can do; a lifetime is a property of the code you wrote, not a number you tune. The first fix makes sure `result` is never used past the end of `short_lived`'s real lifetime. The second fix genuinely makes `short_lived`'s own lifetime longer. Both, in the end, satisfy the same promise the signature makes: the result is only ever used while both inputs are still alive.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
fn first(x: &str, y: &str) -> &str {
    x
}
```

</details>

<details>
<summary>Answer</summary>

No — even though the body only ever returns `x` and never touches `y` at all. Elision rule 2 only activates when there is *exactly one* input lifetime; here there are two. The compiler looks at the signature, not the body — and refuses, the same `E0106`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
fn describe(items: &[String]) -> &str {
    items[0].as_str()
}
```

</details>

<details>
<summary>Answer</summary>

Yes, with no explicit `<'a>` at all. There is exactly one input reference (`items`), so rule 2 works out the output's lifetime from it on its own.

</details>

<details>
<summary>What's the difference between <code>first_word</code> and <code>first_word_explicit</code> in "The concept"?</summary>

None — not in behaviour, not in the code the compiler produces. The second one just spells out, by hand, exactly what rule 2 already writes automatically for the first.

</details>

<details>
<summary>True or false: <code>'static</code> means the data itself stays in memory forever.</summary>

Not quite. `'static` means the *reference* is valid for the rest of the program — string literals qualify because they live inside the binary itself. A fully owned piece of data (like a `String`) doesn't need a reference like that at all; it already keeps itself alive.

</details>

<details>
<summary>Does this compile?</summary>

```rust
fn longest<'a>(x: &'a str, y: &str) -> &'a str {
    if x.len() >= y.len() {
        x
    } else {
        y
    }
}
```

</details>

<details>
<summary>Answer</summary>

No. Only `x` is tied to `'a`; `y` has a completely independent lifetime of its own. The `else` branch wants to return `y` as an `&'a str`, but the compiler cannot prove `y`'s lifetime actually covers `'a` — so it refuses, before it ever reaches a real call.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/05-two-inputs-one-output.rs` so it compiles — write the same lifetime parameter the compiler suggested.
2. Fix `examples/06-shorter-input-does-not-live-long-enough.rs` **two** ways: once by moving `println!` inside the inner block, once by moving `short_lived`'s declaration outside that block. Which one disturbs the real code less?

### Implement

Three signatures in `src/lib.rs`:

```sh
cargo test -p p2-04-01-lifetime-basics-and-elision
```

Before opening each one, say out loud which rule (or the absence of any rule) applies to that signature — then implement it.

### Build

Write a `pub fn` (name and signature your choice) that takes a `&[&'a str]` and returns the longest string in it — `Option<&'a str>`, since the input could be empty. One *shared* lifetime parameter for the whole slice is enough; each element does not need its own. State the tie-breaking rule for equal-length strings in the function's doc comment.

Then write a second version, with the same shape, that returns the shortest string.

### Challenge (optional)

**Part one.** `longest` ties both inputs to one shared `'a` — but that isn't mandatory. Write a function with two *independent* lifetime parameters:

```rust
fn first<'a, 'b>(x: &'a str, y: &'b str) -> &'a str {
    x
}
```

Now call it so that `y` (not `x`) is built in an inner block and dropped before you use the output — exactly the shape of `examples/06-...` above, but this time for the input that *isn't* part of the output. Does it compile? Why is this different from `longest`?

**Part two.** Write a function like `fn first<'a>(x: &'a str, y: &str) -> &'a str { x }` — only `x` is explicit, `y` isn't, and the body only ever returns `x`. Does it compile? Compare the result with Warm up question 1 (`first(x: &str, y: &str) -> &str`, with no `<'a>` on either one, which did not compile) — why is there no problem this time, even though `y` still has no explicit lifetime?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| lifetime | the span the compiler guarantees a reference stays valid for; exists only at compile time | any signature that returns a reference |
| lifetime parameter (`'a`) | a label that ties several references to one shared span of validity | whenever the output could plausibly borrow from more than one input |
| lifetime elision | the three mechanical rules that guess `'a` for you in most signatures | almost every function you write |
| `'static` | the special "for the rest of the program" lifetime; string literals get it for free | fixed text, genuinely global data |

### What you now know

- What a lifetime is: not a value, just a span the compiler guarantees for a reference, purely at compile time.
- Why `fn longest(x: &str, y: &str) -> &str` refuses to compile without your help: two input references, no signal for which one the output comes from.
- You can read and write `<'a>` syntax, and know exactly what it promises: the output is valid for at least the shorter-lived of the inputs tied to `'a`.
- You can state the three elision rules from memory, and tell, on a fresh signature, which one (if any) applies.
- You know exactly what `'static` promises, and why string literals get it for free.
- You can produce, read, and fix `E0106` and `E0597` yourself.

### What comes back later

- **When a struct itself holds a reference, and why there is no elision rule for that** — [2.4.2 — Lifetimes in structs and methods](../02-lifetimes-in-structs-and-methods/README.md)
- **`Deref`, `AsRef`, `Borrow`, `ToOwned`** — [2.4.3](../03-deref-asref-borrow/README.md)
- **`Cow<'_, str>` and copy-on-write** — [2.4.4](../04-cow-and-clone-on-write/README.md)

### Can you explain?

- In your own words, say what a lifetime is and is not.
- Why can't the compiler guess whether `longest`, without `<'a>`, borrows its output from `x` or from `y`?
- What exactly does the signature `fn longest<'a>(x: &'a str, y: &'a str) -> &'a str` promise about `x`, `y`, and the output?
- State the three elision rules from memory. Which one activates on a plain `&self` method?
- Why does `fn first(x: &str, y: &str) -> &str { x }` refuse to compile, even though it never touches `y`?
- What exactly does `'static` guarantee about a reference? Why do string literals get it for free?

---

## Going further

- [The Rust Book — Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html) — the same ground, official and in more depth.
- [The Rust Reference — Lifetime elision](https://doc.rust-lang.org/reference/lifetime-elision.html) — the exact wording of the same three rules you saw today.
- [The Rust Book — The Static Lifetime](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html#the-static-lifetime) — the same chapter's section devoted to `'static`.
- [The Rustonomicon — Lifetimes](https://doc.rust-lang.org/nomicon/lifetimes.html) — for when you're curious to go further under the hood (borrow scope, subtyping).
