# 1.4.2 — UTF-8: bytes, chars, graphemes

## At a glance

After this lesson you can:

- Say how many bytes any piece of text takes — English, Persian, emoji — and why, without running it.
- Choose between `len()`, `.chars().count()` and grapheme counting, and say which one the person filling in your signup form calls a "character".
- Read a `not a char boundary` panic and say exactly what the code did wrong.

**Time:** ~50 minutes · **Prerequisites:**
[1.4.1 — `String` vs `&str`](../01-string-vs-str/README.md) ·
[1.1.6 — `Vec` and `String`](../../01-foundations/06-vec-and-string-basics/README.md)

---

## Why this matters

[1.1.6](../../01-foundations/06-vec-and-string-basics/README.md) showed you a number and then walked away from it: `"سلام".len()` is 8 and `"سلام".chars().count()` is 4. It said this lesson would open it up. Here it is.

This is probably the most important lesson in the whole course for you. Not because Rust is harder here than elsewhere — because you write Persian.

Every bug in this lesson is **invisible** in code written by an English speaker. `len()` on `hello` is 5, and you count five letters in `hello`, so everything looks right. Their test passes, their review passes, it ships, and the first person to hit it is a user called `متین`.

A list of bugs that really do happen in a Persian-language backend:

- "maximum 20 characters" that is really "maximum ten letters".
- Truncating a message at byte 100, landing mid-letter, and crashing the process.
- Report columns that line up in English and collapse in Persian.
- A search that can't find `کیان` because the user typed it on an Arabic keyboard.

None of these are "Rust problems". Rust is just the only widely-used language that makes you face them before production does.

---

## The concept

### One letter, how many bytes?

Unicode gives every character a number. That number is called a **Unicode scalar value**, written as `U+…`. UTF-8 is one way of writing that number down as bytes — and it does not spend the same number of bytes on every scalar:

```rust
println!("scalar    utf-8    name                                it looks like");
show('a', "LATIN SMALL LETTER A");
show('س', "ARABIC LETTER SEEN");
show('م', "ARABIC LETTER MEEM");
show('۵', "EXTENDED ARABIC-INDIC DIGIT FIVE");
show('\u{200c}', "ZERO WIDTH NON-JOINER");
show('\u{064e}', "ARABIC FATHA");
show('€', "EURO SIGN");
show('🌸', "CHERRY BLOSSOM");
```

```text
scalar    utf-8    name                                it looks like
U+0061    1 byte   LATIN SMALL LETTER A                a
U+0633    2 bytes  ARABIC LETTER SEEN                  س
U+0645    2 bytes  ARABIC LETTER MEEM                  م
U+06F5    2 bytes  EXTENDED ARABIC-INDIC DIGIT FIVE    ۵
U+200C    3 bytes  ZERO WIDTH NON-JOINER               ‌
U+064E    2 bytes  ARABIC FATHA                        َ
U+20AC    3 bytes  EURO SIGN                           €
U+1F338   4 bytes  CHERRY BLOSSOM                      🌸
```

Take three things away from that table:

1. ASCII letters are one byte. **One.**
2. Persian and Arabic letters — and Persian digits — are two bytes. **Every scalar in the Arabic block is.**
3. An emoji is four bytes, and the zero-width non-joiner, the character that displays nothing at all, is three.

Look at the `U+200C` row again. There is nothing in the last column, because the **ZWNJ (zero-width non-joiner)** has no width. It still takes three bytes of your database and it still counts in every count you make. Every time you type `می‌روم` or `بک‌اند` there is one of those in your text.

### The rule UTF-8 follows

None of this is arbitrary. UTF-8 has one simple rule, and you can look straight at it:

```rust
bytes_of('a');
bytes_of('س');
bytes_of('€');
bytes_of('🌸');
```

```text
  U+0061 ->  61 (01100001)
  U+0633 ->  D8 (11011000)  B3 (10110011)
  U+20AC ->  E2 (11100010)  82 (10000010)  AC (10101100)
  U+1F338 ->  F0 (11110000)  9F (10011111)  8C (10001100)  B8 (10111000)
```

