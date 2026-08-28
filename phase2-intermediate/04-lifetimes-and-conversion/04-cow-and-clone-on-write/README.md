# 2.4.4 — `Cow<'_, str>` and copy-on-write

## At a glance

After this lesson you can:

- Explain exactly what `Cow<'a, B>` is — a real enum with two states — and why you rarely need to `match` on it yourself.
- Write a function that takes zero copies in the common case and only builds a fresh `String` when it actually needs to, in a way its caller cannot tell apart except in cost.
- Say exactly when `.to_mut()` clones, and honestly say when `Cow` is worth that extra complexity and when it is not.

**Time:** ~50 minutes · **Prerequisites:**
[2.4.2 — Lifetimes in structs and methods](../02-lifetimes-in-structs-and-methods/README.md),
[2.4.3 — `Deref`, `AsRef`, `Borrow`, `ToOwned`](../03-deref-asref-borrow/README.md)

---

## Why this matters

Write a function that takes a `&str` and needs to "clean it up" — collapse extra spaces, append a suffix, whatever. Most of the input this function ever sees is already clean; people usually type things correctly. Now you're stuck with a choice that has no good answer at first glance.

If the signature is `-> String`, every single call — even for that already-clean input — has to build a fresh `String`, purely because you have to return something whose type doesn't match the input. A wasted allocation, every time. If the signature is `-> &str`, that's perfect for the common case — free, zero copies — but the moment you actually need to change something, you can't: the modified version is not the same `&str` you were handed, and there is nowhere to return it from.

This is exactly where Rust adds a third option. `Cow<'_, str>` — short for **c**lone-**o**n-**w**rite — lets a function return that same copy-free view in the common case, and only build an owning `String` in the case that actually needs one. The caller cannot tell the difference except by cost — exactly the thing you never have to think about in Python or Django, because the garbage collector always hands you a fresh object and hides that cost from you. Rust does not hide the cost — and `Cow` exists precisely so you don't always have to pay it.

This lesson closes module 2.4. The three lessons before it each laid down one piece of the same puzzle: [2.4.1](../01-lifetime-basics-and-elision/README.md) put an explicit name on "how long a borrow stays valid," [2.4.2](../02-lifetimes-in-structs-and-methods/README.md) showed how a struct is allowed to hold onto a reference of its own, and [2.4.3](../03-deref-asref-borrow/README.md) opened up a cheap path between a borrowed view and its owned counterpart. Today those three come together to build a fourth thing: a type that decides for itself exactly when to actually make that conversion.

---

## The concept

### What `Cow<'a, B>` actually is: a two-state enum

The standard library's own one-line description of `Cow` calls it "a clone-on-write smart pointer." That's true of its behavior — thanks to `Deref`, it acts exactly like a reference to `B` — but structurally there is no secret at all. Open up its real definition:

```rust
pub enum Cow<'a, B>
where
    B: 'a + ToOwned + ?Sized,
{
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}
```

That's it. A plain enum with two variants — exactly the kind of thing [1.5.3](../../../phase1-fundamentals/05-your-own-types/03-enums-as-data/README.md) already taught you. No raw pointer, no hidden trick: either you have a borrowed reference (`Borrowed`), or you have an owned copy (`Owned`). Because it's a real enum, you can `match` on it exactly like any other:

```rust
fn describe(value: &Cow<'_, str>) -> &'static str {
    match value {
        Cow::Borrowed(_) => "borrowed",
        Cow::Owned(_) => "owned",
    }
}
```

(It deliberately takes `&Cow<'_, str>`, not `&str` — clippy will usually suggest `&str` for a `&Cow` parameter, and most of the time that advice is right. But here the function's whole job is figuring out which variant it is; a plain `&str` has already erased that information. So the signature stays as it is, with a `#[allow(clippy::ptr_arg)]` and a comment above it saying why — silencing a lint on purpose, once you know exactly why it's wrong here, is its own skill.)

Now build one of each and call it:

```rust
let borrowed: Cow<str> = Cow::Borrowed("Matin");
let owned: Cow<str> = Cow::Owned(String::from("Matin"));

println!("borrowed: {borrowed:?} -> {}", describe(&borrowed));
println!("owned:    {owned:?} -> {}", describe(&owned));
```

