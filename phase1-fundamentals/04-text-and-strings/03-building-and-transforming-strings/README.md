# 1.4.3 — Building and transforming strings

## At a glance

After this lesson you can:

- Build text with `format!` and say how many allocations each call makes.
- Read a specifier like `{:>8}`, `{:.2}` or `{:#?}` and say exactly what it does.
- Choose between `push_str`, `+`, `concat`, `join` and `format!` by cost rather than habit.
- Explain why `{:>8}` lines up English text and not Persian — and by exactly how much it misses.

**Time:** ~50 minutes · **Prerequisites:**
[1.4.2 — UTF-8: bytes, chars, graphemes](../02-utf8-bytes-chars-graphemes/README.md)
and [1.1.6 — `Vec` and `String`](../../01-foundations/06-vec-and-string-basics/README.md)

---

## Why this matters

Python gives you one way to build text and you write it everywhere:

`f"{name} — {price:.2f}"` and `"، ".join(parts)`. Both of them make a brand-new string, and you never think about it, because there is nothing else to do.

In Rust there is something else to do. That is the whole difference.

A backend builds a log line per request, a JSON body per response, an error message per failure. Millions of times a day. If every one of those carries an allocation nobody asked for, the cost turns up somewhere nobody is looking.

So this lesson does two things: it gives you the whole formatting mini-language, and next to every tool it writes down what that tool costs.

It also settles a debt. In [1.1.6](../../01-foundations/06-vec-and-string-basics/README.md) you wrote `joined` by hand, and the lesson said the real version was in 1.4.3. Here it is.

---

## The concept

### `format!` is `println!` that hands you the string

Since Phase 0 you have been writing `{name}` inside `println!`. `format!` is exactly the same language with a different destination:

```rust
let name = "ماتین";
let lesson = 3;
let line = format!("{name} is on lesson {lesson}");
println!("{line}");
```

```text
ماتین is on lesson 3
```

`format!` builds a fresh `String` and gives it to you. It estimates the size up front, so it is usually **one** allocation — but never zero. Hold onto that; it comes back at the end of the lesson.

That `{name}` inside the string is not a typographic trick: the compiler really does go looking for a variable called `name`. Which is why a typo inside the braces is a genuine "no such variable" error, and you will meet it in the errors section.

### Three ways to say which argument goes where

```rust
println!("{} / {}", "left", "right");
println!("{0} / {1} / {0}", "first", "second");
println!("{who} / {what}", who = "Matin", what = "Rust");
```

```text
left / right
first / second / first
Matin / Rust
```

The first line is **positional by order**: each `{}` takes the next argument. The second is **positional by index**, counting from zero — and because it is an index, you can use one argument twice. The third is **by name**.

And the fourth way is the one you already write: `{name}`, taken straight from the variables in scope. When the variable already has the right name, that is the most readable form there is. When it is an expression — `parts.len()`, say — it has to be a separate argument, because only a name goes inside the braces, not an expression.

### `{}` and `{:?}` ask two different questions

```rust
let title = "سلام دنیا".to_string();
println!("display: {title}");
println!("debug:   {title:?}");

let parts = vec!["one", "two"];
println!("debug:   {parts:?}");
println!("pretty:  {parts:#?}");
```

```text
display: سلام دنیا
debug:   "سلام دنیا"
debug:   ["one", "two"]
pretty:  [
    "one",
    "two",
]
```

Look at the quotes. `{}` asks "what do you look like to a user?" and the answer is the text itself, unadorned. `{:?}` asks "what do you look like to a programmer?" and the answer has quotes on it, because a programmer needs to know this is a string and where it ends.

`{:#?}` is the same thing with line breaks and indentation. For a two-element vector it is overkill; for the nested structure you build in 1.5.1 it is the difference between reading it and not.

Those two questions are two separate **traits** in Rust: `Display` behind `{}` and `Debug` behind `{:?}`. Learn the names here and move on — implementing them for your own types is [Phase 2](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md). The only part you need today is this: **some types have only one of the two.** A `Vec` has no "for a user" form at all, so `{}` on one does not compile. That is the first error in the errors section.

### Width and alignment

After the colon, a number is a **minimum width** and the character before it is the alignment:

```rust
println!("[{:>8}]", "hello");
println!("[{:<8}]", "hello");
println!("[{:^8}]", "hello");
println!("[{:*>8}]", "hello");
```

```text
[   hello]
[hello   ]
[ hello  ]
[***hello]
```

