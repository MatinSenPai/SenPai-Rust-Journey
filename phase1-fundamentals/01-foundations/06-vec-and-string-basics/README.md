# 1.1.6 — `Vec` and `String` basics

## At a glance

After this lesson you can:

- Build a `Vec` and a `String`, grow them, and read them back.
- Say why an array lives on the stack and a `Vec` lives on the heap, and what that means.
- Say why `text.len()` on "سلام" is 8 and not 4 — and why that's the most important thing in this lesson.
- State the difference between `String` and `&str` in one sentence.

**Time:** ~50 minutes · **Prerequisites:** [1.1.5 — Control flow](../05-control-flow/README.md)

---

## Why this matters

You met arrays in [1.1.3](../03-compound-types-and-destructuring/README.md), and they had one large restriction: the length was part of the type, so they couldn't grow. For real data that's nearly always wrong. How many lines in a file, how many users online, how many rows a query returned — none of those are known at compile time.

`Vec<T>` is the array without that restriction, and `String` is the same thing for text. If you know Python, these are `list` and `str` and they behave much the same way.

Two things are different and both are worth your attention.

**First, you've now genuinely reached the heap.** An array's size was known at compile time, so the compiler could set aside room for it on the stack. A `Vec` can't be — its size isn't known until run time — so its contents live on the heap. That's not an implementation footnote; it's what the whole of [module 1.2](../../02-ownership-and-memory/README.md) is about, and it's where the rest of Rust starts making sense.

**Second, text.** And here Rust does something Python doesn't: it makes you distinguish "how many bytes" from "how many letters". If you work in English that's annoying and looks pointless. For you, writing Persian, **it's exactly the distinction that decides whether your software works or cuts a user's name in half.**

---

## The concept

### `Vec<T>` — the array that grows

Three ways to make one, and you'll see all three:

```rust
let mut readings: Vec<i32> = Vec::new();
let preset = vec![12, 7, 19];
let zeroed = vec![0_u8; 5];
```

```text
empty:     []
preset:    [12, 7, 19]
zeroed:    [0, 0, 0, 0, 0]
```

That `vec![]` is a macro — the `!` says so — and it takes the same syntax as an array, including the "this value, this many times" form.

`Vec::new()` has no elements to infer a type from, so either you annotate it or a later `push` settles it.

#### Growing and reading

```rust
readings.push(12);
readings.push(7);
readings.push(19);
```

```text
pushed:    [12, 7, 19]
length:    3
empty?     false
popped:    Some(19)
after pop: [12, 7]
```

`push` adds one on the end. `pop` takes the last one off and hands back an **`Option`** — because there might not be a last one. The same `Option` you saw from `checked_add` and `.get()`; [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md) finally does it properly.

Everything an array could do, a `Vec` can do:

```text
first:     12
get(9):    None
```

#### Looping without consuming it

```rust
let mut total = 0;
for reading in &readings {
    total += reading;
}
```

```text
total:     19
still here:[12, 7]
```

Look at that `&`. **Without it this still compiles** — and then `readings` doesn't exist after the loop.

This is the first time Rust asks you for an ownership decision: do I *give* the loop the Vec, or only *lend* it? `&` means "just look". For now, write the `&`; [module 1.2](../../02-ownership-and-memory/README.md) and [1.3.1](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md) do the rest.

#### Length versus capacity

```text
len / cap: 0 / 0
  push 0: 1 / 4
  push 1: 2 / 4
  push 2: 3 / 4
  push 3: 4 / 4
  push 4: 5 / 8
```

**Length** is how many are in it. **Capacity** is how much room was reserved.

When length reaches capacity, the `Vec` takes a bigger block, copies everything over, and releases the old one. That's the 4-to-8 jump. Because it doubles each time, `n` pushes cost time proportional to `n` overall rather than `n²` — the same amortised analysis behind Python's `list`.

If you roughly know how many are coming, say so:

```rust
let sized: Vec<i32> = Vec::with_capacity(100);
```

```text
reserved:  0 / 100
```

Still empty — capacity is reserved room, not contents.

### `String` — the text buffer that grows

