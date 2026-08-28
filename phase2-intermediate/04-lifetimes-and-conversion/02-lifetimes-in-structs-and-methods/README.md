# 2.4.2 — Lifetimes in structs and methods

## At a glance

After this lesson you can:

- Add a lifetime parameter to a struct that borrows data, and say exactly what that parameter promises — including why the program refuses to compile when a value tries to outlive the data it points into (and fix `E0106` and `E0597` yourself when you hit them).
- Write a method on that struct which returns a reference without ever naming `'a` yourself, and say exactly which of 2.4.1's three elision rules is doing that work — and when a signature genuinely still needs the explicit form.
- Decide, for a real piece of code, whether a new type should borrow a `&str` or own a `String` — and explain that trade-off out loud, not just recite the syntax.

**Time:** ~40 minutes · **Prerequisites:** [2.4.1 — Lifetime basics and elision](../01-lifetime-basics-and-elision/README.md)

---

## Why this matters

2.4.1 touched this, briefly, in a parenthetical aside, and moved on: holding a reference inside a struct field itself — something like `struct Ticket<'a> { subject: &'a str }` — is a completely separate story, it said, and pointed here for it.

That same lesson came close to this one more time, without quite showing it in full: its `Ticket::subject(&self) -> &str` method demonstrated elision rule 3 on a struct. But that `Ticket` had no lifetime of its own — its field was a fully owned `String`, and the method was just slicing into it. Today you see that same rule 3 on a struct that already carries its own `'a` — and whether that method needs `-> &'a str` or not is less obvious than it looks.

And there's a more practical question sitting behind all of this: why would a struct hold a reference at all? Why not just grab a `String` everywhere and be done with it? The answer is a real design decision, not a syntax rule — and that's exactly what the last part of this lesson opens up.

---

## The concept

### A struct holding a reference must name its lifetime

Say you want to hold on to a piece of a larger text — the first line of a long log entry, for instance — without copying the whole thing. The struct says exactly that:

```rust
struct Excerpt<'a> {
    text: &'a str,
}
```

The `text` field doesn't own the string; it only refers to it. And as 2.4.1 told you, there's no elision rule for struct fields at all — drop the `<'a>` from the struct and the `'a` from in front of `&str`, and it won't compile. The compiler suggests exactly that `<'a>` itself (the full error is in "Errors you will meet").

With the parameter in place, building and using one is entirely ordinary:

```rust
let article = String::from("Ownership is central. Borrowing comes next.");
let excerpt = Excerpt { text: &article };

println!("excerpt: {}", excerpt.text);
println!("source:  {article}");
```

```text
excerpt: Ownership is central. Borrowing comes next.
source:  Ownership is central. Borrowing comes next.
```

### The consequence is load-bearing: an Excerpt can't outlive its data

`<'a>` is not decoration. It's exactly what the compiler uses, from here on, to track every value of `Excerpt<'a>` — the same way it already tracked a plain `&str`. The example above compiled because `article` stayed alive for as long as `excerpt` was used. Now reverse that order: build `article` inside a smaller block, and use `excerpt` outside it, after that block ends.

```senpai-visual
{"kind":"lifetime","labels":["article is created","excerpt borrows article","inner block ends","article is dropped","excerpt still points at article","E0597: does not live long enough"]}
```

That no longer compiles — the compiler runs the exact same rule on `Excerpt` that it runs on any other reference: you're holding a `&article`, so you cannot outlive `article` itself. The only difference from a plain reference is that this one is hiding inside a struct — but the compiler looks straight through the struct and sees the same underlying borrow. The full code, with its real error, is in `examples/03-excerpt-outlives-its-source.rs` and "Errors you will meet".

### Elision rule 3, now on methods, doing real work

Now write a method that actually computes something, not just a field getter:

```rust
impl<'a> Excerpt<'a> {
    fn first_line(source: &'a str) -> Self {
        let text = source.lines().next().unwrap_or(source);
        Excerpt { text }
    }
}
```