`>` is right, `<` is left, `^` is centre. Any character you put **before** the alignment marker becomes the fill character — that is what the `*` is doing.

The number is a minimum, not a maximum: something wider than the column asked for is not cut, it pushes the column out.

If you leave the alignment off, the default depends on the type:

```rust
println!("[{:8}]", "42");
println!("[{:8}]", 42);
```

```text
[42      ]
[      42]
```

The same two characters, once as text and once as a number. Text goes left, numbers go right. This is one of those things that surprises you once and then stays learned: **if the alignment matters to you, write it down.**

### What the `8` in `{:>8}` actually counts

This is where Persian has something new to say. Put three strings through one specifier:

```rust
let english = "hello";
let persian = "سلام";
let joined = "می‌شود";
println!("[{english:>8}]");
println!("[{persian:>8}]");
println!("[{joined:>8}]");
```

```text
[   hello]
[    سلام]
[  می‌شود]
```

Three spaces, four spaces, two spaces. Why?

| Text | Bytes | Chars | Letters you see | Padding added |
|---|---|---|---|---|
| `hello` | 5 | 5 | 5 | 3 |
| `سلام` | 8 | 4 | 4 | 4 |
| `می‌شود` | 13 | 6 | 5 | 2 |

From [1.4.2](../02-utf8-bytes-chars-graphemes/README.md) you know bytes and chars are not the same thing. Now add this: **`{:>8}` counts neither bytes nor what the eye sees. It counts `char`s.**

For `hello` all three numbers agree, so everything looks fine. For `سلام` the bytes are doubled but chars and visible letters still agree, so it still comes out right. For `می‌شود` it does not: between `ی` and `ش` there is a **zero-width non-joiner (ZWNJ)**, which is a full `char` and occupies no width on screen. The formatter counted it; your eye did not. One column short.

So, honestly: **`{:>8}` is the wrong tool for laying out a Persian table.** For text made only of Persian letters it happens to work, but the moment a ZWNJ or an emoji or a combining mark turns up, the count and the appearance part company. The right tool counts graphemes, which comes back in [1.4.4](../04-slicing-text-safely/README.md) — and in practice means an external crate.

### Precision: rounding numbers, cutting text

After a dot, a number is the **precision**:

```rust
println!("[{:.2}]", 1.0_f64 / 3.0);
println!("[{:>8.2}]", 1.0_f64 / 3.0);
println!("[{:.3}]", "hello");
println!("[{:.2}]", "سلام");
```

```text
[0.33]
[    0.33]
[hel]
[سل]
```

On a number it means "this many digits after the point", and it **rounds rather than truncating**. On text it means "at most this many", and it truncates.

And that truncation is **by character**. `{:.2}` on `سلام` gave `سل` — two chars, four bytes. Ask for those same two units in bytes and you get only `س`; ask for three and there is no answer at all, because byte 3 falls inside the letter `ل`. The formatter never lands you there. This matters, because it is the only **safe** way to shorten text you have so far; cutting directly with byte indices is [1.4.4](../04-slicing-text-safely/README.md).

Width and precision combine, and both can come from a variable if you put a `$` after the name:

```rust
let width = 10;
let places = 3;
println!("[{:>width$.places$}]", 2.0_f64.sqrt());
```

```text
[     1.414]
```

### Building without `format!`: `push_str`, `push`, `+`

`format!` takes a fresh buffer every time. If you are building one piece of text out of parts, it does not have to:

```rust
let mut out = String::with_capacity(32);
println!("empty  @ {:p} cap {}", out.as_ptr(), out.capacity());
out.push_str("report");
out.push('-');
out.push_str("2026");
println!("filled @ {:p} cap {} = {out}", out.as_ptr(), out.capacity());
```

```text
empty  @ 0x219accba7a0 cap 32
filled @ 0x219accba7a0 cap 32 = report-2026
```

Same address, same capacity. Three writes and **not one new allocation**, because the buffer had room from the start. `push_str` takes text and `push` takes a single `char` — the same split you saw in 1.1.6.

The `+` operator is also not what you think it is:

```rust
let mut left = String::with_capacity(64);
left.push_str("report");
println!("left   @ {:p}", left.as_ptr());
let right = "-2026".to_string();
let sum = left + &right;
println!("sum    @ {:p} = {sum}", sum.as_ptr());
```

```text
left   @ 0x219accb2360
sum    @ 0x219accb2360 = report-2026
```