```text
borrowed: "Matin" -> borrowed
owned:    "Matin" -> owned
```

One precise detail: the `{:?}` output is just `"Matin"`, never `Borrowed("Matin")` or `Owned("Matin")`. `Cow`'s `Debug` implementation forwards straight to the inner value, not to the variant's name — so `{:?}` alone never tells you which state you have. For that you need either a `match`, or exactly the `describe` helper above.

Even so, you rarely need to `match` by hand, because [2.4.3](../03-deref-asref-borrow/README.md) already solved this: `Cow<'a, B>` implements `Deref<Target = B>`, so any method that works on `&B` works directly on the `Cow` itself:

```rust
println!("borrowed.len(): {}", borrowed.len());
println!("owned.len():    {}", owned.len());
println!("borrowed == owned: {}", borrowed == owned);
```

```text
borrowed.len(): 5
owned.len():    5
borrowed == owned: true
```

`.len()` here has no idea `Cow` exists — it's plain `str::len()`, reached through `Deref`. `borrowed` and `owned` were compared with `==`, not some `Cow`-specific method; as far as the comparison is concerned, both values are equal because both reach the same `str`, through that same `Deref`.

### The motivating shape: a function that usually needs no change

Now make "why this matters" concrete: a function that collapses runs of consecutive spaces in a string down to one — a small piece of text cleanup, exactly the kind you see a lot on user input or a config file.

```rust
fn collapse_spaces(input: &str) -> Cow<'_, str> {
    if !input.contains("  ") {
        return Cow::Borrowed(input);
    }
    let mut collapsed = String::with_capacity(input.len());
    let mut previous_was_space = false;
    for ch in input.chars() {
        if ch == ' ' && previous_was_space {
            continue;
        }
        previous_was_space = ch == ' ';
        collapsed.push(ch);
    }
    Cow::Owned(collapsed)
}
```

The second line is the whole point of this lesson: before doing anything else, the function runs one cheap check — is there even a run of two spaces in here at all? If not, it hands `input` straight back with zero copies. Only when a run genuinely exists does it take the slower path and build a fresh `String`.

```senpai-visual
{"kind":"borrowing","labels":["&str input","two spaces in a row?","no: Cow::Borrowed, 0 copies","yes: build String","Cow::Owned, 1 copy"]}
```

Call it with a clean input and a messy one, and compare memory addresses — not just content:

```rust
println!("clean out:  {clean_result:?}");
println!(
    "clean same address as input? {}",
    clean_result.as_ptr() == clean.as_ptr()
);
println!("messy out:  {messy_result:?}");
println!(
    "messy same address as input? {}",
    messy_result.as_ptr() == messy.as_ptr()
);
```

```text
clean out:  "Trigun: 26 episodes"
clean same address as input? true
messy out:  "Trigun: 26 episodes"
messy same address as input? false
```

This isn't just a claim — it's proof. For the clean input, the output's address is exactly the input's address: nothing moved, nothing was allocated. For the messy input, the address changed — a genuinely new `String` was built. The caller sees neither of these facts in the return type; both are a `Cow<'_, str>`, and both behave, through `Deref`, exactly like a `&str`. The only place the difference shows up is cost.

### `.to_mut()`: where the copy actually happens

Every `Cow` you've built so far started out already knowing which state it was in. But the real mechanism behind "clone on write" is one method: `.to_mut()`. Its own documentation says exactly what it does: "Acquires a mutable reference to the owned form of the data. Clones the data if it is not already owned."

```senpai-visual
{"kind":"ownership","labels":["Cow::Borrowed(&str)",".to_mut() called","clone happens once","Cow::Owned(String)","later .to_mut() calls: no clone"]}
```

Build a `Cow` as `Borrowed` and walk through it:

```rust
let source = String::from("Trigun");
let mut value: Cow<str> = Cow::Borrowed(&source);
println!(
    "before .to_mut(): borrowed? {}",
    matches!(value, Cow::Borrowed(_))
);

value.to_mut().push_str(" - 26 episodes");
println!(
    "after .to_mut():  borrowed? {}",
    matches!(value, Cow::Borrowed(_))
);
println!("value:  {value:?}");
println!("source: {source:?} (never touched)");
```

```text
before .to_mut(): borrowed? true
after .to_mut():  borrowed? false
value:  "Trigun - 26 episodes"
source: "Trigun" (never touched)
```

