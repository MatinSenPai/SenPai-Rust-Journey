# 1.4.4 — Slicing text safely

## At a glance

After this lesson you can:

- Look at a `&text[a..b]` and say *before running it* whether it will panic.
- Read `end byte index N is not a char boundary` and name the exact letter you cut in half.
- Write a `truncate_to_chars` that cannot panic on any input, in any language.
- Choose between `[..]`, `.get(..)` and `floor_char_boundary` for a given job.

**Time:** ~60 minutes · **Prerequisites:**
[1.4.2 — UTF-8: bytes, chars, graphemes](../02-utf8-bytes-chars-graphemes/README.md) ·
[1.4.3 — Building and transforming strings](../03-building-and-transforming-strings/README.md)

---

## Why this matters

This is where the whole text module pays off, and it is probably the most immediately useful lesson in Phase 1 for you.

You know the scenario. A ticket comes in: "card titles are capped at 20 characters, the rest gets an ellipsis." Somebody writes `&title[..20]`. The tests go green against `"Fullmetal Alchemist: Brotherhood"`. It ships. A week later the service falls over on a Persian title — not once, but on roughly half of all Persian titles — and the log says:

```text
end byte index 20 is not a char boundary; it is inside 'ی' (bytes 19..21 of string)
```

In an English-speaking codebase this bug is effectively invisible, because in ASCII the byte count and the letter count are the same number. It is not invisible to you: you have Persian users, and in Persian those two numbers are never the same.

The good news is that Rust does not let this pass quietly. Rather than hand you half a letter and let your user see `Ø±`, it stops the program. That strictness is annoying right up until the day you realise the alternative was corrupted data in your database.

---

## The concept

### `&text[a..b]` counts bytes, not letters

You know the range syntax from slices: `&v[1..4]`. The same syntax works on text — but those two numbers are **byte indices**.

```rust
let english = "programming";
println!("{:?}", &english[0..7]);
println!("{} chars, {} bytes", english.chars().count(), english.len());
```

```text
"program"
11 chars, 11 bytes
```

Seven bytes came out as seven letters. The two numbers agreed, and that is exactly what gets this bug through review: in English you are never forced to tell them apart.

### The same cut, in Persian

```rust
let persian = "برنامه‌نویسی";
println!("{} chars, {} bytes", persian.chars().count(), persian.len());
println!("persian: {:?}", &persian[0..7]);
```

```text
12 chars, 25 bytes

thread 'main' (15184) panicked at phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\04-cut-in-half.rs:15:39:
end byte index 7 is not a char boundary; it is inside 'ا' (bytes 6..8 of string)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

Twelve letters, twenty-five bytes. And byte 7 is not a legal place to cut at all.

(The number in brackets after `main` is the thread id and changes on every run; the rest of the message is always exactly this.)

### Read the panic line by line

The message tells you three things, and all three are worth having:

| Part of the message | What it means |
|---|---|
| `end byte index 7` | the end of your range, byte number 7 |
| `is not a char boundary` | that is not where a letter starts |
| `it is inside 'ا' (bytes 6..8 of string)` | it is inside this letter, which occupies bytes 6 to 8 |

It even shows you the victim. Here is the byte map of the start of that string:

```text
byte index:  0    1    2    3    4    5    6    7    8
             +---------+---------+---------+---------+
             | letter1 | letter2 | letter3 | letter4 |
             +---------+---------+---------+---------+
                                                ^
                            byte 7 sits inside letter 4
```

Every Persian letter took two bytes, so the only legal boundaries are the even numbers: 0, 2, 4, 6, 8. Every odd number is inside a letter.

**The rule to memorise:** `[a..b]` on a string is legal only when `a` and `b` are both char boundaries. Otherwise you get a panic — not broken output, not a warning. A stop.

### `is_char_boundary` — asking instead of guessing

Before you cut, you can ask:

```rust
let persian = "برنامه‌نویسی";
for index in 0..=8 {
    println!("{index}: {}", persian.is_char_boundary(index));
}
```

```text
0: true
1: false
2: true
3: false
4: true
5: false
6: true
7: false
8: true
```

`is_char_boundary` never panics; an index past the end simply gives `false`. Two facts always hold: `0` is always a boundary, and `text.len()` is always a boundary — even for the empty string.

### `char_indices` — the list of real boundaries

Instead of guessing, take the list. `char_indices`, which you met in [1.4.2](../02-utf8-bytes-chars-graphemes/README.md), gives exactly that: the byte index each character starts at.

```rust
for (index, character) in "می‌روم".char_indices() {
    println!("{index:>3} {character:?}  {} bytes", character.len_utf8());
}
```

```text
  0 'م'  2 bytes
  2 'ی'  2 bytes
  4 '\u{200c}'  3 bytes
  7 'ر'  2 bytes
  9 'و'  2 bytes
 11 'م'  2 bytes
