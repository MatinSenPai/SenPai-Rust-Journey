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

Two things carry the weight here. First, **arm order**: `200` and `200..=299` overlap, and arms are tried top to bottom, so the exact literal has to come first — swap them and `describe_status(200)` returns `"success (200)"`, failing the first test. The compiler doesn't complain about the overlap (both arms are still reachable), so this is on you. Second, the `@` bindings: `400..=499 => ...` alone would match but throw the code away, and a bare `n => ...` would keep the code but match *everything*. `n @ 400..=499` is the only way to get both "in this range" and "here is the actual value" from one pattern.

```rust
LogEvent::Message { severity: Severity::Error, text } => Some(format!("error: {text}")),
LogEvent::Message { severity: Severity::Warning, text } if text.contains("disk") => ...
```

In `noteworthy`, the `Error` arm needs **no guard at all** — "severity is Error" is structural, so it belongs in the pattern (`severity: Severity::Error` is a sub-pattern, not a comparison). The `Warning` arm needs both: the structural part in the pattern, and `text.contains("disk")` — which no pattern can express — as a guard. That's the line to keep in mind: shapes go in patterns, arbitrary booleans go in guards. Also note `if *status >= 500`: matching a `&LogEvent` makes `status` a `&u16`, so comparing it against a literal needs the deref. The final `_ => None` is mandatory even though the guards "obviously" cover what's wanted — guards don't count toward exhaustiveness, so without an unguarded catch-all this match wouldn't compile.

```rust
pub fn method_of(event: &LogEvent) -> Option<&str> {
    match event {
        LogEvent::Request { method, .. } => Some(method.as_str()),
        _ => None,
    }
}
```

The whole point of this function is one type: `method` is `&String`, not `String`, because matching through a reference binds by reference — a default binding mode. Bound by value, the pattern would try to move `method` out of data only borrowed — a compile error. `Some(method)` alone would give `Option<&String>`, so `.as_str()` bridges to the `&str` the signature promises.

```rust
match samples {
    [] => "no samples".to_string(),
    [only] => format!("1 sample: {only}ms"),
    [first, .., last] => format!("{} samples, first {first}ms, last {last}ms", samples.len()),
}
```

Slice patterns are checked for exhaustiveness *by length*: `[]` covers 0, `[only]` covers 1, `[first, .., last]` covers 2 and up. Delete the `[only]` arm and the compiler says `&[_]` is not covered — an `if`/`else` chain on `.len()` with indexing has the same gap and compiles fine, only panicking (or misreporting) at run time.

```rust
pub fn middle_samples(samples: &[u64]) -> &[u64] {
    match samples {
        [_, rest @ .., _] => rest,
        _ => &[],
    }
}
```

`[_, rest @ .., _]` requires at least two elements — one for each `_` — so a slice of length 0 or 1 falls to the `_` arm and returns an empty slice. For exactly two elements, `rest` is still an empty slice (nothing sits between two adjacent elements), which is why `middle_samples(&[7, 3])` is `&[]` too, not a panic or a special case to write by hand.

```rust
pub fn first_alert(events: &[LogEvent]) -> Option<String> {
    match events {
        [LogEvent::Request { status, path, .. }, ..] if *status >= 500 => {
            Some(format!("first event: server error {status} on {path}"))
        }
        _ => None,
    }
}
```

One arm does three jobs at once: the outer `[..., ..]` shape says "only the first element matters, whatever the rest are"; `LogEvent::Request { status, path, .. }` nested inside it destructures that first element the moment it's known to be a `Request`; and the guard, `if *status >= 500`, filters on a value no pattern could express. `events.first()` followed by a separate `match` on the `Option<&LogEvent>` it returns would need two matches (one implicit in `first()`'s `Option`, one explicit) to say the same thing — the slice pattern says it once.