Before `.to_mut()`, `value` was a `Cow::Borrowed` — no copy at all, just a reference into `source`. Calling `.to_mut()` hands back a `&mut String`; to build that reference for real, it first cloned `source` and put that copy inside `value`. `.push_str(...)` then wrote to that copy — `source` stayed untouched, exactly as the printout shows.

One sharper detail: the clone happens at the moment `.to_mut()` is *called*, not at the moment you actually write through the `&mut` it hands back. Even if you never touch the returned reference at all, that call alone is enough to move the `Cow` to `Owned` — "clone on write" is more precisely "clone on requesting write access," not "clone the instant a write actually occurs."

And calling it a second time? Since `value` is already `Owned`, there is nothing left to clone:

```rust
let cloned_address = value.as_ptr();
let _ = value.to_mut(); // already `Owned` — nothing left to clone
println!(
    "a second .to_mut() moved the data? {}",
    value.as_ptr() != cloned_address
);
```

```text
a second .to_mut() moved the data? false
```

This is exactly what earns the name "clone on write": the copy happens exactly once, right at that first transition from `Borrowed` to `Owned`.

### Tying back to 2.4.3: `Deref` and `ToOwned` at work

`Cow` invents no new mechanism of its own — it is exactly the two traits [2.4.3](../03-deref-asref-borrow/README.md) showed you, put to work. First `Deref`: because `Cow<'a, B>: Deref<Target = B>`, a function that wants `&B` accepts a `&Cow<'a, B>` with no change at all — the same deref coercion [2.4.3](../03-deref-asref-borrow/README.md) showed for `&String`:

```rust
fn shout(input: &str) -> String {
    format!("{}!", input.to_uppercase())
}
```

```rust
let value: Cow<str> = Cow::Borrowed("trigun");
println!("{}", shout(&value));
```

```text
TRIGUN!
```

`shout` never even knows `Cow` exists; all it sees is a `&str`.

Second, `ToOwned`: look back at the enum definition above — the `Owned` variant holds exactly `<B as ToOwned>::Owned`, not `B` itself. For `B = str`, that's `String` — the same type [2.4.3](../03-deref-asref-borrow/README.md)'s `.to_owned()` produces:

```rust
let built_with_to_owned: Cow<str> = Cow::Owned("trigun".to_owned());
let built_with_string_from: Cow<str> = Cow::Owned(String::from("trigun"));
println!(
    "same value either way: {}",
    built_with_to_owned == built_with_string_from
);
```

```text
same value either way: true
```

Both routes land on the same type, because both are ways of reaching that same `String`. That's not a coincidence: `ToOwned` sets up this exact relationship for every borrowed type that implements it — `[T]` to `Vec<T>`, `Path` to `PathBuf`, `OsStr` to `OsString` — and `Cow<'_, [T]>`, `Cow<'_, Path>` work exactly the same way. Today's lesson only needs `str`/`String`, but the mechanism is general.

### When `Cow` is really worth it, and when it is not

`Cow` is not free — it's an enum with two states, a `match` (or a `Deref`) everywhere you use it, and a return type that looks a little heavier than a plain `String`. That complexity earns its keep when the "no change needed" case is genuinely common — common enough that avoiding the allocation for it actually adds up to something.

When it doesn't: a function that always allocates, no matter what you feed it. Look at this:

```rust
fn tag_always(input: &str) -> Cow<'_, str> {
    Cow::Owned(format!("[{input}]"))
}

fn tag_always_plain(input: &str) -> String {
    format!("[{input}]")
}
```

```rust
for input in ["Trigun", "Blame!", ""] {
    let wrapped = tag_always(input);
    println!(
        "{input:?} -> {wrapped:?} (borrowed? {})",
        matches!(wrapped, Cow::Borrowed(_))
    );
}
```

```text
"Trigun" -> "[Trigun]" (borrowed? false)
"Blame!" -> "[Blame!]" (borrowed? false)
"" -> "[]" (borrowed? false)
```

`false`, all three times. Wrapping a string in brackets can never reuse the input's own bytes — it always needs a fresh allocation. Returning `Cow` here is just an extra layer for the caller: they still have to either `Deref` through it or call `.into_owned()` to get a real `String`, and not a single byte of copying was ever avoided in exchange. `tag_always_plain` does exactly the same work, with a simpler contract. The rule: reach for `Cow` when you genuinely have a common, unmodified path worth preserving — not because it "might come in handy somewhere."