```

That `'\u{200c}'` is the zero-width non-joiner — the character you type in "می‌روم" between "می" and "روم". It is invisible, it takes **three** bytes where the surrounding letters take two, and that is precisely why Persian byte arithmetic is not even reliably "times two".

### `.get(a..b)` — a cut that answers instead of dying

`.get()` makes the same cut, but an illegal one does not kill the program:

```rust
let persian = "برنامه‌نویسی";
println!("{:?}", persian.get(0..7));
println!("{:?}", persian.get(0..6));
println!("{:?}", persian.get(0..7).unwrap_or(""));
```

```text
None
Some("برن")
""
```

That `Some`/`None` is an `Option<&str>`: a type that says "there may be a `&str` here, and there may not". It is the same `.get()` you met on arrays in [1.1.3](../../01-foundations/03-compound-types-and-destructuring/README.md), keeping the same promise.

Two ways of using it are enough for today. The first is `unwrap_or`: "give me the answer, or this if there isn't one". The second is `if let`:

```rust
if let Some(piece) = persian.get(0..6) {
    println!("cut: {piece:?}");
} else {
    println!("byte 6 is not a legal cut");
}
```

```text
cut: "برن"
```

`Option` gets its full lesson in [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md), and `if let` gets one of its own in [module 1.5](../../05-your-own-types/README.md). This much is all we need here.

> `&text[a..b]` and `text.get(a..b)` do the same job. The difference is that the first kills the program on bad input and the second answers. Choosing between them is choosing between "this input can never be bad" and "this input might be".

### `floor_char_boundary` and `ceil_char_boundary` — snapping to the nearest legal cut

Sometimes you don't want to reject the bad cut; you want the nearest *legal* one. "At most 20 bytes", for a database column, say.

```rust
let text = "می‌روم";
println!("{} {}", text.floor_char_boundary(5), text.ceil_char_boundary(5));
println!("{:?}", &text[..text.floor_char_boundary(5)]);
```

```text
4 7
"می"
```

`floor_char_boundary` is the largest boundary at or below your number and `ceil_char_boundary` is the smallest at or above it. Byte 5 is inside the zero-width non-joiner (bytes 4 to 7), so one steps back to 4 and the other forward to 7. Neither panics, and both clamp to `text.len()` for a number past the end.

Both have been **stable since 1.91**, and this repository's toolchain (rustc 1.97.0) has them. If you are stuck on an older compiler, the manual equivalent is three lines:

```rust
let mut cut = 5;
while !text.is_char_boundary(cut) {
    cut -= 1;
}
println!("{cut} -> {:?}", &text[..cut]);
```

```text
4 -> "می"
```

That loop always terminates, because byte 0 is always a boundary. The worst case is three steps back, since no UTF-8 character is longer than four bytes.

### `split_at` breaks in exactly the same way

```rust
let salam = "سلام";
let (head, tail) = salam.split_at(3);
```

```text
thread 'main' (28348) panicked at C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\str\mod.rs:846:21:
end byte index 3 is not a char boundary; it is inside 'ل' (bytes 2..4 of string)
```

Look at that file path: the panic points **into the standard library**, not at your line. On a large project that means hunting for which `split_at` it was. The safe version follows the `.get()` pattern:

```rust
println!("{:?}", salam.split_at_checked(3));
println!("{:?}", salam.split_at_checked(4));
```

```text
None
Some(("سل", "ام"))
```

### "At most N characters", done properly

Now the toolkit is complete. To cut to a number of **characters** you need the byte index of character number N — and `char_indices` is exactly that:

```rust
fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}
```

```text
truncate_to_chars("برنامه\u{200c}نویسی سیستمی", 8) -> "برنامه\u{200c}ن"
truncate_to_chars("Rust systems", 8)              -> "Rust sys"
truncate_to_chars("سلام", 8)                      -> "سلام"
```

Three things in nine lines:

1. The `index` we return came out of `char_indices` itself, so it **is** a boundary. The slice can never panic.
2. If the loop runs to the end, the text was shorter than `max_chars`, so we return all of `text`. No counting pass needed first.
3. `max_chars == 0` returns on the very first turn and gives `""`; empty text never enters the loop at all. Both edge cases are free.

The return type is `&str`, not `String` — nothing is allocated. This function hands back a smaller window onto the same buffer.

### …and an ellipsis, only if you actually cut

Adding an ellipsis to text that was never shortened is a small bug that is absolutely everywhere:

```rust
fn with_ellipsis(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    format!("{}…", truncate_to_chars(text, max_chars))
}
```

```text
"برنامه\u{200c}ن…"
"سلام"
""
```

The test is on `chars().count()`, not `len()`. Use `len()` and "سلام" — four letters, eight bytes — suddenly counts as "shortened" against a limit of 5 and picks up an ellipsis that is a lie.

And that `…` is one character (U+2026), not three dots. Three dots is three characters and spends three times the budget.

### Why "max 20 characters" is a real production bug

Now watch the wrong version on real data. This is `card_title`, written the way it actually gets written:

```rust
fn card_title(title: &str) -> &str {
    if title.len() <= 20 {
        return title;
    }
    &title[..20]
}
```

```text
32 chars in, 20 chars out -> "Fullmetal Alchemist:"
14 chars in, 11 chars out -> "حمله به تای"