The same pattern, exactly:

```rust
let empty = String::new();
let from_literal = String::from("hello");
let converted = "hello".to_string();
```

```text
empty:     ""
from:      "hello"
converted: "hello"
```

And it grows the same way:

```rust
let mut greeting = String::from("hello");
greeting.push_str(", world");
greeting.push('!');
```

```text
greeting:  hello, world!
```

Two methods, because they take two different things: `push_str` appends text and `push` appends a single `char`. Rust doesn't merge them because they really aren't the same — a `char` is exactly one Unicode scalar.

### And now the one that catches everyone

```rust
let english = String::from("hello");
let persian = String::from("سلام");
```

```text
english:   hello
  bytes:   5
  chars:   5
persian:   سلام
  bytes:   8
  chars:   4
```

**`len()` counts bytes, not letters.**

For `hello` both are 5, and that's what makes this bug so durable: if all your test data is English, `len()` behaves as though it counts letters and everything looks right.

For `سلام` it's 8 and 4. Each Persian letter takes two bytes in UTF-8.

Count the consequences:

- "maximum 20 characters" on a Persian name is really a maximum of ten letters.
- Truncating at byte twenty can land mid-letter and produce broken text.
- Column widths based on `len()` don't line up for non-English text.

**Why does Rust do this?** Because `len()` is instant: a `String` already knows its byte count. Counting letters means walking the whole thing decoding as it goes, and that costs something. Rust gives the short name to the cheap operation and makes you ask explicitly for the expensive one:

```rust
text.chars().count()
```

Python hides this behind `len()` and pays for it elsewhere. Rust would rather you knew which question you were asking.

> **And "character" is itself imprecise.** `.chars()` counts Unicode scalars, which for Persian is usually what you want, but not for text with combining marks or emoji. [1.4.2](../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md) opens the whole thing up, and for you it's one of the most important lessons in the course.

### `String` versus `&str` — the one-paragraph version

```rust
let borrowed: &str = "I am a literal";
let owned: String = borrowed.to_string();
```

- **`String`** **owns** its text. It lives on the heap, it grows, and it's destroyed when its owner goes.
- **`&str`** is a **view** of text that is somewhere else. It doesn't grow and it owns nothing.

Every string literal you write in your program is a `&str` — baked into the executable, always there.

Here's the practical rule, and start following it now:

```rust
fn shout(text: &str) -> String {
    text.to_uppercase()
}
```

```text
shout:     I AM A LITERAL
shout:     A LITERAL DIRECTLY
```

**Take `&str` in parameters and return `String`.** Because any `String` can be lent out as a `&str`, a function taking `&str` accepts both; a function taking `String` forces the caller to hand over ownership or make a pointless copy.

The full lesson is [1.4.1](../../04-text-and-strings/01-string-vs-str/README.md). For now that rule is enough.

### Stack and heap, at a glance

| | Where it lives | Size known |
|---|---|---|
| `i32`, `bool`, `char` | stack | at compile time |
| `[i32; 5]` | stack | at compile time |
| `Vec<i32>` | contents on the heap | at run time |
| `String` | contents on the heap | at run time |
| `&str` to a literal | nowhere — it just looks | at compile time |

The `Vec` variable itself is on the stack; what's there is three numbers: a pointer, a length, a capacity. It's the pointer that reaches into the heap.

**That's what makes ownership necessary.** That heap block has to be released one day, and released exactly once. [1.2.1](../../02-ownership-and-memory/01-stack-and-heap/README.md) starts there.

---

## Hands on

```sh
cargo run -p p1-01-06-vec-and-string-basics --example 01-vec-basics
cargo run -p p1-01-06-vec-and-string-basics --example 02-string-basics
cargo run -p p1-01-06-vec-and-string-basics --example 03-collecting-lines
```

Then the two broken ones:

```sh
cargo run -p p1-01-06-vec-and-string-basics --example 04-indexing-a-string --features broken
cargo run -p p1-01-06-vec-and-string-basics --example 05-array-is-not-a-vec --features broken
```

Then try:

