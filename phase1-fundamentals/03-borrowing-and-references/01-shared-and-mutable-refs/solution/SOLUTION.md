# Solution

```rust
pub fn count_vowels(s: &str) -> usize {
    s.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}
```
A shared borrow (`&str`) is all this needs — it only reads. `.chars()`
gives an iterator of `char`s (Phase 2 formalizes iterators; this is another
early preview), `.filter` keeps only vowels, `.count()` tallies them.
`text` stays fully valid in the caller after this call, unlike the
`total_length(strings: Vec<String>)` exercise two lessons ago, which took
ownership on purpose.

```rust
pub fn append_exclamation(s: &mut String) {
    s.push_str("!");
}
```
`&mut String` grants exclusive, temporary write access. No return value
needed — the caller's own `String` was mutated directly through the
reference.

```rust
pub fn swap_and_report(a: &mut i32, b: &mut i32) -> String {
    let (orig_a, orig_b) = (*a, *b);
    std::mem::swap(a, b);
    format!("swapped {orig_a} and {orig_b}")
}
```
`*a` and `*b` **dereference** the references to read the `i32` values
underneath (and since `i32` is `Copy`, `(*a, *b)` makes two independent
copies — no borrow-checker conflict). `std::mem::swap` takes two `&mut T`
and exchanges what they point to — this is allowed simultaneously because
`a` and `b` borrow two *different* `i32`s. The "only one `&mut` at a time"
rule is about one `&mut` per **value**, not a global limit on how many
`&mut` references can exist in a program at once.
