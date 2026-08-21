# Solution — 1.4.2 UTF-8: bytes, chars, graphemes

```rust
pub fn counts(text: &str) -> (usize, usize) {
    (text.len(), text.chars().count())
}

pub fn bytes_for(letter: char) -> usize {
    letter.len_utf8()
}

pub fn bytes_of_widest(text: &str) -> usize {
    let mut widest = 0;
    for letter in text.chars() {
        if letter.len_utf8() > widest {
            widest = letter.len_utf8();
        }
    }
    widest
}

pub fn bytes_of_first(text: &str, n: usize) -> usize {
    let mut seen = 0;
    for (at, _letter) in text.char_indices() {
        if seen == n {
            return at;
        }
        seen += 1;
    }
    text.len()
}

pub fn continuation_bytes(text: &str) -> usize {
    let mut found = 0;
    for byte in text.bytes() {
        if (0x80..=0xBF).contains(&byte) {
            found += 1;
        }
    }
    found
}
```

Five functions, three levels: bytes, scalars, and the boundary between them.

## `counts` — two different questions on one line

```rust
(text.len(), text.chars().count())
```

The point isn't the code. The point is that it makes you write both numbers next to each other and see that they aren't the same.

`text.len()` is instant: a `&str` is two words, a pointer and a length, and the length is already sitting there. `text.chars().count()` has to read the whole buffer and count starting bytes. For one short string that doesn't matter; for filtering ten thousand rows inside one request it does.

The test wants `counts("سلام، من متین هستم.")` to be `(34, 19)`. If you had written that number from memory you'd have got it wrong — which is why this lesson prints everything instead of asserting it.

## `bytes_for` — `size_of` versus `len_utf8`

```rust
letter.len_utf8()
```

One method, but choosing it is the whole lesson. `size_of::<char>()` is always 4, because a `char` in memory is a fixed four-byte box. `letter.len_utf8()` says how many bytes that same value becomes once it is encoded as UTF-8.

If you're working out how much room something needs on disk or on a socket, you want `len_utf8`. If you're working out the size of an array of `char`, you want `size_of`.

## `bytes_of_widest` — a max loop, and zero for empty text

```rust
let mut widest = 0;
for letter in text.chars() {
    if letter.len_utf8() > widest {
        widest = letter.len_utf8();
    }
}
widest
```

Starting at zero does two jobs: it's the right seed for the comparison, and it's exactly the answer the specification wants for empty text. On empty text the loop runs zero times and that zero falls straight out. No special case was needed — only a well-chosen starting value.

`bytes_of_widest("Rust برای بک‌اند")` is 3, not 2. That's a surprise until you remember the ZWNJ is three bytes. The test is there on purpose.

## `bytes_of_first` — why `char_indices` and not `chars`

```rust
let mut seen = 0;
for (at, _letter) in text.char_indices() {
    if seen == n {
        return at;
    }
    seen += 1;
}
text.len()
```

You could write this with `.chars()` and add up the widths yourself. But `.char_indices()` has already done that addition and hands it to you for free: `at` **is** the number of bytes you have passed.

The `return` inside the loop is an early exit; once scalar `n` is found there is nothing more to do. And if it is never found — the text has fewer than `n` scalars — the loop ends and `text.len()` comes back. That's what the specification asked for, and it also gives 0 for `bytes_of_first("", 3)`, because empty text has length zero.

`_letter` gets an underscore because the letter itself isn't wanted. Same `_` as [1.1.3](../../../01-foundations/03-compound-types-and-destructuring/README.md), doing the same job: a position acknowledged and deliberately not named.

**And this function is exactly the number you need to truncate text safely.** Once you know "the first 30 letters end at byte N", cutting there is safe. The standard way to do it is [1.4.4](../../04-slicing-text-safely/README.md).

## `continuation_bytes` — proving the rule

```rust
for byte in text.bytes() {
    if (0x80..=0xBF).contains(&byte) {
        found += 1;
    }
}
```

`0x80..=0xBF` in binary is "every byte starting with `10`" — the exact pattern from the table in the lesson.

`.bytes()` rather than `.as_bytes()` because we only want to pass over them and never need an index. `.as_bytes()` works just as well and copies exactly as much: nothing, in both cases.

And now that last test:

```rust
let (bytes, scalars) = counts(text);
assert_eq!(bytes - continuation_bytes(text), scalars);
```

It ties two separate functions together and proves one fact: **every byte in a `String` is either the start of a scalar or a continuation of one.** There is no third kind. That's why the gap between `len()` and `.chars().count()` is never mysterious — it is always exactly the number of continuation bytes.

If either function is wrong this test blows up, even when that function's own tests passed. A test that measures a law rather than a value is worth more than both of them.

## What this lesson was really about

- **UTF-8 writes each scalar in 1 to 4 bytes**, and the leading bits of every byte say which it is.
- **`len()` is bytes and always will be.** For English it accidentally looks right.
- **The gap between bytes and scalars is the number of continuation bytes** — a law, not a coincidence.
- **`.char_indices()` gives you the number every slicing problem is really about.**
- **`.chars().count()` still isn't "how many characters the user sees".** That question is answered by grapheme clusters and needs a crate.
- And most of all: **before any count, ask who reads the number.** If it's the user, graphemes. If it's a buffer, bytes.