1. In `01-vec-basics`, remove the `&` from `for reading in &readings`. What error do you get, and where?
2. In `02-string-basics`, add your own name in Persian and print its bytes and characters. What's the ratio?
3. In `01-vec-basics`, extend the push loop to 20. At which numbers does the capacity jump?

---

## Errors you will meet

### `E0277` — a `String` can't be indexed by a number

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
   --> examples\04-indexing-a-string.rs:10:26
    |
 10 |     let first = greeting[0];
    |                          ^ string indices are ranges of `usize`
    |
    = help: the trait `SliceIndex<str>` is not implemented for `{integer}`
    = note: you can use `.chars().nth()` or `.bytes().nth()`
            for more information, see chapter 8 in The Book
    ...
    = note: required for `String` to implement `Index<{integer}>`
```

**What the compiler is objecting to:** what does `greeting[0]` mean? The first byte or the first letter? For English text they're the same; for `سلام` the first byte is half a letter and isn't a valid `char` at all.

Rust can't decide which you meant, and won't hand you an operation that works for English text and silently corrupts Persian. So it doesn't offer the operation at all.

**The fix:** say which you want. `greeting.chars().nth(0)` gives the first letter, `greeting.bytes().nth(0)` the first byte. Both hand back an `Option`, because the text might be empty.

**Why that's the fix:** look at that `note` — the compiler named both options and even pointed at the chapter of the Book. It isn't a telling-off; it's saying your question was ambiguous and here are the two unambiguous ones. And `.nth(0)`'s cost isn't hidden either: it has to count from the start, because letters aren't fixed width.

### `E0308` — an array isn't a `Vec`

```text
error[E0308]: mismatched types
  --> examples\05-array-is-not-a-vec.rs:10:30
   |
10 |     let growable: Vec<i32> = fixed;
   |                   --------   ^^^^^ expected `Vec<i32>`, found `[{integer}; 3]`
   |                   |
   |                   expected due to this
   |
   = note: expected struct `Vec<i32>`
               found array `[{integer}; 3]`
help: try using a conversion method
   |
10 |     let growable: Vec<i32> = fixed.to_vec();
   |                                   +++++++++
```

**What the compiler is objecting to:** they hold the same contents and print identically, but they're two different types living in two different places. An array is on the stack with its size in its type; a `Vec` is a pointer into the heap.

**The fix:** `fixed.to_vec()`, exactly as the compiler suggests.

**Why that's the fix:** the conversion isn't free, which is why it isn't automatic. `to_vec()` allocates on the heap and copies the contents across. Rust never allocates behind an assignment for you; if there's an allocation, it's visible in the code.

---

## Exercises

### Warm up

<details>
<summary>What's the difference between a <code>Vec</code> and an array, in one sentence?</summary>

An array has its length in its type and sits on the stack; a `Vec` grows whenever you want and keeps its contents on the heap.

</details>

<details>
<summary>What does <code>String::from("سلام").len()</code> give?</summary>

8. Each Persian letter is two bytes in UTF-8. To count letters, take `.chars().count()`, which gives 4.

</details>

<details>
<summary>Why does <code>len()</code> count bytes rather than letters?</summary>

Because it already knows the byte count, and counting letters means walking the whole string. Rust gives the short name to the cheap operation and makes you ask for the expensive one.

</details>

<details>
<summary><code>String</code> versus <code>&str</code>, in one sentence?</summary>

A `String` owns its text and grows; a `&str` is only a view of text that lives somewhere else.

</details>

<details>
<summary>In <code>for x in &values</code>, what does the <code>&</code> do?</summary>

It lends the Vec to the loop instead of giving it away, so you can still use it afterwards.

</details>

<details>
<summary>How is capacity different from length?</summary>

Length is how many elements are in it; capacity is how much room is reserved. When length reaches capacity, the `Vec` takes a bigger block and moves the contents.

</details>

### Repair

Fix `examples/04-indexing-a-string.rs` two different ways: once so it gives the first **byte**, once the first **letter**. Run both on `"hello"` and on `"سلام"`.

Put the answers side by side. For English they're the same; for Persian they aren't. **That's why Rust refused to choose for you.**

Then fix `examples/05-array-is-not-a-vec.rs`.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-01-06-vec-and-string-basics
```