thread 'main' (2676) panicked at phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\05-max-twenty-characters.rs:13:11:
end byte index 20 is not a char boundary; it is inside 'ی' (bytes 19..21 of string)
```

Three rows, three different behaviours:

| Title | chars | bytes | `&title[..20]` |
|---|---|---|---|
| `Fullmetal Alchemist: Brotherhood` | 32 | 32 | 20 letters — correct |
| `حمله به تایتان` | 14 | 26 | 11 letters — **silently wrong** |
| `برنامه‌نویسی سیستمی با Rust` | 27 | 48 | **panic** |
| `شکارچی شیاطین` | 13 | 25 | panic (never reached) |

The second row is worse than the third. The third row makes noise; the second quietly shows eleven letters where twenty were promised, and nobody finds out until a user complains.

The arithmetic is simple. Persian letters are two bytes in UTF-8, so on all-Persian text the legal boundaries are the even bytes, and a fixed odd number like 19 or 21 always breaks. Even an even number is not safe, because one space (one byte) or one zero-width non-joiner (three bytes) shifts the whole pattern. In practice, on real Persian input, a fixed byte cut panics about half the time.

The same problem hits your database column, only more quietly:

```text
20 chars = 20 bytes     (Fullmetal Alchemist:)
19 chars = 38 bytes     (برنامه‌نویسی سیستمی)
```

A `VARCHAR(20)` that counts bytes is about ten letters for a Persian user.

```senpai-visual
{"kind":"concept","labels":["byte index","boundary?","snap or ask","safe cut"]}
```

---

## Hands on

```sh
cargo run -p p1-04-04-slicing-text-safely --example 01-bytes-not-characters
cargo run -p p1-04-04-slicing-text-safely --example 02-where-the-boundaries-are
cargo run -p p1-04-04-slicing-text-safely --example 03-safe-cuts
```

Then the four broken ones. They sit behind a feature so that running one is a deliberate act:

```sh
cargo run -p p1-04-04-slicing-text-safely --example 04-cut-in-half --features broken
cargo run -p p1-04-04-slicing-text-safely --example 05-max-twenty-characters --features broken
cargo run -p p1-04-04-slicing-text-safely --example 06-get-returns-an-option --features broken
cargo run -p p1-04-04-slicing-text-safely --example 07-indexing-by-number --features broken
```

Then try:

1. In `04-cut-in-half`, change `0..7` to `0..40`. The panic message changes — how?
2. In `04-cut-in-half`, change `0..7` to `0..6`. What gets printed now, and how many letters is it?
3. In `02-where-the-boundaries-are`, change the text to `"سلام"`. Which indices are boundaries now, and why is there no odd number this time?
4. In `05-max-twenty-characters`, delete the third title and run it again. The program now runs to the end — but the output is still wrong. Where?

---

## Errors you will meet

### Panic — `end byte index N is not a char boundary`

```text
english: "program"

