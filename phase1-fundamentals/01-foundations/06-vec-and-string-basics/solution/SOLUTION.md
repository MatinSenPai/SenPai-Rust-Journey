# Solution — 1.1.6 `Vec` and `String` basics

```rust
pub fn evens_up_to(n: u32) -> Vec<u32> {
    let mut evens = Vec::new();
    for value in 0..=n {
        if value % 2 == 0 {
            evens.push(value);
        }
    }
    evens
}

pub fn total(values: Vec<i64>) -> i64 {
    let mut sum = 0;
    for value in &values {
        sum += value;
    }
    sum
}

pub fn largest(values: Vec<i32>) -> i32 {
    let mut best = values[0];
    for value in &values {
        if *value > best {
            best = *value;
        }
    }
    best
}

pub fn joined(parts: Vec<String>, separator: char) -> String {
    let mut out = String::new();
    for index in 0..parts.len() {
        if index > 0 {
            out.push(separator);
        }
        out.push_str(&parts[index]);
    }
    out
}

pub fn byte_and_char_count(text: String) -> (usize, usize) {
    (text.len(), text.chars().count())
}
```

## `evens_up_to` — `Vec::new()` and where its type comes from

```rust
let mut evens = Vec::new();
```

No type annotation, and it compiles. Rust looks ahead: two lines later you `push` a `u32`, and the function promises `Vec<u32>`, so `evens` is a `Vec<u32>`. Delete the `push` and the annotation becomes compulsory, because there'd be nothing left to infer from.

The `0..=n` matters: `0..n` would drop `n` itself, so `evens_up_to(10)` would stop at 8. Same trap as [1.1.5](../../05-control-flow/README.md).

A shorter version exists using [Phase 2](../../../../phase2-intermediate/02-iterators-and-closures/README.md)'s tools:

```rust
(0..=n).filter(|value| value % 2 == 0).collect()
```

One line, no `mut`, no intermediate. Worth seeing now so that when iterators arrive you recognise what they're replacing.

## `total` — `&values` and the reason it's there

```rust
for value in &values {
```

Drop the `&` and it still compiles — and then `values` is gone, consumed by the loop. Try it: add `println!("{values:?}")` after the loop and the `&` version compiles while the other doesn't.

That's your first sight of **ownership**, which is the whole of [module 1.2](../../../02-ownership-and-memory/README.md). The one-line version: `for value in values` gives the loop the Vec, and `for value in &values` only lends it. Since this function has no further use for `values`, either works here. The `&` is the habit worth building.

`sum += value` where `value` is `&i64` also works, because `+=` knows what to do with a reference to a number. That's a convenience you'll stop noticing.

## `largest` — the two dereferences

```rust
if *value > best {
    best = *value;
}
```

`value` here is a `&i32`, not an `i32`, because `&values` lends each element rather than handing it over. The `*` says "the thing being pointed at". Without it you'd be comparing a reference against a number, and the compiler would object.

There's a shorter form worth knowing: pattern the reference away in the loop itself.

```rust
for &value in &values {
    if value > best {
        best = value;
    }
}
```

That `&value` on the left is a *pattern* — the same machinery as [1.1.3](../../03-compound-types-and-destructuring/README.md), matching the shape "a reference to something" and binding the something. Both forms are idiomatic. All of this gets its proper lesson in [1.3.1](../../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md).

**`values[0]` is a real risk** and the spec is what saves it. The doc comment says `values` is never empty; if it were, `values[0]` panics. Once you have `Option` you'd write `.first()` and there would be nothing to promise.

## `joined` — why this one indexes

Every other exercise here loops over elements. This one loops over positions:

```rust
for index in 0..parts.len() {
    if index > 0 {
        out.push(separator);
    }
```

Because the rule isn't about the parts, it's about the *gaps between* them: a separator goes before every part except the first. The index is the thing that knows which part is first.

The flag version is equally correct and slightly clumsier:

```rust
let mut first = true;
for part in &parts {
    if !first {
        out.push(separator);
    }
    out.push_str(part);
    first = false;
}
```

**`push` versus `push_str`.** `push` takes one `char`; `push_str` takes text. The separator is a `char`, the parts are `String`s, so each gets its own method. Rust doesn't have one overloaded `push` because `char` and `&str` really are different things — a `char` is exactly one Unicode scalar and a `&str` is any number of bytes.

The standard library already does this, of course:

```rust
parts.join(&separator.to_string())
```

You were asked to build it by hand once so you know what `join` is doing. [1.4.3](../../../04-text-and-strings/03-building-and-transforming-strings/README.md) covers the real thing.

## `byte_and_char_count` — the important one

```rust
(text.len(), text.chars().count())
```

One line, and it's the most important line in the lesson.

```text
english:   hello
  bytes:   5
  chars:   5
persian:   سلام
  bytes:   8
  chars:   4
```

**`len()` is bytes.** Always. It's not "usually characters and sometimes not" — it is a byte count that happens to equal the character count when every character is ASCII. Which is exactly why this bug survives: if all your test data is English, `len()` looks like it counts characters and every test passes.

Ship that to Persian users and it breaks: text truncated mid-letter, "maximum 20 characters" that rejects a ten-letter name, column widths that don't line up.

**Why is `len()` bytes and not characters?** Because `len()` is instant and counting characters isn't. A `String` knows how many bytes it holds; to know how many characters, something has to walk the whole thing decoding as it goes — which is precisely what `.chars().count()` does. Rust gives the cheap operation the short name and makes you ask for the expensive one, rather than hiding an O(n) walk behind something that looks like a field access.

**And "character" is itself the wrong word**, which is where this gets genuinely deep. `.chars()` counts Unicode scalars. For `سلام` that's 4 and matches what you'd say. For text with combining marks, or an emoji with a skin-tone modifier, one thing a person would call a character can be several scalars. [1.4.2 — UTF-8, bytes, chars, graphemes](../../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md) is the whole lesson, and for someone writing software that handles Persian it's one of the most valuable in the course.

## What this lesson was really about

- **`Vec<T>` and `String` are the same idea twice**: an owned, growable buffer on the heap. `String` is very nearly `Vec<u8>` with a promise that the bytes are valid UTF-8.
- **Growable means owned means heap**, and that has consequences you're about to meet head-on in [module 1.2](../../../02-ownership-and-memory/README.md).
- **`&` on a loop means "lend, don't give"** — the first ownership decision you've had to make, and the last lesson where you can get away with not understanding it.
- **`len()` is bytes.** Write it on something.
