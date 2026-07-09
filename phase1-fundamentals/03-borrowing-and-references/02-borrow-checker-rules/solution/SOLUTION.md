# Solution

```rust
pub fn first_word_len_then_clear(s: &mut String) -> usize {
    let len = s.split_whitespace().next().unwrap_or("").len();
    s.clear();
    len
}
```
`s.split_whitespace().next()` creates a temporary shared borrow of `s` to
produce a `&str` slice — but `.len()` is called on it in the very same
statement, immediately converting it into a plain `usize` (an owned,
`Copy` value with no connection back to `s` at all). By the time
`s.clear()` runs on the next line, that temporary borrow is long gone — no
conflict.

```rust
pub fn make_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}
```
The fix for "dangling reference" isn't a clever borrow-checker trick — it's
architectural: don't try to return a reference to data that's about to be
destroyed. Return an owned value instead, and ownership transfers cleanly
to the caller.

```rust
pub fn describe_and_grow(s: &mut String) -> String {
    let description = {
        let first_char = s.chars().next().unwrap_or(' ');
        format!("{} chars, starts with '{}'", s.len(), first_char)
    };
    s.push_str(" (grown)");
    description
}
```
The inner `{ ... }` block is the whole trick: `first_char` (a borrow
derived from `s`) and the `format!` call that uses it are both confined
inside it. The moment that block ends, `description` (an owned `String` —
`format!` always produces one) is all that's left; every borrow that went
into producing it has already expired. `s.push_str(...)` afterward sees `s`
with no live borrows against it at all.

On checkpoint question 1: uncommenting `conflicting_borrows_demo` gives
roughly `E0502: cannot borrow s as mutable because it is also borrowed as
immutable`. It's not "the compiler doesn't like two borrows" in the
abstract — it's specifically protecting against `r2` being used to mutate
`s` (e.g. reallocating its buffer if it grows) while `r1` still expects to
read the *old* memory location/contents. In a language without this check,
that's exactly the shape of a real use-after-free or iterator-invalidation
bug.