The leading bits of each byte announce its job:

| First byte starts with | Meaning | Total length |
|---|---|---|
| `0…` | a one-byte scalar (plain ASCII) | 1 byte |
| `110…` | the start of a two-byte scalar | 2 bytes |
| `1110…` | the start of a three-byte scalar | 3 bytes |
| `11110…` | the start of a four-byte scalar | 4 bytes |
| `10…` | a **continuation byte** — the second half of something | — |

Look at `D8`: `11011000`, so "two-byte scalar, starting here". Look at `B3`: `10110011`, so "continuation". From any byte in the middle of any text you can tell whether you are standing on a letter or inside one — and that is exactly what Rust uses to protect you.

Two consequences worth memorising:

- **ASCII is UTF-8.** Every ASCII file is already valid UTF-8. That is why this encoding won.
- **No scalar's encoding is a substring of another's.** A continuation byte can never be mistaken for a starting byte, so you cannot get lost in the middle of a string.

```senpai-visual
{"kind":"concept","labels":["bytes","scalars","grapheme clusters","what the user counts"]}
```

### A `char` is four bytes, always

This is where people get confused, so plainly: a `char` in Rust holds **exactly one Unicode scalar value**, and in memory it is always four bytes — whether it is `'a'` or `'🌸'`:

```rust
println!("a char in memory: {} bytes, always", size_of::<char>());
```

```text
a char in memory: 4 bytes, always
```

So there are two different numbers and both are true:

| Question | Answer for `'س'` | How to ask |
|---|---|---|
| How much space does this `char` take in memory? | 4 bytes | `size_of::<char>()` |
| How many bytes will it take inside a `String`? | 2 bytes | `letter.len_utf8()` |

A `char` is a fixed four-byte box so that random access to it is cheap. A `String` is packed tight, because it is meant to be stored and shipped. **`char` is the in-memory representation; UTF-8 is the storage representation.** Don't run the two together.

### `len()` counts bytes, `.chars().count()` counts scalars

Now the text from 1.1.6, with more company:

```rust
report("hello");
report("سلام");
report("متین");
report("سلام، من متین هستم.");
report("Rust برای بک‌اند");
report("🌸");
report("سلام 🌸");
```

```text
bytes  chars   text
    5      5   hello
    8      4   سلام
    8      4   متین
   34     19   سلام، من متین هستم.
   27     16   Rust برای بک‌اند
    4      1   🌸
   13      6   سلام 🌸
```

For `hello` both numbers are 5. For every other row they aren't. **The gap is exactly the number of continuation bytes** — every byte is either the start of a scalar or a continuation of one, so counting scalars is counting starts.

Look at `سلام، من متین هستم.`: 34 bytes, 19 scalars. If you have a database column that accepts "20 characters" and you enforce that 20 in bytes, that sentence does not fit. A perfectly ordinary short sentence.

> **Why did Rust give the short name to bytes?** Because `len()` is instant: a `String` already knows its byte count. Counting scalars means walking the whole thing and decoding as it goes, and that costs something. Rust gives the short name to the cheap operation and makes you ask for the expensive one out loud.

### `.char_indices()` — which byte each scalar starts at

`.chars()` gives you the scalars. `.char_indices()` gives you the same scalars **plus the byte each one starts at** — and that number is what every slicing problem is really about:

```rust
for (at, letter) in text.char_indices() {
    let width = letter.len_utf8();
    println!("    starts at byte {at}, {width} byte(s) wide     {letter}");
}
```

```text
    starts at byte 0, 2 byte(s) wide     س
    starts at byte 2, 2 byte(s) wide     ل
    starts at byte 4, 2 byte(s) wide     ا
    starts at byte 6, 2 byte(s) wide     م
```

0, 2, 4, 6. **No scalar starts at byte 1, 3, 5 or 7.** Those are second halves.

