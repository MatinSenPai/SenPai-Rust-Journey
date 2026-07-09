# 05.3 — `if let` / `while let`

## When a full `match` is overkill

```rust
let maybe_rating: Option<u8> = Some(9);

// Full match, when you only actually care about one case:
match maybe_rating {
    Some(r) => println!("rated {r}/10"),
    None => {}
}

// The same thing, more directly:
if let Some(r) = maybe_rating {
    println!("rated {r}/10");
}
```

`if let PATTERN = value { ... }` runs the block only if `value` matches
`PATTERN`, binding any data the pattern captures — it's a `match` with
exactly one arm you care about and a silent do-nothing fallback. Reach for
`match` when every case needs handling; reach for `if let` when you
genuinely only care about one shape and want to ignore the rest.

`if let` can carry an `else`:

```rust
if let Some(r) = maybe_rating {
    println!("rated {r}/10");
} else {
    println!("not rated yet");
}
```

## `while let` — loop as long as a pattern keeps matching

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("popped {top}");
}
// prints 3, 2, 1, then stops once stack.pop() returns None
```

`Vec::pop` removes and returns the last element as `Some(T)`, or `None`
once the vec is empty. `while let Some(top) = stack.pop()` keeps looping
exactly as long as that pattern keeps matching — a very common way to drain
a collection or work through anything shaped like "keep pulling `Some`
values until you hit `None`."

## Your task

Implement the functions in `src/lib.rs`.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`. That's Structs, Enums &
Pattern Matching complete — and with it, everything Side-quest 1 (Anime
Quote CLI) needs. Do that next, then move on to
[Option, Result & error basics](../06-option-result-error-basics/README.md).
