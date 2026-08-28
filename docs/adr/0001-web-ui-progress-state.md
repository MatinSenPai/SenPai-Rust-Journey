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

---

## Amendment — schema v2 (2026-08-21, plan 005)

v1 stored one set of completed lesson paths. That answered "did I tick this?"
and nothing else, which is exactly the shallow signal
[`plans/005-curriculum-rebuild.md`](../../plans/005-curriculum-rebuild.md) set
out to replace.

**v2 stores a record per lesson**: `status`, `first_seen_at`, `completed_at`,
the exercise rungs ticked, a self-rated `confidence`, and a free-text `note`.
That is what `/{locale}/progress` is derived from, together with
`docs/concept-map.toml` for the concept-mastery grid.

The two properties v1 guaranteed are unchanged: keys are still repo-relative
directory paths, and orphaned keys are still preserved rather than pruned. A v1
file migrates on read and is rewritten as v2 on the next save; the migration is
covered by `a_version_1_file_migrates_without_losing_completions`.

### Two durability rules learned the hard way

Adding a write to the GET path (recording a lesson's first view) made two
latent problems reachable, and both cost real data before they were fixed:

1. **Writes are atomic.** `fs::write` truncates before writing, so a request
   reading the file while another was writing could see it empty, load nothing,
   and then save *that* back over a file holding real completions. `save` now
   writes a sibling `.tmp` and renames over the target.
2. **An unparseable file is never overwritten.** `load` distinguishes *missing*
   (fine — a fresh clone) from *present but unreadable*. In the second case the
   UI still renders from an empty state, but `save` refuses with
   `InvalidData` rather than replacing a file it failed to understand. This
   file is gitignored, so a bad write has nothing to recover from.

Both are covered by tests in `web-ui/src/progress.rs`, and the load-modify-save
sequence itself is serialised behind a process-wide lock in `AppState`.

**`--no-track`** serves every page identically while recording nothing. It
exists for browsing a checkout whose progress file isn't yours — and the route
tests use it, because they run against this very repository.
