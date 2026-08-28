# Solution — 1.4.1 `String` versus `&str`

```rust
pub fn byte_length(text: &str) -> usize {
    text.len()
}

pub fn shout(text: &str) -> String {
    text.to_uppercase()
}

pub fn joined(first: &str, second: &str) -> String {
    let mut out = String::with_capacity(first.len() + second.len());
    out.push_str(first);
    out.push_str(second);
    out
}

pub fn extended(text: String, extra: &str) -> String {
    let mut text = text;
    text.push_str(extra);
    text
}

pub fn all_owned(items: &[&str]) -> Vec<String> {
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        out.push(item.to_string());
    }
    out
}
```

Five functions, each making a different decision about who owns the bytes.

## `byte_length` — a view in, a number out

```rust
text.len()
```

The shortest of them, and a complete demonstration of the first half of the signature rule. The test calls this one function three different ways:

```rust
assert_eq!(byte_length("سلام"), 8);
let owned = "سلام دنیا".to_string();
assert_eq!(byte_length(&owned), 17);
assert_eq!(byte_length(owned.as_str()), 17);
```

A literal, a `&String`, a `&str` — and not one byte copied for any of them. Write the signature as `&String` instead and the first line doesn't compile at all: the caller would have to build a `String` first. An allocation, purely to get past the signature.

And hold onto that 8 and that 17. `"سلام"` is four letters and eight bytes; `"سلام دنیا"` is nine letters and seventeen bytes. `len()` counts bytes and always will. What those bytes actually are is [1.4.2](../../02-utf8-bytes-chars-graphemes/README.md).

## `shout` — a view in, an owner out

```rust
text.to_uppercase()
```

The second half of the rule. `to_uppercase` can't return a view, because the text it produces wasn't in the input. In German `ß` uppercases to `SS` — one letter becomes two — so the answer isn't even a piece of the input. New text means a new buffer means a new owner.

The test includes Persian on purpose:

```rust
assert_eq!(shout("سلام"), "سلام");
```

Persian has no upper case, so the text comes back unchanged — but it's still **a fresh `String`**, not the input. The function can't know in advance that there'd be nothing to do; the signature is decided before it runs.

## `joined` — allocate once, not twice

```rust
let mut out = String::with_capacity(first.len() + second.len());
out.push_str(first);
out.push_str(second);
out
```

This would also have worked:

```rust
let mut out = first.to_string();
out.push_str(second);
out
```

and it passes the test — except for the capacity line:

```rust
assert_eq!(persian.capacity(), persian.len(), "ask for the room once");
```

`first.to_string()` takes a buffer the size of `first`; then `push_str` finds there isn't room, takes a bigger buffer and copies everything across. Two allocations and one extra copy, for text whose final length was known from the start.

`with_capacity` is the answer to what you read about doubling growth in [1.1.6](../../../01-foundations/06-vec-and-string-basics/README.md): when you know the number, say it.

And look at `first.len() + second.len()` — bytes, not letters. For `"سلام"` and `" دنیا"` that's 8 + 9 = 17, which is exactly what the test wants. Count letters instead and the number is wrong and the buffer grows again. **Capacity is in bytes, because the buffer is in bytes.**

## `extended` — the only one that takes an owner

```rust
let mut text = text;
text.push_str(extra);
text
```

`String` in this signature isn't a mistake, it's a decision. The test proves it:

```rust
let was_at = roomy.as_ptr();
let grown = extended(roomy, " دنیا");
assert_eq!(grown.as_ptr(), was_at);
```

The address didn't change. The buffer the caller had is the buffer that came back — the whole function allocated nothing.

Had you written `&str` in that signature, you'd have been forced to build a new `String` and copy everything into it, because a view can't be grown. That works too, and costs one full allocation more.

So the rule becomes: **take `&str` when you're reading, take `String` when you're keeping.** The first lets the caller hold on to ownership; the second takes a buffer off a caller who has finished with it.

The `let mut text = text;` is only there to make the parameter mutable. You could write `mut text: String` in the signature instead and it's exactly the same thing — `mut` on a parameter isn't part of the signature, so callers can't tell the difference.

## `all_owned` — n views in, n allocations out

```rust
let mut out = Vec::with_capacity(items.len());
for item in items {
    out.push(item.to_string());
}
out
```

Nothing in that code says "this goes to the allocator n times", and yet it does: once for the Vec and once per element. It's the same `n + 1` you counted in [1.2.3](../../../02-ownership-and-memory/03-clone-and-copy/README.md), written this time as a view-to-owner conversion.

Before writing a function like this, always ask one question: **do you actually need owners?** If you only mean to add the lengths up or print them, keep the `&[&str]` as it is and allocate nothing. Here the signature says `Vec<String>`, so yes — but that was somebody's decision, not an obligation.

`Vec::with_capacity(items.len())` is `joined`'s move in a different place: the final count is known, so take the room once.

## What this lesson was really about

- **`String` is three words and `&str` is two.** The third word is capacity, and it's what makes growth possible.
- **Going up allocates, going down is free.** Every `.to_string()` is a trip to the allocator; every `.as_str()` is two words on the stack.
- **Take `&str`, return `String`** — unless you're keeping the buffer (like `extended`) or the answer is a piece of the input ([1.4.4](../../04-slicing-text-safely/README.md)).
- **Capacity is in bytes.** For Persian text that isn't the letter count, and that's the subject of [1.4.2](../../02-utf8-bytes-chars-graphemes/README.md).
- **Deref coercion only unwraps a `&`.** It never allocates for you; where an allocation is needed, it's written in the code.