The address did not change. `+` **takes ownership** of its left-hand side and reuses that buffer; it only reads the right-hand side. Which is why after `let sum = left + &right;` there is no `left` any more — that is the move from [1.2.2](../../02-ownership-and-memory/02-move-semantics/README.md).

And it is why the signature is lopsided: the left must be a `String` and the right a `&str`. Add two `String`s together and you get an error, which is the second one in the next section.

### `concat` and `join` — the debt from 1.1.6

```rust
let parts = vec!["نام".to_string(), "شهر".to_string(), "سال".to_string()];
println!("join   = {}", parts.join("، "));
println!("concat = {}", ["a", "b", "c"].concat());
```

```text
join   = نام، شهر، سال
concat = abc
```

`concat` glues the pieces together. `join` puts a separator between them — precisely the job you did in 1.1.6 with a loop and an `index > 0` test.

And it is cheaper than the hand-written version: both of them **measure the total length first** and then take one buffer of exactly that size. One allocation, no intermediate growth. You wrote it by hand once so you would know what is underneath; from here on, write `join`.

### `write!` — `format!`'s machinery aimed at your own buffer

If you are building text in pieces but each piece needs formatting, `format!` makes a throwaway buffer every time round. `write!` does not:

```rust
use std::fmt::Write;

let mut buf = String::new();
let _ = write!(buf, "{}/{}", 3, 4);
let _ = write!(buf, " = {:.2}", 3.0 / 4.0);
println!("write  = {buf}");
```

```text
write  = 3/4 = 0.75
```

Two notes, then move on:

The `use std::fmt::Write;` is compulsory; without it you get an `E0599` saying it cannot write into a `String`, along with an offer to add the import for you. The reason it needs importing at all is the thing you meet in Phase 2 under the name "trait".

And the `let _ =` is compulsory too: `write!` hands back a value saying whether the write succeeded. Writing into a `String` never fails, but Rust will not let you drop a result on the floor silently. `let _ =` means "I know, and I am ignoring it deliberately". That value, and what to do with it properly, is [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md).

### Trimming and splitting borrow

```rust
let raw = "  نام: ماتین  ";
let clean = raw.trim();
println!("[{clean}]");
println!("trim_end starts at the same byte: {}", raw.trim_end().as_ptr() == raw.as_ptr());
```

```text
[نام: ماتین]
trim_end starts at the same byte: true
```

`trim`, `trim_start` and `trim_end` make nothing. They return a `&str`: the same buffer, a narrower window. **Zero allocations.** That `true` proves it — `trim_end` only shortened the tail, so the start is still the same byte.

`split` is the same:

```rust
let line = "ماتین,رشت,۲۶";
for field in line.split(',') {
    println!("field [{field}] — {} bytes, {} chars", field.len(), field.chars().count());
}
```

```text
field [ماتین] — 10 bytes, 5 chars
field [رشت] — 6 bytes, 3 chars
field [۲۶] — 4 bytes, 2 chars
```

None of those three pieces is a new string; all three are windows into `line`. Note that `۲۶` in Persian digits is four bytes and two chars — last lesson, in practice.

### Replacing and case conversion allocate

```rust
let untouched = "abc".replace('z', "!");
println!("new buffer anyway: {}", untouched.as_ptr() != "abc".as_ptr());
println!("{} -> {}", "straße", "straße".to_uppercase());
let fa = "سلام دنیا";
println!("{} -> {}", fa, fa.to_uppercase());
println!("unchanged: {}", fa.to_uppercase() == fa);
```

```text
new buffer anyway: true
straße -> STRASSE
سلام دنیا -> سلام دنیا
unchanged: true
```

Three things in those four lines:

**`replace` always allocates**, even when it finds nothing to replace. Its answer is a `String`, and a `String` means a new buffer. If all you want to know is whether something is in there, call `contains`, which gives you a `bool` and makes nothing.

**`to_uppercase` allocates too, and the reason is worth seeing:** `straße` is six chars and `STRASSE` is seven. Case conversion can change the length of the text, so it cannot possibly be done in place. That is why it returns a `String` and not something cheaper.

**And Persian has no case.** `to_uppercase` on `سلام دنیا` gives back exactly `سلام دنیا`, and that `unchanged: true` says so. This is neither a bug nor a special case: Unicode gives every character an uppercase mapping, and for Persian letters that mapping is the letter itself. Which means you allocated, copied the whole text, and changed nothing.