This constructor — since it doesn't take `&self` at all — writes `source`'s lifetime explicitly: `source: &'a str`. No elision rule can guess that `source`'s data is about to become the struct's own `'a`; you have to say it yourself. (Try dropping the `&'a` in front of `source` and it won't compile — `rustc` says "explicit lifetime required", because `source`'s elided lifetime has no guaranteed relationship to the struct's own `'a`.)

Now a real method, on the same struct:

```rust
impl<'a> Excerpt<'a> {
    // No 'a here. Rule 3: there's one &self input, so the elided output
    // lifetime is that borrow of self — not the struct's own 'a.
    fn text(&self) -> &str {
        self.text
    }

    fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
}
```

This looks like it should need `-> &'a str`, the same way the struct itself did. It doesn't. `text(&self) -> &str` compiles with no explicit lifetime at all, and it works correctly — that's elision rule 3 from 2.4.1: because one of the inputs is `&self`, the elided output lifetime becomes the lifetime of that `self` borrow, not the struct's own `'a`. And that's exactly what you want: `self.text` (which is `&'a str`) is always valid for at least as long as the borrow of `self` itself is — there's no way to have a `&Excerpt<'a>` around while the data underneath it is already gone — so shortening it down to `self`'s own, usually-shorter lifetime is always safe.

```senpai-visual
{"kind":"concept","labels":["call excerpt.text()","self is borrowed just for the call","rule 3 ties output to self","self.text already outlives self","so shortening it is safe","the returned &str is valid"]}
```

Try it with that struct:

```rust
let log_entry = String::from("connection reset by peer\nretrying in 3s\ngiving up");
let excerpt = Excerpt::first_line(&log_entry);

println!("first line: {}", excerpt.text());
println!("word count: {}", excerpt.word_count());
```

```text
first line: connection reset by peer
word count: 4
```

### Borrow or own: a real design decision

`Excerpt<'a>` is a borrow, not a default. When the source data is already alive somewhere and stays alive for as long as you need it, `&'a str` is free — nothing gets copied, nothing gets freshly allocated. That's exactly this case: `logs` outlives the function call, so the `Excerpt`s it returns can safely borrow from it:

```rust
fn first_lines_of<'a>(logs: &'a [String]) -> Vec<Excerpt<'a>> {
    logs.iter().map(|log| Excerpt::first_line(log)).collect()
}
```

But sometimes there's nothing to borrow from. If every string is only temporary — built fresh inside a loop, like a network response or a decoded buffer — no `Excerpt<'a>` could borrow from it and still be around after that one iteration ends. Owning is the answer here:

```rust
fn fetch_response(id: u32) -> String {
    format!("200 OK (request {id})\nbody omitted")
}

fn first_line_of_each_response(request_ids: &[u32]) -> Vec<String> {
    let mut summaries = Vec::new();
    for id in request_ids {
        let response = fetch_response(*id);
        let first_line = response.lines().next().unwrap_or("").to_string();
        summaries.push(first_line);
    }
    summaries
}
```

Run both side by side:

```rust
let logs = vec![
    String::from("started\nrunning"),
    String::from("connected\nidle"),
];
for excerpt in first_lines_of(&logs) {
    println!("borrowed: {}", excerpt.text);
}

for summary in first_line_of_each_response(&[1, 2]) {
    println!("owned:    {summary}");
}
```

```text
borrowed: started
borrowed: connected
owned:    200 OK (request 1)
owned:    200 OK (request 2)
```

The practical rule: if the source's lifetime is already guaranteed to outlast you, take `&'a str` — it's free. If the source is temporary, or there's no single source to point back to at all (say, you're building the string yourself with `format!`), take `String`. There's a third option in between: a type called `Cow<'_, str>`, built for exactly this "maybe borrowed, maybe owned" case — 2.4.4 covers it in full. You don't need it yet; today the choice is binary, and that's fine.

---

## Hands on

```sh
cargo run -p p2-04-02-lifetimes-in-structs-and-methods --example 01-excerpt-needs-a-lifetime
cargo run -p p2-04-02-lifetimes-in-structs-and-methods --example 04-methods-and-elision
cargo run -p p2-04-02-lifetimes-in-structs-and-methods --example 05-borrow-or-own
```

Then the two broken ones:

```sh
cargo run -p p2-04-02-lifetimes-in-structs-and-methods --example 02-struct-without-lifetime --features broken
cargo run -p p2-04-02-lifetimes-in-structs-and-methods --example 03-excerpt-outlives-its-source --features broken
```

Then try these:

1. In `01-excerpt-needs-a-lifetime`, add `drop(article);` right after building `excerpt`, before the two `println!`s. Predict what happens, then run it and read the error.
2. In `04-methods-and-elision`, add `fn is_empty(&self) -> bool { self.text.is_empty() }`. Does it need its own lifetime?
3. In `05-borrow-or-own`, change `first_lines_of`'s return type to `Vec<String>` and see what has to change in its body to make that compile.

---

## Errors you will meet

### `E0106` — missing lifetime specifier

```text
error[E0106]: missing lifetime specifier
 --> phase2-intermediate\04-lifetimes-and-conversion\02-lifetimes-in-structs-and-methods\examples\02-struct-without-lifetime.rs:6:11
  |