thread 'main' (15184) panicked at phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\04-cut-in-half.rs:15:39:
end byte index 7 is not a char boundary; it is inside 'ا' (bytes 6..8 of string)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What it is objecting to:** you asked to cut the string at byte 7, and byte 7 is in the middle of a two-byte letter. If Rust obeyed, the result would be a `&str` that is not valid UTF-8 — and that is the entire guarantee the `&str` type makes. So it cannot obey.

**The fix, depending on what you meant:**

| What you wanted | Write |
|---|---|
| to decide for myself when the cut is bad | `text.get(a..b)` |
| the nearest legal cut without exceeding the budget | `&text[..text.floor_char_boundary(b)]` |
| N characters, not N bytes | `truncate_to_chars(text, n)` |

**Why that's the fix:** all three turn bad input into an explicit decision. `[a..b]` is the only option that defers the decision to run time, and run time's decision is to stop.

Note that this is a panic, not a compile error. The compiler cannot prevent it, because `b` is usually not known until the program runs. The difference between "panic" and "an error you return" is [1.6.4](../../06-absence-and-failure/04-panic-vs-result/README.md).

### Panic — `end byte index N is out of bounds`

Change that `7` in `04-cut-in-half` to `40`:

```text
english: "program"

thread 'main' (11808) panicked at phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\04-cut-in-half.rs:15:39:
end byte index 40 is out of bounds for string of length 25
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What it is objecting to:** this one has nothing to do with char boundaries. The string is 25 bytes and you asked for byte 40.

**The fix:** `.get()` again — it returns `None` for this case too — or `floor_char_boundary`, which clamps to `text.len()` for a number past the end instead of panicking.

**Why it matters:** these two messages are completely different and confusing them wastes time. `not a char boundary` means "your number is in range but in a bad place"; `out of bounds` means "your number is simply too big". The first is a Unicode problem, the second is an arithmetic one.

### `E0308` — `.get()` gives you an `Option`, not a `&str`

```text
error[E0308]: mismatched types
  --> phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\06-get-returns-an-option.rs:11:23
   |
11 |     let piece: &str = persian.get(0..6);
   |                ----   ^^^^^^^^^^^^^^^^^ expected `&str`, found `Option<&str>`
   |                |
   |                expected due to this
   |
   = note: expected reference `&str`
                   found enum `Option<&str>`
help: consider using `Option::expect` to unwrap the `Option<&str>` value, panicking if the value is an `Option::None`
   |
11 |     let piece: &str = persian.get(0..6).expect("REASON");
   |                                        +++++++++++++++++
```

**What it is objecting to:** the first thing everybody does after their first boundary panic is swap `&text[a..b]` for `text.get(a..b)` and expect the same type back. It isn't. `.get()` is safe *precisely because* it hands you a different type: one with "there might be nothing" built into it.

**The fix:** one of these two, depending on how you want the absence handled:

```rust
let piece = persian.get(0..6).unwrap_or("");

if let Some(piece) = persian.get(0..6) {
    println!("{piece}");
}
```

```text
برن
```

**Why that's the fix:** the `help` the compiler offers — `.expect("REASON")` — does compile, but it is usually wrong here: it gives you back exactly the panic you were escaping, with a message you wrote yourself. `.expect()` belongs where you genuinely believe `None` is impossible. This is not that place.

### `E0277` — `str` cannot be indexed by a number

The second thing everybody does: give up on ranges and use a single index.

```text
error[E0277]: the type `str` cannot be indexed by `{integer}`
   --> phase1-fundamentals\04-text-and-strings\04-slicing-text-safely\examples\07-indexing-by-number.rs:11:25
    |
 11 |     let third = persian[2];
    |                         ^ string indices are ranges of `usize`
    |
    = help: the trait `SliceIndex<str>` is not implemented for `{integer}`
    = note: you can use `.chars().nth()` or `.bytes().nth()`
            for more information, see chapter 8 in The Book: <https://doc.rust-lang.org/book/ch08-02-strings.html#indexing-into-strings>
