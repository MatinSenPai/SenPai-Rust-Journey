# Side-quest 2 — Telegram Quiz Bot

Your first real async project — everything from Phase 2 (closures/iterators,
error handling, and especially concurrency/async) put to work, plus a real
`teloxide` bot wired up for the first time. Same consolidation spirit as
side-quest 1: no new *concepts*, just putting recent ones to work on
something more fun than another exercise.

## What it does

A Telegram bot that quizzes you with anime/manga trivia, one question at a
time, tracking your score across a session:

```
/quiz          start a new quiz
(reply 1-4)    answer the current question
/score         see your current score mid-quiz
```

## Structure (same split as `taskforge-admin-bot` and `taskforge-cli`)

- `src/lib.rs` — `QuizSession`, the question bank, and all scoring/state
  logic. Zero I/O, zero `teloxide` dependency, fully unit-tested.
- `src/main.rs` — the actual `teloxide` bot wiring: per-chat session
  storage, command handling, sending messages. Needs a real bot token and
  network access to run; not exercised by `cargo test`.

## Your task

Implement `QuizSession`'s methods in `src/lib.rs` (the question bank itself
is provided). `src/main.rs` is fully written for you — read through it once
you're done to see how a small amount of `teloxide` wiring turns your
tested `QuizSession` into an actual running bot.

## Checkpoint

`cargo test -p sq-02-telegram-quiz-bot`. If you want to see it actually
run: get a bot token from [@BotFather](https://t.me/BotFather),
`export TELOXIDE_TOKEN=...`, `cargo run -p sq-02-telegram-quiz-bot`. Then
`CHECKPOINT.md`, then `solution/SOLUTION.md`.