`byte_and_char_count` is one line and the most important of them. Its test has both Persian and Latin data, deliberately.

### Build

Write a `pub fn truncate_to_chars(text: String, max_chars: usize) -> String` that cuts text down to at most `max_chars` **letters** (not bytes).

Then write the wrong version too — the one that cuts at `max_chars` bytes — and run both on `"سلام دنیا"` with `max_chars` of 5.

The wrong version either produces broken text or panics. **Work out which, and why.** This is precisely the bug that hits Persian text in real systems.

### Challenge (optional)

**Part one.** Run this and explain it:

```rust
let mut v: Vec<i32> = Vec::new();
for n in 0..1000 {
    if v.len() == v.capacity() {
        println!("regrowing at len {}", v.len());
    }
    v.push(n);
}
```

How many times did it reallocate? What if you'd started with `Vec::with_capacity(1000)`?

**Part two.** Without running it, say what this prints:

```rust
let s = String::from("سلام");
println!("{}", s.len());
println!("{}", s.chars().count());
println!("{:?}", s.chars().nth(1));
println!("{:?}", s.bytes().nth(1));
```

Then run it. Why are those last two lines so different?

**Part three.** Build a `String` that has 3 letters but is 9 bytes. (Hint: you aren't restricted to Persian.)

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Vec<T>` | growable array, contents on the heap | any list whose length isn't known up front |
| `vec![]` | the `Vec`-building macro | fixed starting contents |
| `push` / `pop` | add one / take the last one off | growing and shrinking |
| capacity | reserved room, as against length | `with_capacity` when you know roughly |
| `String` | owned, growable text buffer | text you build |
| `&str` | a view of text that lives elsewhere | function parameters |
| `push_str` / `push` | append text / append one `char` | building text |
| `len()` | the number of **bytes** | always — remember it |
| `.chars().count()` | the number of Unicode scalars | anywhere letters matter |
| `&` on a loop | lend rather than give | a loop you need the value after |

### What you now know

- `Vec<T>` and `String` are both owned, growable buffers on the heap.
- Capacity is not length, and a `Vec` grows by doubling.
- `String` owns and `&str` views; take `&str` in parameters.
- `len()` counts bytes. For Persian text that isn't the letter count.
- A `String` can't be indexed by a number, and the reason is a good one.
- `&` on a loop means lend, not give — and that's your first ownership decision.

### What comes back later

- **Why growable means owned** — [1.2.1 — Stack and heap](../../02-ownership-and-memory/01-stack-and-heap/README.md)
- **That `&` in full** — [1.3.1 — References](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
- **`String` versus `&str`, the whole lesson** — [1.4.1](../../04-text-and-strings/01-string-vs-str/README.md)
- **Bytes, chars, graphemes** — [1.4.2](../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md)
- **Building text the way it's really written** — [1.4.3](../../04-text-and-strings/03-building-and-transforming-strings/README.md)
- **The `Option` that came out of `pop`, `get` and `last`** — [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **Loops that become iterator chains** — [Phase 2](../../../phase2-intermediate/02-iterators-and-closures/README.md)

### Can you explain?

- How do a `Vec` and an array differ, and where does each live?
- What is capacity, and why does a `Vec` grow by doubling?
- What does `"سلام".len()` give, and why?
- Why doesn't `s[0]` compile on a `String`?
- How do `String` and `&str` differ, and which should a function parameter be?
- What's the difference between `for x in values` and `for x in &values`?

---

## Going further

- [The Rust Book — Vectors](https://doc.rust-lang.org/book/ch08-01-vectors.html) and [Strings](https://doc.rust-lang.org/book/ch08-02-strings.html) — the same ground, officially.
- [`std::vec::Vec`](https://doc.rust-lang.org/std/vec/struct.Vec.html) — the full method list. Scroll it once so you know what's there.
- [`std::string::String`](https://doc.rust-lang.org/std/string/struct.String.html) — the same for text.
