# Solution

`is_readable` uses `matches!(status, Status::Cancelled)`, a macro that's
shorthand for a full `match` when all you need is "does this value match
this one pattern" as a `bool` — equivalent to
`match status { Status::Cancelled => true, _ => false }`, negated.

`latest_available_chapter` dereferences (`*latest_chapter`) because
`match status` on a `&Status` binds each field as a reference too
(`latest_chapter: &u32`) — `u32` is `Copy`, so `*latest_chapter` just reads
the value out directly, no borrow-checker complications.

On checkpoint question 2: a struct-with-optional-fields version —
```rust
struct Status {
    latest_chapter: Option<u32>,
    since_chapter: Option<u32>,
    total_chapters: Option<u32>,
    is_cancelled: bool,
}
```
— allows plainly nonsensical states the enum makes unrepresentable: both
`latest_chapter` and `total_chapters` set at once, or `is_cancelled: true`
while `latest_chapter` is still `Some(42)`, or all four fields `None`/
`false` with no indication of what that's even supposed to mean. This is
sometimes summarized as "make illegal states unrepresentable" — designing
your types so that a value which shouldn't exist literally cannot be
constructed, rather than trusting every piece of code that touches the
struct to maintain the invariant by convention.
