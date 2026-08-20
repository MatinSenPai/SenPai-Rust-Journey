# 05.2 — Enums and match

## Enums model "exactly one of these shapes"

```rust
enum WatchStatus {
    PlanToWatch,
    Watching { episode: u32 },
    Completed { rating: u8 },
    Dropped,
}
```

This is a genuinely different (and more precise) tool than Python usually
reaches for. A Python developer might model this with a string field
(`status = "watching"`) plus separate, loosely-related fields (`episode`,
`rating`) that only make sense for *some* statuses — nothing stops you from
setting `rating` while `status == "watching"` by mistake. Rust's enum makes
that mistake structurally impossible: `Watching` *carries* an `episode`,
`Completed` *carries* a `rating`, and there is no way to construct a
`WatchStatus` that has a `rating` without being `Completed`. Each variant
can carry completely different associated data (or none, like
`PlanToWatch`) — this is often called a "sum type" or "tagged union," and
it's one of Rust's most-used features for real backend domain modeling.

## `match` unpacks enums exhaustively

```rust
fn describe(status: &WatchStatus) -> String {
    match status {
        WatchStatus::PlanToWatch => "not started yet".to_string(),
        WatchStatus::Watching { episode } => format!("on episode {episode}"),
        WatchStatus::Completed { rating } => format!("finished, rated {rating}/10"),
        WatchStatus::Dropped => "dropped".to_string(),
    }
}
```

Each `match` arm both **checks** which variant it is and **destructures**
any data that variant carries (`{ episode }` binds the `Watching` variant's
`episode` field directly, in one step). Add a fifth variant to the enum
later and forget to update this `match`? The compiler refuses to build —
exhaustiveness checking again, now doing real work: it's now telling you
"you added a new state and forgot to handle it somewhere," a genuine class
of bug (especially in growing backend systems) caught at compile time
instead of surfacing as a runtime `else` branch nobody wrote.

## Your task

`src/lib.rs` defines a `Status` enum for tracking a manga/webtoon's release
state. Implement the functions that pattern-match on it.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