```

(The compiler prints a few more lines after this showing that `usize` *is* a valid index for ordinary slices; those lines point at files inside your own toolchain.)

**What it is objecting to:** what should `persian[2]` give you? A byte? A character? In ASCII those are the same thing and the question is meaningless; in UTF-8 they are two different answers, and Rust will not guess. So the operator simply does not exist for `str`. You saw this in [1.1.6](../../01-foundations/06-vec-and-string-basics/README.md) too.

**The fix:** what the message itself says — `.chars().nth(2)` for the third character, `.bytes().nth(2)` for the third byte. Both give you an `Option`, because the string might not be that long.

**Why that's the fix:** being forced to choose between `chars` and `bytes` is the teaching. Every time you hit this error it means you were asking a question whose answer depends on the language of the text.

---

## Exercises

### Warm up

No typing required. Answer, then open.

<details>
<summary>What is <code>"سلام".len()</code>?</summary>

8. Four letters, two bytes each. `len()` counts bytes.

</details>

<details>
<summary>What does <code>&"سلام"[0..2]</code> give? And <code>&"سلام"[0..3]</code>?</summary>

The first gives `"س"`. The second panics: byte 3 is inside the second letter.

</details>

<details>
<summary>What does <code>"سلام".get(0..3)</code> give?</summary>

`None`. The same illegal cut, but this time it answers instead of killing the program.

</details>

<details>
<summary>What are <code>"سلام".floor_char_boundary(3)</code> and <code>ceil_char_boundary(3)</code>?</summary>

2 and 4. The boundaries either side of byte 3.

</details>

<details>
<summary>Is <code>text.len()</code> always a legal boundary? What about <code>0</code>?</summary>

Yes, both. Even for the empty string, where they are both zero.

</details>

<details>
<summary>How often does a fixed byte cut like <code>&title[..20]</code> panic on a Persian title?</summary>

About half the time. Persian letters are two bytes, so the boundaries alternate; spaces and zero-width non-joiners shift that pattern, and the result is effectively a coin toss.

</details>

<details>
<summary>Why is the "silently wrong" row worse than the "panic" row?</summary>

Because you see the panic and fix it the same day. The silent cut carries incomplete data into your database and in front of your users, and stays there for months.

</details>

### Repair

Fix `examples/04-cut-in-half.rs` **three** ways, without changing what the program means:

1. With `.get()` and a fallback value.
2. With `floor_char_boundary`.
3. With `char_indices`, using neither of the other two.

Then say which is the right one for a "card title" and why the question cannot be answered until you know where that `7` came from.

Fix `examples/06-get-returns-an-option.rs` two ways: once with `unwrap_or`, once with `if let`. Then try the third version the compiler suggests (`.expect(...)`) and say why it is a poor choice here.

Fix `examples/07-indexing-by-number.rs` so that it prints the third character, not the third byte.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-04-04-slicing-text-safely
```

The tests cover Persian, English, mixed text, the empty string and both ends of every range. None of these five functions may panic on any input — and two of the tests check exactly that, across every combination.

Read `truncated_with_ellipsis` carefully: the difference between "4 characters of a 4-character text" and "4 characters of a 9-character text" is the whole exercise.

### Build

Write a `pub fn fits_in(text: &str, max_chars: usize) -> String` that does the job of `truncated_with_ellipsis` with one difference: **the ellipsis counts against the budget.** The result is never, under any circumstances, more than `max_chars` characters.

Decide for yourself what should happen for `max_chars` of 0 and 1, and write your decision into the doc comment in one sentence.

Then measure: for a `max_chars` of 20, how many bytes does the result take for an English title, and how many for a Persian one? That ratio is what turns a `VARCHAR(20)` into half a field for a Persian user.

### Challenge (optional)

**Part one — the dangling non-joiner.** Run this:

```rust
let word = "می‌روم";
println!("{:?}", truncate_to_chars(word, 3));
```

