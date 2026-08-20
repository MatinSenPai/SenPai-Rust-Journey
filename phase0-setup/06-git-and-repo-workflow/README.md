# 06 — Git and repo workflow

No new Rust here — this lesson is entirely about how to actually use this
repository over the coming months without losing track of where you are.

## The three files that make this repo self-explaining

- **`docs/conventions.md`** — the rulebook: how a lesson folder is laid out,
  the naming scheme, the workspace-glob quirks. Read once, re-check when
  something about the structure confuses you.
- **`PROGRESS.md`** — the master checklist, one checkbox per lesson, grouped
  by phase. A "Currently working on" line at the top means you can close
  this repo for three weeks and know exactly where to pick back up.
- Every phase's own `README.md` — the table of contents for that phase.

## The commit convention

One commit per completed lesson, message scoped by the lesson's package
name:

```
feat(p1-02-01-move-semantics): complete exercise
```

Flip that lesson's checkbox in `PROGRESS.md` in the **same commit**, so the
checklist and the actual git history can never drift apart — if you ever
wonder "did I actually finish this?", `git log --grep` for the package name
gives you a definitive answer.

```sh
git add phase1-fundamentals/02-ownership-and-memory/01-move-semantics PROGRESS.md
git commit -m "feat(p1-02-01-move-semantics): complete exercise"
```

## Tagging phase completions

Small, free, motivating waypoints:

```sh
git tag -a phase1-complete -m "Finished Phase 1: Fundamentals"
git push origin phase1-complete   # if/when you push tags
```

Months from now, `git log --oneline --tags` gives you a timeline of the
whole journey.

## Doc-only lessons and why some folders have no `Cargo.toml`

A few lessons (this one included) are pure reading — no code, no
`Cargo.toml`. That's deliberate, not incomplete: forcing every lesson to be
a compiling crate would mean padding pure-concept lessons with meaningless
code. The root `Cargo.toml` documents exactly which lessons are excluded
this way, right above `[workspace.dependencies]`.

## Working solo vs. practicing the PR workflow

You're the only contributor here, so there's no requirement to use branches
or pull requests. But since job-market readiness is an explicit goal of this
journey: consider optionally opening one PR per phase against `main` anyway,
purely to build comfort with the PR/diff-review workflow real teams use.
Low cost, realistic practice, entirely optional.

## Next

the recall questions in this folder, then you're done with Phase 0 — on to
[Phase 1](../../phase1-fundamentals/README.md).