And the same loop over `"من 🌸"`, where the widths are not uniform:

```text
    starts at byte 0, 2 byte(s) wide     م
    starts at byte 2, 2 byte(s) wide     ن
    starts at byte 4, 1 byte(s) wide      
    starts at byte 5, 4 byte(s) wide     🌸
```

Two bytes, two bytes, one byte for the space, four for the flower. There is no simple pattern to compute from; you have to read the text.

### `.bytes()` and `.as_bytes()` — the raw view

Sometimes bytes are genuinely what you want: writing to a socket, hashing, measuring real size. There are two ways:

```rust
print!("  as_bytes() ->");
for byte in text.as_bytes() {
    print!(" {byte:02X}");
}
```

```text
  as_bytes() -> D8 B3 D9 84 D8 A7 D9 85
```

- `.as_bytes()` hands over the whole buffer at once as a `&[u8]`. It costs nothing — those are the bytes that were already there, seen through a different type.
- `.bytes()` walks the same bytes one at a time, for when you only want to pass over them.

Neither decodes anything and neither copies anything. `.chars()` is the one that does work: it reads bytes and builds scalars out of them.

### Which bytes are boundaries

Rust has an official name for "the start of a scalar": a **char boundary**. You can ask directly:

```rust
print!("  is_char_boundary:");
for at in 0..=text.len() {
    print!(" {at}={}", text.is_char_boundary(at));
}
```

```text
  is_char_boundary: 0=true 1=false 2=true 3=false 4=true 5=false 6=true 7=false 8=true
```

For `"سلام"` only the even bytes are boundaries. For `"من 🌸"` the pattern breaks down:

```text
  is_char_boundary: 0=true 1=false 2=true 3=false 4=true 5=true 6=false 7=false 8=false 9=true
```

This is exactly what gets checked when you write `&text[a..b]`. If `a` or `b` isn't a boundary the program **panics** — it does not hand you broken text. That panic is in the next section, and safe slicing is the subject of [1.4.4](../04-slicing-text-safely/README.md).

### Where "how many characters" gets away from you

So far `.chars().count()` has been the answer to "how many letters?". Now the case where it isn't:

```rust
println!("bytes  chars  seen   what it is                    text");
show("a plain Persian word", "سلام", 4);
show("the same word with a fatha", "سَلام", 4);
show("a word with a ZWNJ in it", "می‌روم", 5);
show("e plus a combining acute", "e\u{301}", 1);
show("the flag of Iran", "🇮🇷", 1);
show("a family emoji", "👨‍👩‍👧", 1);
```

```text
bytes  chars  seen   what it is                    text
    8      4     4   a plain Persian word          سلام
   10      5     4   the same word with a fatha    سَلام
   13      6     5   a word with a ZWNJ in it      می‌روم
    3      2     1   e plus a combining acute      é
    8      2     1   the flag of Iran              🇮🇷
   18      5     1   a family emoji                👨‍👩‍👧
```

The `seen` column is what a person counts. The `chars` column is what `.chars().count()` says. From the second row on, they disagree:

- `سَلام` still has four letters but five scalars: that fatha is a **combining mark** — a scalar of its own that rides on top of the letter before it. All vowelled Persian text is like this.
- `می‌روم` has five letters and six scalars, because the ZWNJ is a scalar too.
- The flag of Iran is one picture and two scalars (two "regional indicators"); the family emoji is one picture and five.

The unit a person calls "one character" has its own name: a **grapheme cluster** — one or more scalars that together make one visible thing. Rust's standard library does not count them, because the rules need multi-megabyte Unicode tables that change every year. The `unicode-segmentation` crate does, adding a `.graphemes(true)` method to `&str`. **We are not adding it in this lesson** — adding a dependency is the subject of [Phase 2 — cargo features](../../../phase2-intermediate/08-rust-toolbox/03-cargo-features/README.md). For now it is enough to know that this question has no easy answer in `std`, and to know its name.

### Two spellings of the same thing

