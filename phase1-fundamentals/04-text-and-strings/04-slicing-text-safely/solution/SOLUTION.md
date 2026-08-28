# Solution — 1.4.4 Slicing text safely

```rust
pub fn char_boundaries(text: &str) -> Vec<usize> {
    let mut found = Vec::new();
    for index in 0..=text.len() {
        if text.is_char_boundary(index) {
            found.push(index);
        }
    }
    found
}

pub fn safe_prefix(text: &str, max_bytes: usize) -> &str {
    &text[..text.floor_char_boundary(max_bytes)]
}

pub fn truncate_to_chars(text: &str, max_chars: usize) -> &str {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == max_chars {
            return &text[..index];
        }
        seen += 1;
    }
    text
}

pub fn truncated_with_ellipsis(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let kept = truncate_to_chars(text, max_chars);
    format!("{kept}…")
}

pub fn split_at_char(text: &str, char_index: usize) -> (&str, &str) {
    let mut seen = 0;
    for (index, _) in text.char_indices() {
        if seen == char_index {
            return text.split_at(index);
        }
        seen += 1;
    }
    (text, "")
}
```

None of these five functions panics on any input, and that is not luck: every slice written in this file takes its number from somewhere that guarantees it is a boundary.

## `char_boundaries` — why `0..=text.len()` and not `0..text.len()`

```rust
for index in 0..=text.len() {
```

The range is inclusive because `text.len()` is itself a legal boundary. With `0..text.len()` the empty string would give an empty list where the right answer is `[0]`, and every other string would lose its final boundary — which is to say, the ability to slice all the way to the end.

`is_char_boundary` returns `false` for an index past the end rather than panicking, so this loop is never dangerous.

The same thing can be written with `char_indices`, and in practice that is faster:

```rust
for (index, _) in text.char_indices() {
    found.push(index);
}
found.push(text.len());
```

Both are correct. The first says directly what the definition of "boundary" is and is better for learning; the second builds the list in one pass instead of asking `n` separate questions.

## `safe_prefix` — one line, because the library did the work

```rust
&text[..text.floor_char_boundary(max_bytes)]
```

Three edge cases are handled in that one line, and none of them needs an `if`:

- `max_bytes` past the end: `floor_char_boundary` clamps to `text.len()`, so the whole text comes back.
- `max_bytes` in the middle of a character: it steps back to the first boundary.
- `max_bytes` of zero: zero is always a boundary, so `""`.

And the `&text[..cut]` can never panic, because `cut` came from a function whose entire definition is "give me a boundary".

If you are on a compiler older than 1.91, the manual equivalent is:

```rust
let mut cut = if max_bytes > text.len() {
    text.len()
} else {
    max_bytes
};
while !text.is_char_boundary(cut) {
    cut -= 1;
}
&text[..cut]
```

That first `if` is needed: without it, `is_char_boundary` returns `false` for a large number and the loop starts counting down from above the length — which works, but does needless work.

## `truncate_to_chars` — a counter, not arithmetic

```rust
let mut seen = 0;
for (index, _) in text.char_indices() {
    if seen == max_chars {
        return &text[..index];
    }
    seen += 1;
}
text
```

Here is the central point of the whole lesson: `index` came out of `char_indices` itself, so it **is** a character boundary. The slice is safe without you checking anything.

The order inside the loop matters. Check first, then count. Swap those two and `max_chars` of 3 returns four characters.

And if the loop runs to the end, the text had `max_chars` characters or fewer, so all of `text` is the answer. Notice that `chars().count()` is never computed up front — that is a whole extra pass over the text, and most calls do not need it.

The return type is `&str`. Nothing was allocated; we handed back a smaller window onto the same buffer. Make the signature `String` and every call becomes an allocation — and this is exactly the function you call in a loop over a thousand rows.

Run `cargo clippy` over this crate and it will object:

```text
warning: the variable `seen` is used as a loop counter
  --> src\lib.rs:61:5
   |
61 |     for (index, _) in text.char_indices() {
   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ help: consider using: `for (seen, (index, _)) in text.char_indices().enumerate()`
```

Clippy is right, and the counter is still the correct thing to write today: `.enumerate()` is an iterator adapter, and those belong to Phase 2. That rewrite is the third part of this lesson's challenge, and it is worth coming back for once you have the tool.

## `truncated_with_ellipsis` — test on characters, not bytes

```rust
if text.chars().count() <= max_chars {
    return text.to_string();
}
```

Here `chars().count()` is genuinely needed and there is no shortcut: you have to know *whether* anything was cut, and that question is about characters.

Writing `text.len() <= max_chars` breaks this test:

```rust
assert_eq!(truncated_with_ellipsis("سلام", 4), "سلام");
```

"سلام" is four characters but eight bytes. With the byte test, `8 <= 4` is false, so the function decides it must cut, takes four characters (which is the whole text) and glues on an ellipsis that is a lie. The user sees "سلام…" and believes something was hidden from them.

That is precisely the test that exists to catch this mistake, and precisely what breaks a "20 character limit" in the real world.

The ellipsis is one `…` character (`U+2026`), not three `.`. The "never longer than `max_chars + 1` characters" test fails with three dots.

## `split_at_char` — the same pattern, using `split_at`

```rust
if seen == char_index {
    return text.split_at(index);
}
```

`split_at` is normally the dangerous one, but not here — for the usual reason: `index` came from `char_indices`.

Two slices would have worked as well: `(&text[..index], &text[index..])`. `split_at` does the same job, checks once instead of twice, and says more plainly that you want two adjacent pieces.

The `(text, "")` at the end is the case where `char_index` was larger than the number of characters. There is a test that checks, for every sample and every cut, that the two halves rebuild the original:

```rust
let (head, tail) = split_at_char(sample, cut);
assert_eq!(format!("{head}{tail}"), sample);
```

`("", text)` would have passed that one too — and would have changed the function's behaviour for a large `char_index` with nobody noticing. So there is a separate test demanding that `split_at_char("سلام", 99)` is exactly `("سلام", "")`. You read the specification from the doc comment, not from the tests.

## What this lesson was really about

- **`&text[a..b]` counts bytes.** In ASCII that is invisible; in Persian it is not.
- **A slice on a bad boundary panics rather than producing broken output.** That is design, not strictness.
- **Any index that comes out of `char_indices` is safe.** Most of the safe code in this file follows from that single fact.
- **`.get()` is safe because it returns a different type.** `Option<&T>` means "there might be nothing", and the compiler makes you think about that case.
- **"At most N characters" has to be implemented in characters,** or it is a bug that only your non-English users ever see.