---

## Hands on

```sh
cargo run -p p2-04-04-cow-and-clone-on-write --example 01-cow-is-an-enum
cargo run -p p2-04-04-cow-and-clone-on-write --example 02-collapse-spaces-fast-and-slow-path
cargo run -p p2-04-04-cow-and-clone-on-write --example 03-to-mut-triggers-the-clone
cargo run -p p2-04-04-cow-and-clone-on-write --example 04-owned-is-exactly-toowned-owned
cargo run -p p2-04-04-cow-and-clone-on-write --example 05-when-cow-is-not-worth-it
```

Then the three broken ones:

```sh
cargo run -p p2-04-04-cow-and-clone-on-write --example 06-forgot-to-wrap-in-cow-broken --features broken
cargo run -p p2-04-04-cow-and-clone-on-write --example 07-owned-wants-string-broken --features broken
cargo run -p p2-04-04-cow-and-clone-on-write --example 08-to-mut-borrow-conflict-broken --features broken
```

Then try these:

1. In `01-cow-is-an-enum.rs`, build a third value, `Cow::Owned(String::from("Someone Else"))`, and compare it with `borrowed`. Guess what `==` gives you, then check.
2. In `02-collapse-spaces-fast-and-slow-path.rs`, add a string with a tab between two words instead of spaces. Does it still come back `Borrowed`? Why must it?
3. In `03-to-mut-triggers-the-clone.rs`, change the first line from `Cow::Borrowed(&source)` to `Cow::Owned(source.clone())`. What does `matches!(value, Cow::Borrowed(_))` give before `.to_mut()` now, and why?

---

## Errors you will meet

### `E0308` — forgetting to wrap the return in `Cow`

```text
error[E0308]: mismatched types
  --> phase2-intermediate\04-lifetimes-and-conversion\04-cow-and-clone-on-write\examples\06-forgot-to-wrap-in-cow-broken.rs:11:5
   |
10 | fn passthrough(input: &str) -> Cow<'_, str> {
   |                                ------------ expected `Cow<'_, str>` because of return type
11 |     input
   |     ^^^^^ expected `Cow<'_, str>`, found `&str`
   |
   = note:   expected enum `Cow<'_, str>`
           found reference `&str`
help: try wrapping the expression in `std::borrow::Cow::Borrowed`
   |
11 |     std::borrow::Cow::Borrowed(input)
   |     +++++++++++++++++++++++++++     +

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is actually objecting to:** `Cow<'_, str>` does not automatically absorb a `&str` — unlike `Deref`, which only works in the direction "from `Cow` to `&str`," there is no implicit conversion the other way. You have to say explicitly which variant you mean.

**The fix:** wrap it in `Cow::Borrowed`:

```rust
fn passthrough(input: &str) -> Cow<'_, str> {
    Cow::Borrowed(input)
}
```

**Why this is the fix:** `input` is exactly what should come back — completely unmodified — so `Borrowed` is exactly the variant this needs. The compiler's own suggestion says the same thing.

### `E0271` — the `Owned` variant needs a `String`, not a `&str`

```text
error[E0271]: type mismatch resolving `<str as ToOwned>::Owned == &str`
  --> phase2-intermediate\04-lifetimes-and-conversion\04-cow-and-clone-on-write\examples\07-owned-wants-string-broken.rs:12:27
   |
12 |     let value: Cow<str> = Cow::Owned("Trigun");
   |                           ^^^^^^^^^^^^^^^^^^^^ expected `&str`, found `String`

For more information about this error, try `rustc --explain E0271`.
```

**What the compiler is actually objecting to:** `Cow::Owned` needs a value of type `<B as ToOwned>::Owned`, not of type `B`. For `Cow<str>` that means `String`, not `&str` — even though `&str` looks "owned enough" at a glance. (Read the expected/found the opposite way you might guess: `expected` names what you actually wrote — `"Trigun"`, a `&str`; `found` names what this position genuinely requires — `String`.)

**The fix:** hand it a real `String`:

```rust
let value: Cow<str> = Cow::Owned("Trigun".to_owned());
```

