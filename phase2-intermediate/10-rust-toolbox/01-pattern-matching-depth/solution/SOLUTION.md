# Solution

```rust
pub fn describe_status(status: u16) -> String {
    match status {
        200 => "OK".to_string(),
        n @ 200..=299 => format!("success ({n})"),
        301 | 302 | 307 | 308 => "redirect".to_string(),
        n @ 400..=499 => format!("client error ({n})"),
        n @ 500..=599 => format!("server error ({n})"),
        other => format!("unrecognized status {other}"),
    }
}
```

Two things are load-bearing here. First, **arm order**: `200` and
`200..=299` overlap, and match arms are tried top to bottom, so the exact
literal has to come first — swap them and `describe_status(200)` returns
`"success (200)"`, failing the first test. The compiler doesn't complain
about the overlap (both arms are still reachable), so this is on you.
Second, the `@` binding: `400..=499 => ...` alone would match but throw
the code away, and a bare `n => ...` would keep the code but match
*everything*. `n @ 400..=499` is the only way to get both "in this range"
and "here's the actual value" from a single pattern.

```rust
LogEvent::Message { severity: Severity::Error, text } => Some(format!("error: {text}")),
LogEvent::Message { severity: Severity::Warning, text } if text.contains("disk") => ...
```

In `noteworthy`, the `Error` arm needs **no guard at all** — the
requirement "severity is Error" is structural, so it belongs in the
pattern (`severity: Severity::Error` is a sub-pattern, not an
assignment). The `Warning` arm needs both: the structural part in the
pattern, and `text.contains("disk")` — which no pattern can express — as
a guard. That's the dividing line to internalize: shapes go in patterns,
arbitrary booleans go in guards. Note also `if *status >= 500`: since we
matched a `&LogEvent`, default binding modes made `status` a `&u16`, so
comparing it against a literal needs the deref (or `>= &500`, which works
but reads worse). The final `_ => None` is mandatory even though our
guards "obviously" cover what we want — the compiler ignores guards when
proving exhaustiveness, so without an unguarded catch-all this match
doesn't compile.

```rust
pub fn method_of(event: &LogEvent) -> Option<&str> {
    match event {
        LogEvent::Request { method, .. } => Some(method.as_str()),
        _ => None,
    }
}
```

The whole point of this function is one type: `method` is `&String`, not
`String`, because matching through a reference binds by reference
("match ergonomics"). If it bound by value, the pattern would try to move
`method` out of data we've only borrowed — a compile error. Pre-2018 Rust
made you write `ref method` to get this; today it's automatic, and `ref`
survives only for the rare owned-scrutinee-but-borrow-a-field case.
`Some(method)` would give `Option<&String>`, so `.as_str()` bridges to
the `&str` the signature promises (returning `&str` is the more flexible
API — callers with a `String`, a literal, or a slice can all compare
against it).

```rust
match samples {
    [] => "no samples".to_string(),
    [only] => format!("1 sample: {only}ms"),
    [first, .., last] => format!("{} samples, first {first}ms, last {last}ms", samples.len()),
}
```

Slice patterns are checked for exhaustiveness *by length*: `[]` covers 0,
`[only]` covers 1, `[first, .., last]` covers 2 and up (it needs at least
one element for `first` and one for `last`, and `..` absorbs the middle —
including an empty middle for exactly two elements, which is why
`summarize_samples(&[7, 3])` works). Delete the `[only]` arm and the
compiler tells you `&[_]` is not covered — an `if samples.len() == 0 /
else if / else` chain with `samples[0]` indexing would compile fine with
the same gap and just panic (or silently misreport) at runtime. That
compile-time proof is the practical reason to prefer slice patterns over
`.len()` checks.
