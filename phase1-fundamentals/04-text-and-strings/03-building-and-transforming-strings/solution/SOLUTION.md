# Solution — 1.4.3 Building and transforming strings

```rust
pub fn joined(parts: &[String], separator: &str) -> String {
    parts.join(separator)
}

pub fn aligned_row(label: &str, amount: f64) -> String {
    format!("{label:<10}{amount:>8.2}")
}

pub fn tidy(text: &str) -> String {
    let mut words: Vec<&str> = Vec::new();
    for word in text.split_whitespace() {
        words.push(word);
    }
    words.join(" ")
}

pub fn shout(text: &str) -> String {
    let mut out = text.trim().to_uppercase();
    out.push('!');
    out
}

pub fn preview(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut out = format!("{text:.limit$}");
    out.push('…');
    out
}
```

Two one-liners, and three that are worth the paragraphs below.

## `joined` — the debt from 1.1.6, paid

```rust
parts.join(separator)
```

That's the whole thing. Here is what you wrote in [1.1.6](../../../01-foundations/06-vec-and-string-basics/README.md):

```rust
let mut out = String::new();
for index in 0..parts.len() {
    if index > 0 {
        out.push(separator);
    }
    out.push_str(&parts[index]);
}
out
```

Same answer. But the hand-written one starts from an empty buffer and grows it, so a long list reallocates several times on the way — `String` doubles when it fills, exactly like the `Vec` in 1.1.6.

`join` doesn't. It walks the slice once to add up the lengths, takes **one** buffer of exactly that size, and then copies. One allocation, no growth, no waste at the end.

You wrote it by hand once so that "join" is a thing you understand rather than a thing you call. From here on, call it.

Two details the tests check that are easy to miss. An empty slice gives `""` and not a lone separator — `join` puts separators *between* things, and between nothing there is nothing. And `joined(&["a", "b"], "")` gives `"ab"`, because an empty separator is a perfectly good separator; that is `concat` written a different way.

## `aligned_row` — one format string, two columns

```rust
format!("{label:<10}{amount:>8.2}")
```

Read it left to right and it says exactly what the specification said:

- `label` — the variable, captured inline from scope.
- `:<10` — left-aligned, minimum width ten.
- `amount` — the other variable.
- `:>8.2` — right-aligned, minimum width eight, two digits after the point.

Nothing between the two groups, which is why nothing appears between the columns.

**The alignment markers are not optional here, even though they match the defaults.** Text already goes left and numbers already go right. Writing them anyway means the row doesn't quietly re-lay-itself-out the day somebody changes `amount` from an `f64` to something else. It also means a reader doesn't have to remember the defaults.

`.2` **rounds**; it doesn't cut. That's what `aligned_row("x", 12.3456)` returning `"x            12.35"` is checking — `12.34` would be a truncation and it is not what happened.

And the test asserting `aligned_row("a-very-long-label", 1.0)` is 25 characters is checking the other half of the rule: the width is a *minimum*. A value too wide for its column pushes the row out rather than getting chopped. If you want a maximum too, that is precision on the text — `{label:<10.10}` — and then long labels get cut instead.

## `tidy` — split and rejoin, and the pieces cost nothing

```rust
let mut words: Vec<&str> = Vec::new();
for word in text.split_whitespace() {
    words.push(word);
}
words.join(" ")
```

The trick is that "strip the outside and squeeze the inside" is one operation if you look at it right: take only the non-empty pieces, then put single spaces back between them.

`split_whitespace` does both halves at once. Unlike `split(' ')`, it treats any run of whitespace as one separator and produces no empty pieces — which is why `"a\t\tb\nc"` comes out as three words and `"   "` comes out as none. And it handles the ends for free, so no `trim` is needed.

Note the type: `Vec<&str>`, not `Vec<String>`. Every piece is a **window into `text`**, so filling that vector copies no text at all. The only allocation in the whole function is the one `join` makes at the end — plus the vector's own buffer, which holds pointers rather than characters.

That is the whole lesson in one function. Nothing gets copied until something is actually built.

The `""` case falls out for free: no pieces, so `join` returns an empty `String`. No special-case branch needed.

## `shout` — the one where Persian is the point

```rust
let mut out = text.trim().to_uppercase();
out.push('!');
out
```

Three operations, and only one of them allocates.

`trim` returns a `&str` into the same buffer. `to_uppercase` is the allocation: it builds a new `String`, and it has to, because upper case can change the length of the text. `push` writes one more character into the buffer that already exists.

Order matters for cost. Doing `text.to_uppercase().trim().to_string()` gets the same answer with **two** allocations: one for the uppercase copy, one to turn the trimmed view back into an owned string. Trim the view first, then allocate once from it.

Now the assertion that matters:

```rust
assert_eq!(shout("سلام"), "سلام!");
```

`to_uppercase` ran, allocated a buffer, copied all seventeen bytes into it, and produced identical text. Persian has no upper case, so Unicode's uppercase mapping for each of those letters is the letter itself.

It is worth sitting with, because the habit it should break is a common one. "Lowercase it before comparing" is reflex in a Python codebase and it is *free of meaning* for Persian input — it does no folding at all and still costs a full copy. If you need `چای` and `چای` typed with an Arabic `ي` to compare equal, that is `replace`, not case conversion.

## `preview` — count characters, not bytes

```rust
if text.chars().count() <= limit {
    return text.to_string();
}
let mut out = format!("{text:.limit$}");
out.push('…');
out
```

The interesting line is the format string. `{text:.limit$}` means "format `text` with a precision taken from the variable `limit`", and precision on text means **truncate to at most that many characters**.

That `$` is what lets the limit be a runtime value rather than something baked into the string.

Why go through the formatter at all? Because it is the only tool in this lesson that shortens text **safely**. Writing `&text[..limit]` would be a *byte* index, and for Persian the two units never line up: `&"سلام"[..2]` hands you `"س"` — one letter where two were asked for — and `&"سلام"[..3]` hands you nothing at all, because byte 3 is inside the letter `ل` and the program panics on the spot. Cutting text by byte offsets is [1.4.4](../../04-slicing-text-safely/README.md), and it needs tools you don't have yet.

The guard is doing real work in two directions. Without it, `preview("hi", 5)` would append a `…` to text that was never shortened, which is a lie about the data. And the `chars().count()` in it is the same count the precision uses, so the two agree by construction — using `len()` there would make the function cut Persian text that fit perfectly well.

One caveat the tests deliberately leave alone. `preview("می‌شود", 3)` gives `"می‌…"` — three `char`s, of which one is a zero-width non-joiner, so you see two letters and an ellipsis. The function is doing exactly what it says. It is "characters" that is a slippery unit, and that is the subject of the challenge exercise.

## What this lesson was really about

- **A format string is code.** `{name}` is a variable lookup, and getting it wrong is a compile error rather than a 3 a.m. page.
- **`{}` and `{:?}` are two different questions.** Some types answer only one of them.
- **Width and precision count `char`s.** Not bytes, and not what a reader sees — which is exactly the gap Persian exposes.
- **Know what allocates.** `trim` and `split` borrow; `format!`, `join`, `replace` and `to_uppercase` all build a new buffer.
- **Build in one buffer where you can.** `push_str` into a sized `String`, or `write!`, instead of `format!` once per turn of a loop.