**Why this is the fix:** `.to_owned()` builds exactly what `<str as ToOwned>::Owned` requires — a `String`. `String::from("Trigun")` is just as correct; both land on the same type, exactly what the "Tying back to 2.4.3" subsection showed.

### `E0502` — holding on to `.to_mut()`'s reference while also reading the `Cow`

```text
error[E0502]: cannot borrow `value` as immutable because it is also borrowed as mutable
  --> phase2-intermediate\04-lifetimes-and-conversion\04-cow-and-clone-on-write\examples\08-to-mut-borrow-conflict-broken.rs:14:24
   |
13 |     let episodes = value.to_mut();
   |                    ----- mutable borrow occurs here
14 |     println!("before: {value}");
   |                        ^^^^^ immutable borrow occurs here
15 |     episodes.push_str(" - 26 episodes");
   |     -------- mutable borrow later used here

For more information about this error, try `rustc --explain E0502`.
```

**What the compiler is actually objecting to:** `.to_mut()` takes `&mut self`, and what it returns keeps that mutable borrow alive until it's used again — exactly the same aliasing rule you already know from Phase 1: any number of shared borrows, or exactly one mutable borrow, never both at once. `episodes` is still alive (line 15 uses it again), so the `println!` on line 14 cannot take a shared borrow of `value`.

**The fix:** let the `.to_mut()` borrow end before reading `value`:

```rust
let mut value: Cow<str> = Cow::Borrowed("Trigun");
value.to_mut().push_str(" - 26 episodes");
println!("after: {value}");
```

**Why this is the fix:** here the result of `.to_mut()` is never stored in a variable at all — the borrow ends the instant `.push_str(...)` is called. By the time `println!` runs, there is no live mutable borrow left to conflict with it. This is exactly what [1.3.3](../../../phase1-fundamentals/03-borrowing-and-references/03-borrow-scopes-and-nll/README.md) called a **borrow scope** — a borrow lives until its *last use*, not until the end of the block.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
use std::borrow::Cow;

let value: Cow<str> = Cow::Owned("Trigun");
```

</details>

<details>
<summary>Answer</summary>

No. `Cow::Owned` needs a `String`, not a `&str` — `"Trigun"` is a `&str`. The error is `E0271`, exactly what you saw in "Errors you will meet."

</details>

<details>
<summary>What does this print?</summary>

```rust
let value: Cow<str> = Cow::Borrowed("Trigun");
println!("{value:?}");
```

</details>

<details>
<summary>Answer</summary>

```text
"Trigun"
```

Not `Borrowed("Trigun")`. `Cow`'s `Debug` forwards straight to the inner value, never to the variant's name.

</details>

<details>
<summary>Does this compile?</summary>

```rust
fn shorten(input: &str) -> Cow<'_, str> {
    input
}
```

</details>

<details>
<summary>Answer</summary>

No. `Cow<'_, str>` does not automatically absorb a `&str`; you have to write `Cow::Borrowed(input)` explicitly.

</details>

<details>
<summary>After running this, what are <code>before</code> and <code>after</code>?</summary>

```rust
let mut value: Cow<str> = Cow::Borrowed("Trigun");
let before = matches!(value, Cow::Borrowed(_));
value.to_mut();
let after = matches!(value, Cow::Borrowed(_));
```

</details>

<details>
<summary>Answer</summary>

`before` is `true`, `after` is `false` — even though the value `.to_mut()` returned was never used. Simply *calling* `.to_mut()` is enough to trigger the clone.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/06-forgot-to-wrap-in-cow-broken.rs` with `Cow::Borrowed(input)`.
2. Fix `examples/07-owned-wants-string-broken.rs` so `Cow::Owned` gets a real `String` — either `.to_owned()` or `String::from`, whichever you prefer.
3. Rewrite `examples/08-to-mut-borrow-conflict-broken.rs` so the `.to_mut()` borrow ends before the `println!`.

### Implement

Three functions in `src/lib.rs`:

```sh
cargo test -p p2-04-04-cow-and-clone-on-write
```

- `describe` — says whether a `Cow` is borrowed or owned.
- `collapse_spaces` — this lesson's motivating function: collapses runs of consecutive spaces, allocating only when it actually needs to.
- `ensure_exclaimed` — makes sure a `Cow<str>` ends with `'!'`, using `.to_mut()` to add one only when it's actually missing.

The exact specification — including each variant's precise behavior — is in the doc comment above each function.

### Build

Write another function of your own, in the same shape as `collapse_spaces`: `pub fn your_name(input: &str) -> Cow<'_, str>` that does some simple piece of text cleanup and only allocates when it genuinely needs to — trimming leading/trailing spaces only if any exist, say, or replacing one forbidden character only if it appears. In a comment above the function, explain why you think the "no change needed" case is genuinely common for it — the same argument the "when `Cow` is worth it" subsection asked you to make.

