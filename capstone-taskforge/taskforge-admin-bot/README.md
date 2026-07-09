# taskforge-admin-bot

A Telegram bot as a ChatOps admin client for `taskforge-api` — the same
`teloxide` skills from
[`side-quests/sq-02-telegram-quiz-bot`](../../side-quests/sq-02-telegram-quiz-bot/README.md),
now pointed at real infrastructure instead of a quiz. Commands:

- `/status` — recent jobs, counted by status
- `/cancel <id>` — cancel a job by id

## Why only these two, when the ADR mentions `/retry` and `/pause_queue`?

`../docs/adr/0001-architecture-overview.md` sketches `/retry <id>` and
`/pause_queue` as the eventual command set. Only `/status` and `/cancel`
are implemented, honestly, because `taskforge-api` doesn't have `POST
/jobs/{id}/retry` or `POST /queue/pause` endpoints yet — this bot only
calls real, existing endpoints. Adding those two commands is a genuine,
well-scoped stretch extension:
1. Add `POST /jobs/{id}/retry` to `taskforge-api` (re-enqueue a
   `DeadLetter` job as `Pending`) and a corresponding `retry_job` method
   on `taskforge-core`'s `JobStore` trait.
2. Add a `retry_job`/`pause_queue` method to `ApiClient` (`src/client.rs`).
3. Add the matching `Command` variant in `src/main.rs`.

## Structure (same split as sq-02)

- `src/format.rs` — pure formatting logic (`format_status`,
  `format_cancel_result`), zero I/O, fully unit-tested.
- `src/client.rs` — a thin `reqwest`-based HTTP client for `taskforge-api`.
- `src/main.rs` — the actual `teloxide` bot wiring. Needs a real bot token
  (`TELOXIDE_TOKEN` env var — get one from
  [@BotFather](https://t.me/BotFather)) and network access to run; **not**
  exercised by `cargo test`.

## Running the tests

```sh
cargo test -p taskforge-admin-bot
```

## Running it for real

```sh
export TELOXIDE_TOKEN="your-bot-token-from-botfather"
export TASKFORGE_API_URL="http://localhost:8080"
export TASKFORGE_API_TOKEN="whatever-token-taskforge-api-was-started-with"
cargo run -p taskforge-admin-bot
```
