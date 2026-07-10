# SenPai-Rust-Journey

A complete, self-paced Rust curriculum — from "what is a compiled language"
to shipping a production-grade backend system — built for a learner coming
from Python/Django with zero prior systems-programming background.

This isn't a syntax tour. The goal is to come out the other side as a
**software engineer**: someone who understands ownership and memory well
enough to reason about it without a garbage collector, who can design a
backend service (APIs, databases, caching, queues, auth) and explain the
system-design trade-offs behind it, and who has a genuinely marketable
capstone project to show for it.

## How this is organized

- **`docs/`** — start here. `docs/setup-guide.md` gets your toolchain
  running, `docs/conventions.md` explains how every lesson is structured and
  the workspace/naming rules behind it, `docs/glossary.md` is a living
  plain-English dictionary for new jargon.
- **`phase0-setup/`** → **`phase4-backend-advanced/`** — the main curriculum,
  in order. Each phase folder has its own `README.md` table of contents.
- **`side-quests/`** — themed, motivational mini-projects (Telegram bots,
  anime/manga/webtoon data) slotted in right after the phase that unlocks
  the skills they need. Optional in the sense that skipping one doesn't
  block later phases, but they're where the fundamentals stop feeling
  abstract — worth doing.
- **`capstone-taskforge/`** — the flagship project: **TaskForge**, a
  Postgres-backed background job/task-queue engine (the Rust equivalent of
  Sidekiq/Celery/BullMQ — a real, employable infra category, not a toy).
- **`PROGRESS.md`** — the master checklist across every phase. Check this
  first when picking the journey back up after a break.

## The path, phase by phase

| Phase | Theme |
|---|---|
| [Phase 0](phase0-setup/README.md) | Setup & orientation — installing Rust, cargo, tooling, first program |
| [Phase 1](phase1-fundamentals/README.md) | Core fundamentals — ownership, borrowing, strings, structs/enums, `Option`/`Result` |
| [Phase 2](phase2-intermediate/README.md) | Intermediate & idiomatic Rust — collections, iterators, generics/traits, lifetimes, smart pointers, concurrency, async |
| [Phase 3](phase3-backend-foundations/README.md) | Backend foundations — HTTP, `axum`, PostgreSQL/`sqlx`, auth, testing |
| [Phase 4](phase4-backend-advanced/README.md) | Backend advanced + system design — caching, rate limiting, queues, gRPC/GraphQL, observability, deployment |
| [Capstone](capstone-taskforge/README.md) | TaskForge — a production-grade job queue engine |
| [Phase 5](phase5-system-design-mastery/README.md) | System design mastery — protocols, databases at scale, caching, distributed systems patterns, security, DevOps/cloud, and hands-on "design X" projects, curated from [ByteByteGo's System Design 101](https://github.com/ByteByteGoHq/system-design-101) and tied back to code you've already built |

Side quests live in [`side-quests/`](side-quests/README.md) and slot in
between phases.

## Quick start

```sh
# one-time setup — see docs/setup-guide.md for details
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

cargo build --workspace   # builds every lesson crate that exists so far
```

Then open `phase0-setup/README.md` and start at lesson 1.