6 |     text: &str,
  |           ^ expected named lifetime parameter
  |
help: consider introducing a named lifetime parameter
  |
5 ~ struct Excerpt<'a> {
6 ~     text: &'a str,
  |

For more information about this error, try `rustc --explain E0106`.
```

**What the compiler is objecting to:** the `text` field is a reference (`&str`), but no lifetime is named for it, and for struct fields — unlike function parameters — there is no elision rule to guess one. The compiler needs to know exactly how long this reference stays valid so it can track every `Excerpt` value by it.

**The fix:** exactly what the compiler suggests:

```rust
struct Excerpt<'a> {
    text: &'a str,
}
```

**Why this is the fix:** `<'a>` on the struct itself defines the parameter that `text` can refer to. Now every value of `Excerpt<'a>` has a precise answer to "how long is this good for" — which is exactly what the next error shows you when that promise gets broken.

### `E0597` — borrowed value does not live long enough

```text
error[E0597]: `article` does not live long enough
  --> phase2-intermediate\04-lifetimes-and-conversion\02-lifetimes-in-structs-and-methods\examples\03-excerpt-outlives-its-source.rs:13:35
   |
12 |         let article = String::from("Ownership is central. Borrowing comes next.");
   |             ------- binding `article` declared here
13 |         excerpt = Excerpt { text: &article };
   |                                   ^^^^^^^^ borrowed value does not live long enough
14 |     }
   |     - `article` dropped here while still borrowed
15 |     println!("first line: {}", excerpt.text);
   |                                ------------ borrow later used here

For more information about this error, try `rustc --explain E0597`.
```

**What the compiler is objecting to:** `excerpt` is holding a `&article`, but `article` is declared inside a nested block and dropped right there. `excerpt` is still used outside that block — meaning it would be pointing at data that no longer exists. This is exactly what `<'a>` on the struct promised would never happen.

**The fix:** reorder so the source outlives its user — without deleting the inner block:

```rust
let article = String::from("Ownership is central. Borrowing comes next.");
let excerpt;
{
    excerpt = Excerpt { text: &article };
}
println!("first line: {}", excerpt.text);
```

**Why this is the fix:** `article` is now declared outside any block that `excerpt` is built in, so it's still alive by the time `excerpt` gets used. Nothing about `Excerpt` itself changed — the order the data stays alive in now matches the order it's used in, exactly what the compiler wants for any other reference too.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
struct Coupon {
    code: &str,
}
```

</details>

<details>
<summary>Answer</summary>

No. `code` is a reference with no lifetime — exactly the `E0106` shape from "Errors you will meet". It needs to be `struct Coupon<'a> { code: &'a str }`.

</details>

<details>
<summary>What does this print?</summary>

```rust
struct Coupon<'a> {
    code: &'a str,
}

let id = String::from("A1029");
let coupon = Coupon { code: &id };
println!("{}", coupon.code);
```

</details>

<details>
<summary>Answer</summary>

```text
A1029
```

`id` stays alive for as long as `coupon` is used — exactly the same working shape as the lesson's own first struct.

</details>

<details>
<summary>Does this compile?</summary>

```rust
impl<'a> Coupon<'a> {
    fn code(&self) -> &str {
        self.code
    }
}
```

</details>

<details>
<summary>Answer</summary>

Yes. There's one `&self` input, so elision rule 3 gives the elided output lifetime the lifetime of that `self` borrow — no `-> &'a str` needed.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let coupon;
{
    let id = String::from("A1029");
    coupon = Coupon { code: &id };
}
println!("{}", coupon.code);
```

</details>

<details>
<summary>Answer</summary>

No. The same `E0597` shape: `id` is dropped inside the inner block, but `coupon` is still used outside it.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/02-struct-without-lifetime.rs` so it compiles — by adding the missing lifetime parameter to the struct, exactly what the compiler's own suggestion says.
2. Fix `examples/03-excerpt-outlives-its-source.rs` so it compiles — without deleting the inner block; move where `article` is declared instead, so it outlives `excerpt`.