The first three characters are "م", "ی" and the zero-width non-joiner itself. The result is perfectly valid as far as Rust is concerned — but what does it look like to a Persian reader? Write a rule that strips a dangling joiner.

**Part two — characters versus what the user sees.** A `char` still is not what a user calls "one letter". Try `truncate_to_chars("👨‍👩‍👧", 1)` and look at what comes out. The right unit here is a **grapheme**, which std does not know about at all and which needs an outside crate such as `unicode-segmentation`. [1.4.2](../02-utf8-bytes-chars-graphemes/README.md) started this argument, and it does not finish here either.

**Part three — reaching forward.** Write `truncate_to_chars` as a single line using `.char_indices()` and iterator adapters. Those tools belong to [Phase 2](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md), so getting stuck is fine — but when you reach that lesson, come back and open this one again.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| char boundary | a byte index where a character starts | the only legal place to cut |
| `&text[a..b]` | a byte slice; panics on a bad boundary | when you know both ends are boundaries |
| `.get(a..b)` | the same cut, as an `Option<&str>` | when the input comes from outside |
| `is_char_boundary` | boundary or not; never panics | checking before you cut |
| `char_indices` | the list of (byte index, character) | finding the boundary of character N |
| `floor_char_boundary` | nearest boundary, downwards | a byte budget you must not exceed |
| `ceil_char_boundary` | nearest boundary, upwards | when you'd rather keep the whole letter |
| `split_at_checked` | the non-panicking `split_at` | cutting text in two |
| ZWNJ | the invisible three-byte `U+200C` | what breaks Persian byte arithmetic too |

### What you now know

- The two numbers in `&text[a..b]` are byte indices, not letter positions.
- A slice is legal only when both ends are char boundaries; otherwise you get a panic, not broken output.
- The panic message names the letter you hit and the byte range it occupies.
- `is_char_boundary`, `char_indices`, `.get()`, `floor_char_boundary` and `split_at_checked` never panic.
- `.get()` returns an `Option<&str>`, and that difference in type is what makes it safe.
- Cutting to N characters means taking the byte index of character N out of `char_indices`.
- An ellipsis is added only when something was actually removed, and `…` is one character, not three.
- "Max 20 characters" implemented with bytes gives a Persian user either half a string or a panic.

### What comes back later

- **`Option` in full, with `match` and all of its methods** — [1.6.1 — `Option` and null safety](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **`if let`, which you met here only as far as you needed it** — [module 1.5 — Your own types](../../05-your-own-types/README.md)
- **Panicking versus returning an error** — [1.6.4 — Panic or `Result`](../../06-absence-and-failure/04-panic-vs-result/README.md)
- **Iterator adapters, which make these functions one-liners** — [Phase 2 — Iterator adapters](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md)
- **Graphemes, and the outside crate for counting what a user sees** — [1.4.2 — UTF-8](../02-utf8-bytes-chars-graphemes/README.md)

### Can you explain?

- What do the two numbers in `&text[2..6]` count?
- Why does Rust stop the program instead of returning half a letter?
- What do the three parts of `end byte index 7 is not a char boundary; it is inside 'ا' (bytes 6..8 of string)` each tell you?
- The difference between `&text[a..b]` and `text.get(a..b)`, in one sentence?
- What do `floor_char_boundary` and `ceil_char_boundary` give for byte 5 of "می‌روم", and why?
- Why must you go through `char_indices` to cut at N characters, instead of computing it from `len()`?
- Why is a silent, wrong cut worse than a panic?

---

## Going further

- [`str::get`](https://doc.rust-lang.org/std/primitive.str.html#method.get) — its documentation has the exact "when does it return `None`" table.
- [`str::floor_char_boundary`](https://doc.rust-lang.org/std/primitive.str.html#method.floor_char_boundary) — stable since 1.91.
- [The Rust Book — chapter 8, strings](https://doc.rust-lang.org/book/ch08-02-strings.html#indexing-into-strings) — the same argument, officially. It is the link the compiler itself gives you in `E0277`.
- [`unicode-segmentation`](https://docs.rs/unicode-segmentation) — for when "one letter" means what the user sees, not what `char` says.
