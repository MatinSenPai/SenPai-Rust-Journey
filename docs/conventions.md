# Conventions

This document is the single source of truth for how every lesson in this repo
is structured. Read it once before starting Phase 0, then again before Phase 3
(a couple of things — like the workspace-member glob rule — only start to
matter once you're writing real crates).

## Directory shape

```
SenPai-Rust-Journey/
├── phase0-setup/                <phase>
│   └── 03-cargo-basics/         <lesson>              (2 levels deep here)
├── phase1-fundamentals/
│   └── 02-ownership-and-memory/ <module-group>
│       └── 01-move-semantics/   <lesson>               (3 levels deep here)
├── side-quests/
│   └── sq-01-anime-quote-cli/   <standalone project>
└── capstone-taskforge/
    └── taskforge-core/          <capstone sub-crate>
```

- **Phase** = a season of the journey (`phase0-setup` … `phase4-backend-advanced`).
  Has its own `README.md` acting as a table of contents.
- **Module-group** = a cluster of closely related lessons (e.g. "ownership and
  memory"). Has its own short `README.md` framing the theme.
- **Lesson** = the atomic unit you actually work through. Not every lesson
  compiles — some early Phase 0 lessons are pure reading (installing Rust,
  how git/this repo works) and have no `Cargo.toml`.

## Anatomy of a lesson (when it has code)

```
0N-slug/
├── Cargo.toml
├── README.md         # theory, goals, why it matters, links to further reading
├── src/
│   └── lib.rs         # starter code: real signatures, `todo!()` where you fill in logic
├── tests/             # OR #[cfg(test)] inline in src/lib.rs — see rule below
├── CHECKPOINT.md      # 3-6 short-answer questions, answer them in your own words
└── solution/          # a full second crate: reference solution + reasoning
    ├── Cargo.toml
    ├── src/
    └── SOLUTION.md
```

**Unit tests vs. integration tests** — deliberately used to teach the
distinction as it comes up:
- Single-concept "kata" lessons (most of Phase 1-2) put tests inline as
  `#[cfg(test)] mod tests { ... }` at the bottom of `src/lib.rs`. Fast
  feedback, and it's the idiomatic way to test a small crate's internals.
- "Mini-project" lessons (CRUD APIs, the toy job queue, etc.) put tests in
  `tests/*.rs` — these only see the crate's `pub` surface, exactly like a real
  consumer of the crate would. This is introduced deliberately in
  `phase2-intermediate/06-project-organization-and-testing/02-unit-integration-doc-tests`.

## Naming & the workspace-member glob rule

The whole repo is **one Cargo workspace** (single `Cargo.lock`, single
`target/`, so shared dependencies like `tokio` compile once, not 90 times).
The root `Cargo.toml` picks up lessons with glob patterns like
`"phase1-fundamentals/*/*"`.

Two consequences you should know about:

1. **Cargo package names can't start with a digit**, even though folder names
   do (`01-move-semantics/`). So every lesson's `Cargo.toml` uses a prefixed
   package name: `p{phase}-{module}-{lesson}-{slug}` for 3-level lessons
   (e.g. `p1-02-01-move-semantics`) or `p{phase}-{lesson}-{slug}` for 2-level
   ones (e.g. `p0-03-cargo-basics`). Side-quests use `sq-0N-slug`. Capstone
   sub-crates use `taskforge-<role>`.
2. **A glob member pattern that matches zero directories is a hard error**
   (confirmed against cargo 1.94.1 — `cargo metadata`/`build` fails with
   "failed to read .../Cargo.toml", not a silent no-op). So a phase's glob
   is only added to the root `Cargo.toml`'s `members` list once at least one
   real lesson crate exists under it. If you add a brand-new phase folder
   from scratch, create its first lesson crate *before* adding the glob.
3. **A glob only matches the depth you write.** `"phase1-fundamentals/*/*"`
   matches `phase1-fundamentals/<module-group>/<lesson>/` exactly — it does
   **not** recurse into `<lesson>/solution/`. That's not an oversight: it's
   why `solution/` crates are never accidentally picked up as workspace
   members (no double-compiling, no name collisions, and you won't see
   solution code in `cargo build --workspace` output by accident).
   Every `solution/Cargo.toml` has its **own empty `[workspace]` table** —
   without it, Cargo walks up, finds the root workspace, notices the crate
   isn't in its `members`, and refuses to build with "current package
   believes it's in a workspace when it's not" (confirmed empirically; this
   is Cargo's real behavior, not a hypothetical). The empty table tells Cargo
   "this crate is its own workspace root, stop looking further up." One
   consequence: a `solution/` crate is a fully standalone workspace, so it
   **cannot** use `some_dep.workspace = true` — it pins its own explicit
   dependency versions instead (matching whatever the root
   `[workspace.dependencies]` currently pins, but written out literally).
   Build/test a solution directly with:
   `cargo test --manifest-path phase1-fundamentals/02-ownership-and-memory/01-move-semantics/solution/Cargo.toml`.
4. Every lesson without a `Cargo.toml` (pure-reading lessons) must **not** be
   matched by a glob at all, or `cargo build --workspace` fails with a missing
   `Cargo.toml` error. The root `Cargo.toml` documents exactly which lessons
   are excluded this way in a comment above `[workspace.dependencies]`.

## Working through a lesson

1. Read the lesson's `README.md` end to end before touching code.
2. Open `src/lib.rs`, read the starter code and its doc comments, then replace
   each `todo!()` with real logic.
3. `cargo test -p <package-name>` until it's green.
4. `cargo clippy -p <package-name>` — read every warning, don't just silence it.
5. Answer `CHECKPOINT.md` in your own words (out loud or written — the point
   is active recall, not passing a quiz).
6. Only then open `solution/SOLUTION.md`. Compare reasoning, not just diffs.
7. Check the box for that lesson in `PROGRESS.md` and commit
   (see the commit convention below).

## Commit convention

One commit per completed lesson, scoped by package name:

```
feat(p1-02-01-move-semantics): complete exercise
```

Flip the corresponding checkbox in `PROGRESS.md` in the **same commit** so the
checklist and git history never drift apart. Tag the end of each phase:

```
git tag -a phase1-complete -m "Finished Phase 1: Fundamentals"
```

## Glossary

Unfamiliar term and no idea what it means? Check `docs/glossary.md` first —
it's a living document, add to it as new jargon shows up.
