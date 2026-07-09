# 06.3 — `panic!` vs `Result`

You now have two genuinely different tools for "something went wrong," and
this lesson is about choosing correctly between them — a real design
decision you'll make constantly starting in Phase 3.

## `panic!` — unrecoverable, stops the current thread

```rust
fn get_config_value(map: &std::collections::HashMap<String, String>, key: &str) -> String {
    map.get(key)
        .unwrap_or_else(|| panic!("missing required config key: {key}"))
        .clone()
}
```

`panic!` (and everything built on it — `.unwrap()`, `.expect()`, indexing
out of bounds, integer overflow in debug builds) immediately stops the
current thread, unwinding and running `Drop` for every value in scope on
the way out (RAII, from Phase 1's ownership module — cleanup still happens
correctly even during a panic). In a web server, one request's handler
panicking typically takes down *that request*, not the whole server — but
it's still an abrupt, "something is so wrong I'm not going to try to
continue" signal.

## `Result` — recoverable, the caller decides what to do

`Result` says "this can fail, and I'm handing the decision about what to do
next to whoever called me" — they might retry, log and continue with a
default, or propagate the error further up (`?`, previous lesson).

## The actual decision rule

Reach for `panic!` (or `.unwrap()`/`.expect()`) when:
- The failure represents a **bug** in the program itself, not bad input —
  an invariant your own code is supposed to guarantee was violated.
- You're in test code, a quick script, or a prototype where "just crash and
  show me the error" is genuinely the right behavior.
- Continuing would be unsafe or meaningless (e.g. a config file the whole
  program depends on is missing at startup — there's no sensible way to
  "handle" that and keep running).

Reach for `Result` when:
- The failure is **expected, ordinary, and about external input or the
  outside world** — a user submitted invalid data, a network call timed
  out, a file doesn't exist yet. This is nearly everything in Phase 3-4's
  backend work: a malformed HTTP request body is not a bug in your server,
  it's Tuesday.
- The caller might reasonably want to do something other than crash —
  retry, show a friendly error message, fall back to a default.

`.expect("message")` over `.unwrap()`: always prefer `.expect` with a
message that explains *why* you believed this couldn't fail — when it does
turn out to fail anyway (bugs happen), a message like `"config file was
validated at startup"` next to the panic tells the next person exactly what
assumption broke, instead of a bare `unwrap()` panic with no context at all.

## Your task

`src/lib.rs` has three functions — decide, for each, whether `panic!` or
`Result` is the right tool, matching the doc comment's description of what
each represents.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`. Phase 1 complete — go do
[Side-quest 1: Anime Quote CLI](../../../side-quests/sq-01-anime-quote-cli/README.md),
then move on to [Phase 2](../../../phase2-intermediate/README.md).