> **Working rule:** if your users write Persian, `to_lowercase` for a "case-insensitive comparison" does nothing at all and costs you a copy. The normalisation you actually want is a different one — folding Arabic `ي` onto Persian `ی` and `ك` onto `ک` — and that is a job for `replace`, not for case conversion.

### The cost model, on one page

```senpai-visual
{"kind":"concept","labels":["parts","join","one buffer","String"]}
```

| What you write | What you get | Allocations |
|---|---|---|
| `trim` / `trim_start` / `trim_end` | `&str` | none |
| `split` and walking it | a `&str` per piece | none |
| `contains` / `starts_with` / `len` | a `bool` or a number | none |
| `push_str` / `push` | nothing back, room used up | only when capacity runs out |
| `+` | `String` | reuses the left buffer |
| `join` / `concat` | `String` | exactly one |
| `format!` | `String` | usually one, never zero |
| `replace` | `String` | one, even when nothing changes |
| `to_uppercase` / `to_lowercase` | `String` | one, even when nothing changes |
| `to_string` on a `&str` | `String` | one |

And one shape you should learn to recognise in code:

```rust
let mut slow = String::new();
for part in &parts {
    slow = format!("{slow}{part} ");
}

let mut fast = String::new();
for part in &parts {
    fast.push_str(part);
    fast.push(' ');
}
```

```text
slow = [نام شهر سال]
fast = [نام شهر سال]
same answer, 3 allocations against 1
```

The first version takes a new buffer every time round, copies everything built so far into it, and throws the old one away. With three pieces you cannot see the cost; with three thousand you have quadratic slowness that no log will show you.

The second gives the same answer and only allocates when the buffer fills up. If you know the rough length and write `String::with_capacity`, even that goes away.

**What about going the other way?** Turning `"42"` into a number. That can fail, and its answer is a type meaning "either a number or an error". You cannot do it properly until you have that type, so it is deliberately not here: `.parse()` belongs to [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md).

---

## Hands on

```sh
cargo run -p p1-04-03-building-and-transforming-strings --example 01-format-basics
cargo run -p p1-04-03-building-and-transforming-strings --example 02-width-and-precision
cargo run -p p1-04-03-building-and-transforming-strings --example 03-building-a-string
cargo run -p p1-04-03-building-and-transforming-strings --example 04-trim-split-replace-case
```

Then the three broken ones:

```sh
cargo run -p p1-04-03-building-and-transforming-strings --example 05-display-on-a-vec --features broken
cargo run -p p1-04-03-building-and-transforming-strings --example 06-adding-two-strings --features broken
cargo run -p p1-04-03-building-and-transforming-strings --example 07-a-name-that-is-not-there --features broken
```

Then try:

1. In `02-width-and-precision`, change the width from `8` to `12` and rerun the last three lines. Did the gap between Persian and English shrink, or stay the same?
2. In `02-width-and-precision`, put your own name in place of `hello` and see whether the columns break. If they do not, put a ZWNJ in the middle and run it again.
3. In `03-building-a-string`, drop the starting capacity from `32` to `4`. Is `filled`'s address still the same as `empty`'s?
4. In `04-trim-split-replace-case`, call `to_uppercase` on text that mixes Persian and English. Which half changes?

---

## Errors you will meet

### `E0277` — this type has no "for a user" form

```text
error[E0277]: `Vec<&str>` doesn't implement `std::fmt::Display`
 --> examples\05-display-on-a-vec.rs:9:20
  |
9 |     println!("{}", words);
  |               --   ^^^^^ `Vec<&str>` cannot be formatted with the default formatter
  |               |
  |               required by this formatting parameter
  |
  = help: the trait `std::fmt::Display` is not implemented for `Vec<&str>`
  = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead
```

**What the compiler is objecting to:** `{}` asks a value to write itself out "for a user". A number knows how; so does a string. A `Vec` does not — and that is a deliberate decision by the standard library, because there is no right answer to "how do you show a list to a user?". Commas? Newlines? Brackets? It depends on your program, not on `Vec`.

**The fix:**

```rust
println!("{words:?}");
```

**Why that's the fix:** "for a programmer" does have one right answer, and `Vec` has it. Notice that the error's last line suggested exactly this — `rustc` is handing you the answer here.

And if you really do want to show it to a user, `{:?}` is not the right answer; `join` is. Keep `{:?}` for logs and error messages.

### `E0308` — `+` wants a `&str` on its right