Counting isn't the only problem. Two pieces of text can look completely identical and not be equal:

```rust
let decomposed = "e\u{301}";
let precomposed = "é";
println!("decomposed == precomposed: {}", decomposed == precomposed);
```

```text
decomposed == precomposed: false
```

Three bytes versus two, two scalars versus one, and indistinguishable on screen. Persian has its own version:

```rust
let two_letters = "لا";
let ligature = "\u{fefb}";
println!("two_letters == ligature: {}", two_letters == ligature);
```

```text
two_letters == ligature: false
```

The standard answer to this family of problems is **normalization**: before comparing or storing, convert the text into one agreed shape (usually NFC). In Rust that is the `unicode-normalization` crate — again a dependency, not `std`.

And one trap that is specifically ours and has nothing to do with counting: Persian `ک` (`U+06A9`) is not Arabic `ك` (`U+0643`), and Persian `ی` (`U+06CC`) is not Arabic `ي` (`U+064A`). Both pairs render almost identically and will never compare equal. A user typing on an Arabic keyboard creates a row in your database that your search will never find. Example `04` prints both side by side.

### So which number do you want?

Three different questions, three different answers. The common mistake is thinking there is only one question:

| What you want to know | Ask | Cost |
|---|---|---|
| how much memory / disk / bandwidth it takes | `text.len()` | instant |
| how many Unicode scalars it has | `text.chars().count()` | linear — the whole text is read |
| how many "characters" the user sees | count grapheme clusters | linear, and needs a crate |
| whether you may cut at byte N | `text.is_char_boundary(at)` | instant |

Working rule: **if the number is shown to a user, it's graphemes. If the number is handed to a buffer, it's bytes. Scalars are a good approximation of the first and a dangerous one for the second.**

### What this means in a real backend

Four places it will matter to you this week:

**1. Validation.** Which number implements "at most 12 characters"?

```rust
let name = "محمدمتین";
println!("  len()            = {}", name.len());
println!("  chars().count()  = {}", name.chars().count());
println!("  rejected by a 12-byte rule:   {}", name.len() > 12);
```

```text
  len()            = 16
  chars().count()  = 8
  rejected by a 12-byte rule:   true
```

A perfectly ordinary eight-letter name, rejected by the byte rule. If the rule is written for a human, count scalars — or better, graphemes.

**2. Truncation.** `&text[..100]` on Persian text is a time bomb: the moment byte 100 lands mid-letter, the program panics. The right way is [1.4.4](../04-slicing-text-safely/README.md).

**3. Column alignment.** `{:<20}` pads by scalar count, not by displayed width. It will always be crooked for Persian, and worse for emoji. Every table printed in this lesson puts the Persian text **last on the line** — the same trick you'll see in the example code.

**4. Database and network limits.** In Postgres `varchar(n)` counts characters, the `Content-Length` header counts bytes, and an index's 64 KB limit counts bytes too. Text can pass your application's rule and then hit the database's. When that happens, the first question to ask is: "is that number bytes or characters?"

> **The Python bridge, and where it breaks.** In Python 3, `len("سلام")` is 4, because `str` is a sequence of scalars rather than bytes. That is easier — and it is the same convenience that leaves you unable to say how much memory the text takes or when you are paying to encode it. Python is exactly as unaware of graphemes as Rust is: `len("👨‍👩‍👧")` is 5 there too. The difference isn't that Python solved this. The difference is that Rust makes you say which question you're asking.

---

## Hands on

```sh
cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 01-one-scalar-many-bytes
cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 02-three-ways-to-count
cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 03-walking-the-text
cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 04-what-people-call-a-character
```

Then the three broken ones. One of them compiles and blows the program up; the other two never compile at all:

```sh
cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 05-not-a-char-boundary --features broken
cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 06-a-char-is-not-a-string --features broken
cargo run -p p1-04-02-utf8-bytes-chars-graphemes --example 07-chars-has-no-len --features broken
```

Then try:

1. In `01`, add your own name letter by letter to the `show` list. What do the bytes add up to? Now check it with `02`.
2. In `02`, add a `report` for an all-English and an all-Persian sentence with the same number of letters. What is the byte-to-scalar ratio?
3. In `03`, change the text to `"a۵🌸"`. Which bytes are boundaries now? Guess before you run it.
4. In `04`, add `show("your name", "متین", 4)`, then a version with a fatha on the first letter. Which column changed?

---

## Errors you will meet

### Panic — `byte index 1 is not a char boundary`

This one is not caught at compile time. The code is well typed; the program dies at run time:

```text
the whole word:  سلام
its byte length: 8

thread 'main' (19756) panicked at examples\05-not-a-char-boundary.rs:15:22:
end byte index 1 is not a char boundary; it is inside 'س' (bytes 0..2 of string)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What it is objecting to:** the code said `&word[0..1]` — "byte zero up to byte one". Byte 1 is not a char boundary; it is inside `س`. If Rust allowed that slice it would produce a `&str` that isn't valid UTF-8, and then it wouldn't be a `&str`.

The number in brackets — `(19756)` — is a thread id and changes every run. Two other things matter: **which byte**, and **inside which letter**. Rust's message gives you both.

**The fix:** stop guessing at bytes and ask for scalars:

```rust
for letter in word.chars() {
    println!("the first letter: {letter}");
    break;
}
```

```text
the first letter: س
```

**Why that's the fix:** `&word[0..1]` has "one letter is one byte" baked into it. That assumption is true for English and false for your text. `.chars()` makes no assumption at all — it asks the text.

And notice what Rust chose here: **a panic, not broken text.** Other languages hand you the half-byte and you find `Ø³` in a production log three weeks later. Safe slicing is [1.4.4](../04-slicing-text-safely/README.md); why panicking is sometimes the right choice is [1.6.4](../../06-absence-and-failure/04-panic-vs-result/README.md).

### `E0308` — a `char` is not a string

```text
error[E0308]: mismatched types
  --> examples\06-a-char-is-not-a-string.rs:12:22
   |
