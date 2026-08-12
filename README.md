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

## New here and never written Rust? Read [`docs/START-HERE.md`](docs/START-HERE.md) first

It's a ten-minute, hand-holding walkthrough: what Rust is, how to set up, and
exactly how to work through a lesson. Everything below assumes you've read it.

## How this is organized

- **`docs/`** — start here. [`docs/START-HERE.md`](docs/START-HERE.md) is the
  newcomer's guide, `docs/setup-guide.md` gets your toolchain running,
  `docs/conventions.md` explains how every lesson is structured and the
  workspace/naming rules behind it, `docs/glossary.md` is a living
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
- **`web-ui/`** — tooling, not curriculum: the local web UI for reading all of
  the above in a browser and ticking lessons off (`cargo run -p course-ui` —
  see "Reading it in a browser" below).

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

## Reading it in a browser

There's a small local web UI that renders this whole repo as a browsable site
and lets you tick lessons off as you go. Run it from the repo root:

```sh
cargo run -p course-ui
```

That serves **http://127.0.0.1:5000** and opens your browser at it. The first
run compiles the server (a few seconds); after that it's instant. Stop it with
`Ctrl-C`.

Options:

| Flag | What it does |
|---|---|
| `--no-open` | Start the server but don't launch a browser |
| `--root <path>` | Serve a different checkout of this repo |

```sh
cargo run -p course-ui -- --no-open
```

### What you get

- **A sidebar with the full nested tree** — phases, module-groups and lessons,
  exactly as they're laid out on disk, collapsed except the branch you're in.
- **One page per lesson**, stacking its `README.md`, then `CHECKPOINT.md`, then
  its reference solution behind a *Show the reference solution* toggle — so the
  order [`docs/conventions.md`](docs/conventions.md) recommends is still the
  order you meet things in.
- **A "Mark complete" button** at the bottom of each lesson. Completed lessons
  get a checkmark and a line through them in the sidebar; phases and
  module-groups show how far in you are (`3/6`) and only strike through once
  every lesson beneath them is done.
- `docs/` and the ADR folders are browsable too — readable, but not tickable,
  since they aren't lessons.

Nothing is cached: edit a `README.md` or add a new lesson folder and it shows up
on refresh.

### Where your progress is stored

In `.course-progress.json` at the repo root, keyed by lesson directory path.
It's **gitignored**, so it's yours alone and a fresh clone starts empty — you
won't inherit anyone else's checkmarks, and you can delete the file to reset.

This is the web UI's source of truth. [`PROGRESS.md`](PROGRESS.md) is the
secondary, hand-maintained checklist and the server never writes to it, so the
two can drift if you tick boxes in both places. The reasoning behind that split
is in [`docs/adr/0001-web-ui-progress-state.md`](docs/adr/0001-web-ui-progress-state.md).

### If port 5000 is busy

The port is fixed so the URL stays bookmarkable, and the server exits with
`could not bind 127.0.0.1:5000` rather than quietly moving elsewhere. On macOS
the usual culprit is AirPlay Receiver — turn it off in
*System Settings → General → AirDrop & Handoff*.

On Windows PowerShell, stop the process currently listening on port 5000, then
restart the UI:

```powershell
Get-NetTCPConnection -LocalPort 5000 -State Listen |
  ForEach-Object { Stop-Process -Id $_.OwningProcess -Force }
cargo run -p course-ui
```

The whole thing is optional: every lesson reads fine as plain markdown on GitHub
or in your editor.
