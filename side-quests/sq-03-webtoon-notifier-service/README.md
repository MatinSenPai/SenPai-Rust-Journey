# Side-quest 3 — Webtoon Notification Service

You know the feeling: a webtoon you follow drops a new chapter and you find
out three days late because you forgot to check. This side-quest builds the
service that would fix that — the "motivational checkpoint" Phase 3's
README points to right after the `axum` CRUD lesson, previewing the
background-jobs module that's still ahead of you in Phase 4.

This is everything from Phase 3's `axum` module (routing, extractors,
in-memory `Mutex`-guarded state, `IntoResponse` error handling) plus your
first taste of a **scheduled background job** — a `tokio::spawn` task that
wakes up on a timer, does work, and logs what it found, running alongside
your HTTP server the whole time the process is up.

## What it does

A tiny REST API for tracking webtoons you follow, plus a background poller:

```
POST /webtoons               follow a webtoon (title + the chapter you're currently on)
GET  /webtoons                list everything you follow
POST /webtoons/{id}/check     check one webtoon right now, on demand
```

...and, running the whole time the server is up, a background task that
calls the same "check for a new chapter" logic for *every* followed
webtoon on a timer, logging (via `tracing`) whenever it finds one.

**A note on realism:** there's no public API this project can depend on to
actually know whether, say, chapter 180 of *Solo Leveling* is out yet. So
every "check for a new chapter" here is simulated through a small
`ChapterChecker` trait — some implementations are deterministic (for
tests), one rolls dice (for a fun demo). A real version of this service
would implement `ChapterChecker` by calling `reqwest::get(...)` against a
tracking API or scraping a source site's chapter list; the trait boundary
is exactly where that swap would happen, and nothing else in this codebase
would need to change.

## Structure

- `src/lib.rs` — everything: `Webtoon`, `WebtoonStore` (the in-memory,
  `Mutex`-guarded catalog — same shape as Phase 3's `AnimeStore`), the
  `ChapterChecker` trait and its three implementations, `check_all_webtoons`
  (one round of "check everything, update what changed"), `spawn_notifier`
  (the actual background job loop), and the axum handlers/router. Almost
  all of it is already implemented and unit/integration-tested — see below
  for exactly what's left for you.
- `src/main.rs` — wires it all together and starts the server. Fully
  written for you; read it once you're done to see how little glue code it
  takes to turn your library into a real running service.
- `tests/` — `store_test.rs` (the catalog), `notifier_test.rs` (the "did
  anything change" check logic), `api_test.rs` (the HTTP layer end to end).

## What's already done for you

- `WebtoonStore` — `follow`, `get`, `list`, `update_chapter`. Fully
  implemented; read it as a refresher on the `Mutex<HashMap<...>>` pattern.
- `ChapterChecker` and its three implementations (`AlwaysNewChapterChecker`,
  `NeverNewChapterChecker`, `RandomChapterChecker`) — fully implemented.
- `spawn_notifier` — the actual `tokio::spawn` + `tokio::time::interval`
  background loop. Fully implemented, and deliberately *thin*: all it does
  is tick, call `check_all_webtoons`, and log whatever came back. That
  separation (timer loop vs. the logic it runs) is exactly what makes the
  logic testable without a real timer — see the next section.

## Your task

Four `todo!()`s, in order of how interesting they are:

1. **`check_all_webtoons`** — for every followed webtoon, ask the checker
   if there's a new chapter; if the reported chapter number is *strictly
   greater* than what's stored, update the store, record the event, and log
   it. This is the actual "brain" of the notifier — everything else is
   plumbing around this one function. Notice it's a plain `async fn`, not
   the timer loop itself: that's deliberate, so `notifier_test.rs` can call
   it directly and assert on its behavior without ever waiting on a real
   `tokio::time::interval` tick.
2. **`check_webtoon`** — the `POST /webtoons/{id}/check` handler: look the
   webtoon up (404 if it's not followed), run the checker once, update the
   store if there's a new chapter, and report what happened.
3. **`follow_webtoon`** — the `POST /webtoons` handler.
4. **`app`** — wire the two routes to their handlers, same shape as Phase
   3's anime catalog `app` function.

## Checkpoint

`cargo test -p sq-03-webtoon-notifier-service`. Then actually run it:
`cargo run -p sq-03-webtoon-notifier-service`, follow a webtoon with the
`curl` command it prints, and watch the terminal — every ~10 seconds
you'll see a `new chapter detected` log line about 30% of the time (that's
`RandomChapterChecker`'s `probability: 0.3`). Try `POST
/webtoons/{id}/check` a few times too, to see the on-demand path.

Then `CHECKPOINT.md`, then `solution/SOLUTION.md`.