12 |         if letter == "س" {
   |            ------    ^^^ expected `char`, found `&str`
   |            |
   |            expected because this is `char`
   |
help: if you meant to write a `char` literal, use single quotes
   |
12 -         if letter == "س" {
12 +         if letter == 'س' {
   |
```

**What it is objecting to:** `.chars()` yields values of type `char`. `"س"` with double quotes is a `&str` — a two-byte view of text living somewhere else. They are different types, and Rust does not convert between them behind your back.

**The fix:** exactly what the compiler wrote — single quotes:

```rust
if letter == 'س' {
    seen += 1;
}
```

```text
the letter appears 1 times
```

**Why that's the fix:** in Python `"س"` is both a string and a character, because Python has no `char` type — it has a one-element string. In Rust these are genuinely different things: a `char` is four bytes on the stack holding one scalar, a `&str` is a pointer and a length. Single and double quotes build different types and always will.

### `E0599` — `Chars` has no `len`

```text
error[E0599]: no method named `len` found for struct `Chars<'a>` in the current scope
    --> examples\07-chars-has-no-len.rs:10:42
     |
  10 |     println!("letters: {}", word.chars().len());
     |                             ----         ^^^
     |                             |
     |                             method `len` is available on `&str`
     |
help: there is a method `le` with a similar name, but with different arguments
```

**What it is objecting to:** `.chars()` doesn't build a collection; it returns a **walk** — something that hasn't run yet and doesn't know how many items it has. To know, it would have to go to the end. `len` belongs to things that already know their length.

**The fix:** `.count()`, which is precisely "go to the end and count":

```rust
println!("letters: {}", word.chars().count());
println!("bytes:   {}", word.len());
```

```text
letters: 4
bytes:   8
```

**Why that's the fix:** and now read that middle line of the error again: `method len is available on &str`. The compiler is suggesting you drop the `.chars()` and write `word.len()` — which compiles, warns about nothing, and **gives the wrong answer**: 8 instead of 4.

That's the best lesson in this whole section. The compiler understands your types, not your intent. Read its suggestion, but don't take it blind.

---

## Exercises

### Warm up

<details>
<summary>What are <code>"سلام".len()</code> and <code>"سلام".chars().count()</code>?</summary>

Eight and four. Every Persian letter is two bytes in UTF-8.

</details>

<details>
<summary><code>"Rust برای بک‌اند".len()</code> is 27 but it has 16 scalars. Where are the 11 extra bytes?</summary>

They are continuation bytes. Nine of the sixteen scalars are Persian letters, two bytes each, so each contributes one. The tenth non-ASCII scalar is the ZWNJ, which is three bytes and contributes two. Nine plus two is eleven. The gap between bytes and scalars is always exactly the number of continuation bytes.

</details>

<details>
<summary>How many bytes is a <code>char</code>?</summary>

In memory, always four. Inside a `String`, between one and four depending on which scalar it is. `size_of::<char>()` answers the first, `letter.len_utf8()` the second.

</details>

<details>
<summary>In <code>"سلام"</code>, which bytes are char boundaries?</summary>

0, 2, 4, 6 and 8. All the odd ones are inside a letter, and slicing at one of them panics.

</details>

<details>
<summary><code>"می‌روم"</code> has five letters. What does <code>.chars().count()</code> say?</summary>

Six. The ZWNJ is a full scalar of its own — three bytes — that displays nothing and still counts.

</details>

<details>
<summary>Why doesn't Rust's standard library count grapheme clusters?</summary>

Because the rules need large Unicode tables that change every year, and `std` doesn't want to carry them. The `unicode-segmentation` crate does the job.

</details>

<details>
<summary>Are two strings that look identical necessarily equal under <code>==</code>?</summary>

No. `"é"` written as one scalar is not equal to `"é"` written as `e` plus a combining mark. `==` compares bytes, not pictures.

</details>

### Repair

Fix all three broken examples.

1. Fix `examples/05-not-a-char-boundary.rs` **two** ways: once so it really gives the first letter, once so it really gives the first byte. Then say which one the original author meant.

2. Fix `examples/06-a-char-is-not-a-string.rs` with the compiler's own suggestion. Then try it the other way: instead of changing the quotes, convert `letter` into a string. Does it compile? Is it the right thing to do?

3. Fix `examples/07-chars-has-no-len.rs` **two** ways: once with the suggestion the compiler offers, once correctly. Put the two numbers side by side and write down why the compiler's suggestion is wrong here.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-04-02-utf8-bytes-chars-graphemes
```

The first three are one-liners if you know the right method. The last two need a loop, and the last one proves something: the `every_byte_is_a_start_or_a_continuation` test says `counts` and `continuation_bytes` are not independent. Get both right and that test passes on its own.

### Build

Write a `pub fn describe(text: &str)` that reports, for any text:

- its byte length and its scalar count
- the width of the widest scalar in it
- for every scalar: the byte it starts at, and its width

Run it on your own name in Persian, then on `"سلام، من متین هستم. 🌸"`, then on an English sentence with the same number of letters.

Then answer this, in one written sentence: **if tomorrow you have to implement a "display name" field with a limit of "at most 30 characters", which number do you check, and why?** The right answer depends on who reads that 30 — the user, or the database. Write down both cases.

### Challenge (optional)

**Part one.** Without running anything, work out three numbers for `"سَلام"`: its byte length, its scalar count, and its number of continuation bytes. Then check with example `04`.

**Part two.** Example `01` prints `F0 9F 8C B8` for `'🌸'`. Using the bit table in "The rule UTF-8 follows", pull the data bits out of those four bytes and join them up. The number you get should be `0x1F338`. If it isn't, count again — this exercise is exactly what `.chars()` does on every turn of the loop.

**Part three.** *This part reaches forward.* In a separate project — not this lesson — add `unicode-segmentation` to a `Cargo.toml` and write:

```rust
let clusters = UnicodeSegmentation::graphemes("👨‍👩‍👧", true);
```

Then compare the number of clusters with `.chars().count()`. Adding a dependency is [Phase 2](../../../phase2-intermediate/08-rust-toolbox/03-cargo-features/README.md) material, so it's fine if you get stuck — the goal is to see the real number once.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Unicode scalar value | the number Unicode assigns a character | exactly what a `char` holds |
| UTF-8 | writing one scalar as 1–4 bytes | the encoding of every Rust `String` |
| continuation byte | a byte starting with `10` | the gap between `len()` and the scalar count |
| char boundary | a byte where a scalar starts | the only place a slice is allowed |
| `len()` | the number of **bytes** | buffers, disk, network |
| `.chars().count()` | the number of scalars | anywhere "letters" matter |
| `.char_indices()` | each scalar with the byte it starts at | slicing, locating, truncating |
| `.as_bytes()` / `.bytes()` | the raw byte view | hashing, protocols, size limits |
| `len_utf8()` | the encoded width of one `char` | working out how much room you need |
| combining mark | a scalar that rides on the letter before it | vowel marks, decomposed `é` |
| grapheme cluster | what a person calls "one character" | `unicode-segmentation` |

### What you now know

- UTF-8 writes each scalar in 1 to 4 bytes, and the leading bits of every byte say which it is.
- ASCII is one byte, Persian and Arabic are two, emoji are four, and the ZWNJ is three.
- A `char` in memory is always four bytes; the same `char` inside a `String` is one to four.
- The gap between `len()` and `.chars().count()` is exactly the number of continuation bytes.
- `.char_indices()` gives you the starting byte of every scalar, and `is_char_boundary` says where you may cut.
- Slicing at a byte that isn't a boundary panics rather than producing broken text.
- `.chars().count()` still isn't "how many characters a person sees": combining marks, the ZWNJ and composed emoji pull them apart.
- Two identical-looking strings can compare unequal, and the answer to that is normalization.

### What comes back later

- **Building text, and `format!`** — [1.4.3](../03-building-and-transforming-strings/README.md)
- **Slicing text safely, on boundaries** — [1.4.4](../04-slicing-text-safely/README.md)
- **The `Option` that comes out of `.next()`** — [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **Panic versus `Result`, and when each is right** — [1.6.4](../../06-absence-and-failure/04-panic-vs-result/README.md)
- **Loops that become iterator chains** — [Phase 2 — iterators](../../../phase2-intermediate/02-iterators-and-closures/README.md)
- **Adding a crate such as `unicode-segmentation`** — [Phase 2 — cargo features](../../../phase2-intermediate/08-rust-toolbox/03-cargo-features/README.md)

### Can you explain?

- Why is `"سلام".len()` eight?
- What's the difference between a starting byte and a continuation byte in UTF-8?
- Why is a `char` four bytes when `'a'` inside a `String` is one?
- What does `&word[0..1]` do to `"سلام"`, and why does Rust die instead of handing you broken text?
- What are the three different numbers you could give for "the length of this text", and what is each one good for?
- Why isn't `.chars().count()` the answer to "how many characters does the user see"?

---

## Going further

- [The Rust Book — Storing UTF-8 encoded text with strings](https://doc.rust-lang.org/book/ch08-02-strings.html) — the same ground, officially.
- [`char::len_utf8`](https://doc.rust-lang.org/std/primitive.char.html#method.len_utf8) and [`str::char_indices`](https://doc.rust-lang.org/std/primitive.str.html#method.char_indices) — short docs, worth reading.
- [`unicode-segmentation`](https://docs.rs/unicode-segmentation) — the crate that counts grapheme clusters. Don't add it yet; just know it's there.
- [UTF-8 Everywhere](https://utf8everywhere.org/) — why the world settled on this encoding.
- [The Unicode Arabic block chart](https://www.unicode.org/charts/PDF/U0600.pdf) — where `U+0633` and the rest of our letters live.