### Challenge (optional)

**Part one.** On your own (no need to add it to `src/lib.rs`), write `fn to_static(input: &str) -> Cow<'static, str>`. Explain out loud why `Cow::Borrowed(input)` can never be a valid return from this function, no matter what `input` is — and why that forces this function, unlike `collapse_spaces`, to always end up `Owned`.

**Part two.** (This one looks ahead.) `ToOwned` isn't only for `str` — `[T]` implements it too, with `Owned = Vec<T>`. Write a small version of the same idea for `Cow<'_, [i32]>`: a function that removes consecutive duplicate numbers only when they actually occur, otherwise returning the input slice completely unchanged.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Cow<'a, B>` | a two-state enum: `Borrowed(&'a B)` or `Owned(<B as ToOwned>::Owned)` | a function that usually just reads, but sometimes has to build |
| `Cow::Borrowed` | the zero-copy state — just a reference, nothing allocated | the common, unmodified path |
| `Cow::Owned` | the allocated state — exactly the type `ToOwned::Owned` produces | the case that genuinely needed a copy |
| `.to_mut()` | gives a `&mut <B as ToOwned>::Owned`; clones once, only if still `Borrowed` | the actual mechanism behind "clone on write" |
| clone-on-write | deferring a copy until the moment something actually needs to write | data that is read far more often than it's changed |

### What you now know

- `Cow<'a, B>` is a real enum with two variants, not a smart-pointer trick; `Deref` is why you rarely need to `match` on it yourself.
- The `Owned` variant holds exactly `<B as ToOwned>::Owned` — `String` for `str` — never a plain `B`.
- `.to_mut()` is the actual clone mechanism: it copies only on the first transition from `Borrowed` to `Owned`, and merely calling it is enough, even without writing anything.
- Returning `Cow` from a function lets it answer with zero copies in the common case and allocate only when it must, with the caller unable to tell the difference except in cost.
- `Cow` earns its keep when the unmodified path is genuinely common; for a function that always allocates, it's only extra complexity.

### What comes back later

- **`Rc<str>`/`Arc<str>` for genuinely shared ownership** — `Cow` solves "I might need to build my own copy"; when the real problem is that several owners need to hold the same value, cheaply, that's a different problem — [2.6.3 — `Rc` and `Arc`](../../06-smart-pointers/03-rc-and-arc/README.md).
- **Measuring whether this saving actually matters for you** — the "when it's worth it" subsection argued by reasoning, not by measurement; to actually measure it instead of guessing — [2.7.5 — Benchmarking with `criterion`](../../07-project-structure-and-testing/05-benchmarking-with-criterion/README.md).

### Can you explain?

- Name `Cow<'a, B>`'s two variants, and say why the `Owned` one doesn't just hold a plain `B`.
- Why does calling `.to_mut()` a second time, once a `Cow` is already `Owned`, never trigger a fresh clone?
- Why does `describe` deliberately take `&Cow<'_, str>` instead of `&str` — against clippy's default advice?
- Trace the module's throughline: how does naming how long a borrow lives (2.4.1) lead, step by step, to being able to put off a copy until it's actually needed (2.4.4)?
- Give a real (or plausible) example of a function where `Cow` is not worth it, and say why.

---

## Going further

- [`std::borrow::Cow`](https://doc.rust-lang.org/std/borrow/enum.Cow.html) — the official docs, with the full list of its methods.
- [`std::borrow::ToOwned`](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html) — the same trait 2.4.3 showed you, straight from the docs; it also lists every type that implements it.
- [The `std::borrow` module](https://doc.rust-lang.org/std/borrow/index.html) — an overview of how `Borrow`, `ToOwned`, and `Cow` relate to each other.