```text
error[E0308]: mismatched types
  --> examples\06-adding-two-strings.rs:11:26
   |
11 |     let joined = first + second;
   |                          ^^^^^^ expected `&str`, found `String`
   |
help: consider borrowing here
   |
11 |     let joined = first + &second;
   |                          +
```

**What the compiler is objecting to:** adding two strings in Rust is not symmetric. The left has to be a `String` that **gives up ownership**, and the right a `&str` that is only **read**. You gave it two `String`s.

**The fix:**

```rust
let joined = first + &second;
```

**Why that's the fix:** that one `&` is the difference between "take this" and "just look at it". The result reuses `first`'s buffer and `second` is left untouched where it was — exactly what you watched happen with the addresses in the third example.

The lopsidedness is not an accident: if `+` borrowed both sides it would have to build a third buffer. This way the worst case is one allocation and the best case is none.

And notice that the compiler's suggestion is that single character. Get into the habit of reading the `help` block; for most of this lesson's errors, the answer is written in it.

### `E0425` — a typo inside the braces

```text
error[E0425]: cannot find value `nmae` in this scope
  --> examples\07-a-name-that-is-not-there.rs:10:22
   |
10 |     println!("hello {nmae}");
   |                      ^^^^
   |
help: a local variable with a similar name exists
   |
10 -     println!("hello {nmae}");
10 +     println!("hello {name}");
   |
```

**What the compiler is objecting to:** `{nmae}` is not decorative text; it is a real variable lookup. There is no variable called `nmae`, so you get the same error you would have got by writing `nmae` outside the string.

**The fix:** spell it right. The compiler found the closest name in scope and printed the whole corrected line for you.

**Why that's the fix:** and this is the point of the lesson, not a spelling slip. In Python, `"{nmae}".format(...)` is a runtime error that might surface months later down a rarely-taken path. In Rust the same mistake does not compile. **A format string is not a string; it is code**, and the compiler reads it.

---

## Exercises

### Warm up

<details>
<summary>What does <code>format!</code> give you that <code>println!</code> doesn't?</summary>

The `String` itself. `println!` does the same work and sends the result to standard output, handing you nothing.

</details>

<details>
<summary>What does <code>{:?}</code> add to a string that <code>{}</code> doesn't?</summary>

The quotes. `{}` is the "for a user" form and `{:?}` is the "for a programmer" form, which has to show where the string ends.

</details>

<details>
<summary>What does the <code>8</code> in <code>{:>8}</code> count?</summary>

Characters. Not bytes, and not what appears on screen. For `hello` all three agree; for `می‌شود` they do not.

</details>

<details>
<summary>What's the difference between <code>{:.2}</code> on a number and on text?</summary>

On a number it means two digits after the point and it **rounds**. On text it means at most two characters and it **truncates**.

</details>

<details>
<summary>What does <code>let sum = left + &right;</code> do to <code>left</code>?</summary>

Takes ownership of it and reuses its buffer. After that line `left` no longer exists, and `right` is untouched.

</details>

<details>
<summary>Which of these doesn't allocate: <code>trim</code>, <code>replace</code>, <code>to_uppercase</code>, <code>join</code>?</summary>

`trim`. It returns a `&str` into the same buffer. The other three each build a new `String`.

</details>

<details>
<summary>What does <code>"سلام".to_uppercase()</code> give you?</summary>

`سلام`. Persian has no upper case, so the text is unchanged — but the allocation and the copy happened anyway.

</details>

### Repair

Fix all three broken examples, and for each one write a sentence on *why* that is the right fix.

