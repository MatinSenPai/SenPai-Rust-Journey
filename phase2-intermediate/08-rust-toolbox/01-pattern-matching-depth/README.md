# 08.1 — Pattern matching in depth

You've been writing `match` since Phase 1, but so far only its entry-level
form: match a variant, bind its fields, add a `_` fallback. Python 3.10's
`match`/`case` borrowed most of its design from languages like Rust, so if
you've used structural pattern matching in Python, about half of this
lesson will look familiar — and the other half is where Rust goes further,
because the compiler *proves* your match is exhaustive instead of hoping
you remembered a `case _:`.

The running model: a `LogEvent` enum for a web service's log stream —
requests, log messages, heartbeats.

## Match guards

A guard bolts an arbitrary boolean onto an arm — for conditions that
patterns can't express, like comparisons:

```rust
match event {
    LogEvent::Request { status, path, .. } if *status >= 500 => alert(path),
    _ => {}
}
```

Python has the same thing (`case Request(status=s) if s >= 500:`). One
Rust-specific catch: the compiler **ignores guards when checking
exhaustiveness**. `match x { n if n >= 0 => .., n if n < 0 => .. }` does
not compile, even though the two guards obviously cover everything — the
compiler can't reason about arbitrary booleans, so you still need an
unguarded arm.

## Or-patterns and `@` bindings

`|` lets one arm match several patterns; `@` lets you *test* a pattern and
*keep* the matched value at the same time:

```rust
match status {
    301 | 302 | 307 | 308 => "redirect".to_string(),
    n @ 400..=499 => format!("client error ({n})"),
    ...
}
```

Without `@` you'd have to choose: `400..=499 => ...` matches but loses the
actual code, while a bare `n => ...` keeps the code but matches everything.
`n @ 400..=499` does both. Python's `case 400 | 401 | 402:` has or-patterns
too, but no ranges and no `@` (its `as` capture is close, but Python won't
range-match integers at all). Two rules worth memorizing: arms are tried
top to bottom (so put `200` before `200..=299` if the exact code should
win), and every alternative of an or-pattern must bind the same names with
the same types — `Ok(n) | Err(n)` is legal only if both `n`s are the same
type.

## Nested destructuring

Patterns nest arbitrarily deep. To match "a `Message` whose severity is
`Error`", you don't bind `severity` and then compare it — you put the
variant *inside* the struct pattern:

```rust
LogEvent::Message { severity: Severity::Error, text } => format!("error: {text}"),
```

`severity: Severity::Error` is a sub-pattern, not an assignment — read it
as "the `severity` field must itself match `Severity::Error`". This works
through any depth of structs, enums, tuples, and references, and it's the
single biggest thing people underuse after learning basic `match`.

## Slice patterns

Slices and arrays match on shape:

```rust
match samples {
    [] => "no samples".to_string(),
    [only] => format!("1 sample: {only}ms"),
    [first, .., last] => format!("..."),
}
```

`..` means "any number of elements I don't care about", and you can name
the middle with `rest @ ..` if you need it. The compiler checks
exhaustiveness by length: `[]`, `[x]`, and `[x, .., y]` provably cover
lengths 0, 1, and 2+ — an `if/else if` chain on `.len()` with indexing
gets no such proof (and panics if you get an index wrong). This is
Python's `match [first, *rest]:` / iterable unpacking, but compile-checked.

## Binding modes (and the `ref` you'll rarely write)

Match a `&LogEvent` and write `LogEvent::Request { method, .. }` — what
type is `method`? It's `&String`, *not* `String`. When the scrutinee is a
reference, Rust automatically binds fields by reference instead of trying
to move them out of borrowed data. This is called "match ergonomics" or
default binding modes, and it's why you almost never see the older
explicit form, `LogEvent::Request { ref method, .. }`, in modern code.
`ref` still exists for the rarer case where you match an *owned* value but
only want to borrow a field from it (to avoid moving it). If you remember
one thing: **matching through a `&` gives you `&` bindings for free**, and
that's why `method_of` below needs an `.as_str()` before returning.

## Your task

Implement the four functions in `src/lib.rs` — each one forces one feature:

- `describe_status` — or-patterns, range patterns, `@` bindings, arm order.
- `noteworthy` — match guards + nested enum-inside-struct destructuring.
- `method_of` — binding modes: `method` arrives as `&String`.
- `summarize_samples` — slice patterns, exhaustive over lengths 0 / 1 / 2+.

The doc comments spell out the exact strings the tests expect.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
