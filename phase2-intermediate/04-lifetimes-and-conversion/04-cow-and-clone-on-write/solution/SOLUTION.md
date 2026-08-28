# Solution — 2.4.4 `Cow<'_, str>` and copy-on-write

```rust
pub fn describe(value: &Cow<'_, str>) -> &'static str {
    match value {
        Cow::Borrowed(_) => "borrowed",
        Cow::Owned(_) => "owned",
    }
}

pub fn collapse_spaces(input: &str) -> Cow<'_, str> {
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

pub fn ensure_exclaimed(mut value: Cow<'_, str>) -> Cow<'_, str> {
    if !value.ends_with('!') {
        value.to_mut().push('!');
    }
    value
}
```

## `describe` — a plain `match`, because there is no other way

`Cow`, exactly as the lesson said, is a real enum — and the only way to find out which variant you have is either a `match`, or something built on top of one (like the `matches!` macro you saw plenty of in the tests themselves). This function does exactly what `{:?}` cannot: `Debug`, as the concept section showed, forwards straight to the inner value, never to the variant's name. That's why the signature deliberately takes `&Cow<'_, str>` rather than `&str` — a plain `&str` would already have erased that information.

## `collapse_spaces` — one cheap check before any decision

```rust
if !input.contains("  ") {
    return Cow::Borrowed(input);
}
```

This line splits the whole function into two paths before a single byte gets allocated. `.contains("  ")` only ever reads the input — no copying involved — and if the answer is no, `input` comes straight back, completely unchanged, that same instant. This is exactly the address-based proof the "motivating shape" subsection showed you.

```rust
let mut previous_was_space = false;
for ch in input.chars() {
    if ch == ' ' && previous_was_space {
        continue;
    }
    previous_was_space = ch == ' ';
    collapsed.push(ch);
}
```

Once a run genuinely exists, the loop handles it with one simple flag: every character except a space immediately following another space gets pushed onto `collapsed`. `previous_was_space` is only updated at the exact point a character is actually pushed — meaning it stays untouched for every character that gets `continue`d (the extra spaces themselves), which is exactly what collapses a run of three spaces down to a single one, not zero. Tabs and newlines are never equal to `' '`, so the condition is always false for them and they always pass through untouched.

## `ensure_exclaimed` — check first, `.to_mut()` second, never the other way around

```rust
if !value.ends_with('!') {
    value.to_mut().push('!');
}
value
```

Order matters here. `.ends_with('!')` works on `Cow` through `Deref` — no `.to_mut()` needed for that — so if `value` already ends with `'!'`, this function never calls `.to_mut()` at all, and `value` — whether it started `Borrowed` or `Owned` — comes back exactly as it arrived. `.to_mut()` only gets called when there is genuinely something to append: if `value` was still `Borrowed`, that one call clones it; if it was already `Owned`, no fresh clone happens. This is what makes "clone on write" a real behavior, not just a name.

## What this lesson was really about

- **`Cow` is a real enum that `Deref` mostly lets you avoid touching directly.** `describe` was the only place in this lesson that genuinely needed an explicit `match` — everywhere else, `Deref` did the work.
- **The Borrowed-versus-Owned decision always comes with a cheap check first, before any allocation.** `collapse_spaces` showed this with `.contains("  ")`; the pattern is always the same shape: ask first, build second.
- **`.to_mut()` gets called only when it's genuinely needed, not on every pass.** `ensure_exclaimed` showed this by putting `.ends_with('!')` ahead of `.to_mut()` — exactly the discipline that avoids an unnecessary clone.
