# Solution

```rust
pub fn sorted_by_key(counts: HashMap<String, u32>) -> BTreeMap<String, u32> {
    counts.into_iter().collect()
}
```

The interesting part is what *doesn't* appear here: no `.sort()`, no
comparator, nothing. `counts.into_iter()` yields `(String, u32)` pairs in
whatever unspecified order the `HashMap` happens to produce them in, but
`.collect()` is generic over its target type — here inferred as
`BTreeMap<String, u32>` from the function's return type — and `BTreeMap`'s
`FromIterator` implementation inserts each pair one at a time into its
internal sorted tree structure, exactly the same as if you'd called
`.insert()` in a loop. The sortedness is a property of *where the data
ends up*, not of the order it arrived in — which directly answers
checkpoint question 1: nothing "leaks through" from the `HashMap`'s
iteration order because `BTreeMap` re-establishes its own order on every
insert, regardless of insertion order.

```rust
pub fn shared_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    a.intersection(b).cloned().collect()
}
```

`a.intersection(b)` returns an iterator of `&String` — references borrowed
*from `a`* (not new allocations), because `HashSet` doesn't know you want
to keep them beyond this function call. This function's signature promises
an *owned* `HashSet<String>` though (checkpoint question 3), so the
references can't be returned as-is — the borrow wouldn't outlive `a` and
`b` being dropped at the end of the caller's scope, and the return type
doesn't even mention a lifetime for `&str` to borrow against. `.cloned()`
converts each `&String` into an owned `String` (cloning the underlying heap
allocation), which is what makes `.collect::<HashSet<String>>()` type-check
at all.

On checkpoint question 4: with a plain `Vec<String>`, "jump to the front"
is `vec.insert(0, title)`, which is `O(n)` — every existing element has to
shift one slot to the right to make room at index `0`. `VecDeque::push_front`
is `O(1)` (amortized) because a deque is internally a ring buffer with
slack at both ends, so adding to the front just claims the next free slot
behind the current front, no shifting required.
