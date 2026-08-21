# Solution

```rust
pub fn find_by_id(users: &[(u32, String)], id: u32) -> Option<String> {
    users.iter().find(|(uid, _)| *uid == id).map(|(_, name)| name.clone())
}
```
`.find()` (an iterator method — Phase 2 covers these properly) returns
`Option<&(u32, String)>`; `.map()` transforms *inside* that `Option`
without needing to unwrap it first — if `.find()` came back `None`,
`.map()` is simply skipped and the whole expression is `None`. Returning
`Option<String>` (owned, cloned) rather than `Option<&String>` sidesteps
tying the result's validity to `users`'s borrow — a caller can hold onto
the returned name independently, which matters once this kind of lookup
moves into a real function boundary (e.g. across an API handler in Phase 3)
where the original `users` list might not still be around.

```rust
pub fn average_known_age(ages: &[Option<u32>]) -> Option<f64> {
    let known: Vec<u32> = ages.iter().filter_map(|a| *a).collect();
    if known.is_empty() {
        return None;
    }
    let sum: u32 = known.iter().sum();
    Some(sum as f64 / known.len() as f64)
}
```
`.filter_map()` combines filtering and mapping in one pass: give it a
closure returning `Option<U>`, and it keeps only the `Some` values,
already unwrapped — exactly "ignore every `None`, keep the rest." Returning
`None` for an empty input isn't defensive boilerplate; it's the only
type-correct answer — there is no sensible `f64` that means "the average of
zero numbers," and `Option<f64>` lets the caller's type system force them
to handle that case too, the same guarantee this whole lesson is about.
