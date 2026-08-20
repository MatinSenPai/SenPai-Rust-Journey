# Persian learning experience — execution index

Planned at `80ec4d1` on 2026-08-12 using the `shadcn/improve` handoff format.
Execute in order; every plan has an explicit verification gate.

| Plan | Outcome | Depends on | Status |
|---|---|---|---|
| 001 | Persian-first SSR, locale routes, dashboard, search, SVG and assets | — | VERIFYING |
| 002 | Phase 0–1 Persian curriculum and hard ownership/borrowing explanations | 001 | COMPLETE |
| 003 | Phase 2–4 and TaskForge Persian curriculum | 001, 002 | IN PROGRESS |
| 004 | Phase 5, docs, side quests and repository-wide quality audit | 001–003 | TODO |
| 005 | Rebuild the curriculum into a teaching course (format, syllabus, web UI, gates) | — (supersedes 001–004 pedagogy) | PLANNED |

## Shared verification gates

```sh
cargo fmt --all -- --check
cargo clippy -p course-ui --all-targets -- -D warnings
cargo test -p course-ui
cargo test --workspace --no-run
```

Do not add React, Tailwind, runtime translation, a CDN, user accounts or cloud
sync.

> **Superseded by plan 005** (2026-08-20): the constraints "no progress-schema
> migration" and "preserve the existing lesson/checkpoint/gated-solution
> workflow" no longer hold. Plan 005 replaces the lesson format, removes
> `CHECKPOINT.md` entirely, and migrates `.course-progress.json` to schema v2.
> Plans 001–004 remain the record of how the Persian SSR shell and the first
> translation pass were built; their pedagogy assumptions are no longer current.