1. Fix `examples/05-display-on-a-vec.rs` two ways: once with `{:?}`, and once with `join` so that the output is readable by someone who isn't a programmer.
2. Fix `examples/06-adding-two-strings.rs` two ways: once with `+`, and once without `+` at all. Which of the two allocates less?
3. Fix `examples/07-a-name-that-is-not-there.rs`. Then make the same mistake deliberately with a named argument — write `{who}` and don't supply `who`. Is the error the same one?

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-04-03-building-and-transforming-strings
```

Two of them are one line. One of them is the `joined` you wrote by hand in 1.1.6. One of them must give the same answer for Persian text as for English, and one of them must **not** — and the specification says which is which.

Read `preview` carefully: it has to count characters rather than bytes, and in this lesson you have exactly one tool that does that safely.

### Build

Write a `pub fn receipt(items: &[(String, f64)]) -> String` that produces one line per item: the item's name left-aligned in a ten-character column, the price right-aligned in an eight-character column with two decimal places, and a total line at the end.

Write it with `format!` first, line by line, collecting into a `String`.

Then count your allocations. Now rewrite it with `write!` into a single buffer and count again.

Then give it an item with a Persian name — `چای`, say — and then one containing a ZWNJ, like `آب‌میوه`. Look at the columns and write one sentence on what you saw and why.

### Challenge (optional)

**Part one.** Predict this, then run it:

```rust
let text = "می‌شود";
println!("[{:.3}]", text);
println!("{}", text.chars().count());
```

How many letters do you see? Why does that number disagree with the letter count?

**Part two.** Write a function that computes the *real* width of a Persian string — one that doesn't count zero-width characters like the ZWNJ. Start with `\u{200c}`. Then say why that still isn't a complete answer, and what you'd need for a complete one. (Hint: you met the word in 1.4.2.)

**Part three.** Run `cargo clippy` over this whole lesson. How many lints are about strings? Look up `useless_format` and `format_in_format_args` in the documentation and read them — both of them catch exactly the wasted work this lesson was about.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `format!` | like `println!` but it returns a `String` | building any text |
| indexed argument | `{0}`, `{1}` | when one value appears twice |
| named argument | `{who}` with `who = ...` | long format strings |
| inline `{name}` | taken straight from scope | the most readable form |
| `Display` | the "for a user" form, behind `{}` | your own types, Phase 2 |
| `Debug` | the "for a programmer" form, behind `{:?}` | logs and error messages |
| `{:#?}` | the same `Debug` with indentation | nested data |
| width and alignment | `{:>8}`, `{:<8}`, `{:^8}` | columns — carefully, in Persian |
| precision | `{:.2}` — rounds numbers, cuts text | money, percentages, previews |
| `push_str` / `push` | append into the buffer you have | building in pieces |
| `+` on strings | takes the left, keeps its buffer | joining two |
| `join` / `concat` | one allocation, exactly sized | a list into text |
| `write!` | `format!`'s machinery, your buffer | text built in a loop |
| ZWNJ | a character with no width | why Persian columns drift |

### What you now know

- `format!` returns a new `String` and usually costs one allocation; never zero.
- `{}` and `{:?}` ask different questions, and some types answer only the second.
- The width in `{:>8}` counts `char`s — not bytes, not what the eye sees.
- Precision rounds numbers and truncates text, and that truncation is safely by character.
- `trim` and `split` borrow; `replace` and `to_uppercase` allocate.
- `+` takes its left-hand side by value and reuses its buffer.
- `join` and `concat` measure first and then allocate once.
- `to_uppercase` does nothing at all to Persian, and you pay full price for it.

### What comes back later

- **Cutting text safely with byte indices** — [1.4.4 — Slicing text safely](../04-slicing-text-safely/README.md)
- **`{:?}` on your own types** — [1.5.1 — Structs](../../05-your-own-types/01-structs-and-methods/README.md)
- **That value `write!` returns, and `.parse()`** — [1.6.3 — `Result` and `?`](../../06-absence-and-failure/03-result-and-question-mark/README.md)
- **`Display` and `Debug` as traits, written by hand** — [Phase 2 — Defining and implementing traits](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md)
- **Those text-building loops as iterator chains** — [Phase 2 — Iterator adapters](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md)

### Can you explain?

- What does `format!` return, and how many allocations does it cost?
- What is the difference between `{}` and `{:?}`, and why doesn't `{}` work on a `Vec`?
- What does the `8` in `{:>8}` count, and why doesn't it always come out right for Persian?
- Which string operations don't allocate? Name three.
- Why does `+` want a `String` on the left and a `&str` on the right?
- What does `"سلام".to_uppercase()` give you, and what did it cost?

---

## Going further

- [`std::fmt`](https://doc.rust-lang.org/std/fmt/) — the complete reference for the formatting language. Read it top to bottom once so you know what exists.
- [The Rust Book — Strings](https://doc.rust-lang.org/book/ch08-02-strings.html) — the same ground, officially.
- [`std::string::String`](https://doc.rust-lang.org/std/string/struct.String.html) — the method list. Look down the return-type column: everywhere it says `String`, that's an allocation.
- [`clippy::format_in_format_args`](https://rust-lang.github.io/rust-clippy/master/#format_in_format_args) — the lint that catches a `format!` inside a `format!`.
