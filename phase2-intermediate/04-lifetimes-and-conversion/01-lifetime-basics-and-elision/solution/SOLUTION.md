# Solution

```rust
pub fn shorter<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() <= b.len() {
        a
    } else {
        b
    }
}
```

`shorter<'a>(a: &'a str, b: &'a str) -> &'a str` is `longest` with the comparison flipped and the tie-break kept the same shape: `a` wins when `a.len() <= b.len()`, so an exact tie returns `a`. The `<'a>` on both parameters and the return type is required for the same reason it was in `longest` — two input references, and the output could plausibly come from either one, so nothing in the three elision rules can pick for you.

```rust
pub fn first_sentence(text: &str) -> &str {
    match text.split_once('.') {
        Some((before, _after)) => before,
        None => text,
    }
}
```

`first_sentence(text: &str) -> &str` needed no `<'a>` at all — elision rule 2 fires, because there is exactly one input reference. `.split_once('.')` is the cleaner tool here than `.split('.').next()`: it returns `None` when the delimiter is genuinely absent, so the `None` arm above is really reachable and really means "no period was found" — nothing is left silently unreachable.

```rust
impl Setting {
    pub fn value(&self) -> &str {
        match self.raw.split_once('=') {
            Some((_key, value)) => value,
            None => &self.raw,
        }
    }
}
```

`value(&self) -> &str` is elision rule 3: `&self` is the only reference in the signature, so its lifetime is assigned to the return type automatically — no `<'a>` appears anywhere on the method, even though `Setting` itself is a perfectly ordinary struct with no lifetime parameter at all. That last part matters: `Setting` owns a `String` outright, so this method is exactly the shape [2.4.2](../../02-lifetimes-in-structs-and-methods/README.md) contrasts with a struct that holds a *borrowed* field.

**On Warm up question 5:** `fn longest<'a>(x: &'a str, y: &str) -> &'a str` **does not** compile with the body from the lesson. Only `x` is tied to `'a`; `y` gets its own, independent, unnamed lifetime. The moment the body's `else` branch tries to return `y` as an `&'a str`, the compiler has no proof that `y`'s real lifetime covers `'a` — so it refuses, with `E0621`, before any call site is even considered. Tying both inputs to the same `'a` is what supplies that proof; it does not make either input live any longer than it already does.
