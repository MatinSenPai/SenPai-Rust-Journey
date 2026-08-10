# ADR-0001: The web UI tracks progress in a gitignored state file, not `PROGRESS.md`

## Status

Accepted

## Context

`PROGRESS.md` has been this repo's progress tracker since the beginning: 121
markdown checkboxes, and `docs/conventions.md` step 7 tells you to tick one and
commit it in the same commit as the lesson. It works, it lives in git, and it
needs no tooling.

Adding a local web UI (`web-ui/`, package `course-ui`) that marks lessons
complete forces the question of where that state lives. Two constraints pull in
opposite directions:

1. **This repo is used by more than one person.** It's a curriculum other
   learners are meant to clone and work through — not a personal notebook.
2. **The web UI needs to map "completed" onto something the filesystem
   enforces**, because it derives its navigation tree from the filesystem.

Making `PROGRESS.md` the source of truth means the server parses it and rewrites
`- [ ]` → `- [x]` on click. The blocker is the mapping: `PROGRESS.md` says
`- [ ] 01 — Move semantics` while the filesystem says
`phase1-fundamentals/02-ownership-and-memory/01-move-semantics/`. Linking the two
requires fuzzy-matching prose titles against directory names — which only works
because the numbering happens to line up today, and breaks the first time a
title is reworded. The two sets aren't even the same size or shape: the
filesystem has **116 leaf nodes**, while `PROGRESS.md` tracks **121** items,
including entries with no directory at all (`ADRs read`, three individual
`main.rs` files, unbuilt Phase 5 designs). There is no clean bijection to
maintain, so any sync mechanism would be lossy in both directions.

## Decision

**A separate state file, `.course-progress.json`, is the source of truth for the
web UI, and it is gitignored.**

Schema — a `version` field for future migrations, and a sorted array of
repo-relative lesson directory paths:

```json
{
  "version": 1,
  "completed": [
    "phase0-setup/01-what-is-a-compiled-language",
    "phase1-fundamentals/02-ownership-and-memory/01-move-semantics"
  ]
}
```

Keys are **directory paths, not titles** — a path is a fact the filesystem
enforces, unlike a prose heading. Unknown/orphaned keys (left behind when a
lesson directory is renamed) are preserved on load and never pruned, so a rename
costs one re-tick rather than silently deleting history, and the file stays safe
to hand-edit.

**`PROGRESS.md` stays exactly as it is** — untouched by the server, still
tickable by hand — but is explicitly demoted to the secondary tracker, with a
note saying so in both `PROGRESS.md` and `docs/conventions.md` step 7.

## Considered options

- **Rewrite `PROGRESS.md` on click (rejected).** The prose-title-to-directory
  matching described above, now on the write path, where a bad match corrupts a
  tracked file rather than merely displaying the wrong thing.
- **Commit `.course-progress.json` (rejected).** This was the initial
  recommendation, and it's right for a single-user repo: progress survives a
  fresh clone and follows you between machines. It's wrong here — the moment
  other learners clone this repo, a committed progress file ships one person's
  checkmarks to everyone, so the first thing a newcomer must do is wipe someone
  else's state. It also makes progress a merge-conflict surface and leaves a
  permanently dirty working tree.
- **Strip the checkboxes from `PROGRESS.md` (rejected).** Would leave people who
  don't want to run a server with no tracker at all.

## Consequences

- **Positive**: one writer, one format, no parser guessing. Every learner gets a
  clean slate on clone. The web UI's state is keyed by something the filesystem
  guarantees, so it can't drift out of sync with the navigation tree it's
  rendered against.
- **Negative — accepted knowingly**: progress does **not** survive a fresh clone
  and does **not** follow you between machines. Fixing that would need a
  committed file with per-user keys, which is a meaningfully larger build than
  the "very basic and minimal" tool this is meant to be.
- **Negative — accepted knowingly**: there are now **two trackers that can
  disagree** — the exact thing this design otherwise avoids. It's tolerable only
  because one is explicitly labelled secondary in both places a reader would
  look. If they drift badly enough to cause confusion in practice, that's the
  signal to revisit.
- **Revisit trigger**: if cross-machine progress becomes a real need, or if the
  two trackers cause actual confusion, reopen this — most likely by generating
  `PROGRESS.md`'s checkbox state from `.course-progress.json` as a one-way build
  step, rather than by parsing markdown.