### Implement

Four pieces in `src/lib.rs`:

```sh
cargo test -p p2-04-02-lifetimes-in-structs-and-methods
```

`struct Excerpt` and its `first_line` constructor are already fully written — exactly what you saw in "The concept". Your job is one more constructor (`whole`), two methods (`text`, `word_count`), and one plain function (`total_words`). The exact format for each one is written in the doc comment right above it — don't guess.

### Build

Design a small struct of your own that holds a `&'a str` (or `&'a [T]`) — a parser, a tokenizer, a validator, whatever real scenario you like, just not `Excerpt`. Give it at least one method that takes `&self` and returns a reference, written without any explicit lifetime. Write at least one test for it.

### Challenge (optional)

Next to `text`, give `Excerpt` a second method that returns the same `self.text`, but with the struct's own lifetime written explicitly instead of elided — `-> &'a str` instead of `-> &str`. Both compile. Now write one more free function, outside the `impl` block, that takes a `&Excerpt<'a>` and tries to return a `&'a str` by calling one of the two methods. Only one of them lets it compile — which one, and why? (Hint: think about what the elided version's return value is actually tied to — it isn't `'a`.)

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Struct lifetime parameter | `<'a>` on a struct that holds a reference; ties every value of that type to the data it borrows | Any type storing `&T` instead of owning `T` |
| `E0106` | missing lifetime specifier | Forgetting `<'a>` on a struct field that's a reference |
| `E0597` | borrowed value does not live long enough | Using a struct value after the data it borrows is dropped |
| Elision rule 3 on methods | the elided output gets `self`'s own lifetime, not the struct's `'a` | Any method taking `&self` on a lifetime-holding struct |
| Borrow or own | `&'a str` when the source outlives you; `String` when it doesn't | Designing a new type around existing text |

### What you now know

- A struct holding a reference must carry its own lifetime parameter — there is no elision rule for struct fields.
- That parameter is load-bearing: no value of that struct can outlive the data its reference points into, enforced exactly the way any other borrow is.
- Forgetting the parameter is `E0106`; letting a value outlive its source is `E0597` — both caught at compile time, before the program ever runs.
- A method taking `&self` on such a struct almost never needs its own explicit lifetime — elision rule 3 ties an elided output to `&self`'s own borrow, and that's exactly correct because the struct's own lifetime is always at least as long as any borrow of it.
- An associated function that doesn't take `&self` (a constructor) gets none of that help — if it returns `Self` with a lifetime inside, you write that lifetime explicitly yourself.
- Whether a new type should borrow (`&'a str`) or own (`String`) is a real design decision, not a syntax rule: borrow when the source clearly outlives the new value; own when it doesn't, or when there's no single source to borrow from at all.

### What comes back later

- **`Deref`, `AsRef`, `Borrow`, `ToOwned` — the full family of borrowing conversions** — [2.4.3 — `Deref`, `AsRef`, `Borrow`, `ToOwned`](../03-deref-asref-borrow/README.md)
- **`Cow<'_, str>` — the third option, between borrowing and owning** — [2.4.4 — `Cow` and copy-on-write](../04-cow-and-clone-on-write/README.md)

### Can you explain?

- Why does a struct holding a reference need its own lifetime parameter, when plenty of function parameters don't?
- What does `Excerpt<'a>` actually promise about any value of that type?
- Why is forgetting the struct's lifetime a different error from a value not outliving its source? What is the compiler objecting to in each case?
- Why doesn't `fn text(&self) -> &str` need `-> &'a str`? Which elision rule does that, and what does it actually tie the output to?
- Why does `first_line`, which doesn't take `&self`, have to write its lifetime explicitly?
- When do you reach for `&'a str` in a new type, and when do you reach for `String` instead?

---

## Going further

- [The Rust Book — Validating References with Lifetimes](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html) — the same ground, official and complete.
- [The Rust Reference — Lifetime elision](https://doc.rust-lang.org/reference/lifetime-elision.html) — the precise definition of all three rules.
- [`rustc --explain E0106`](https://doc.rust-lang.org/error_codes/E0106.html) and [`rustc --explain E0597`](https://doc.rust-lang.org/error_codes/E0597.html) — the official explanation of both errors you saw today.
