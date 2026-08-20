# 07 — Git and repo workflow

## At a glance

After this lesson you can:

- Work out where you left off in under a minute after three weeks away.
- Make a clean commit per finished lesson that still means something months later.
- Say what CI checks on every push, and why some code is linted strictly and some isn't.

**Time:** ~25 minutes · **Prerequisites:** [06 — Tooling](../06-tooling-clippy-fmt-rust-analyzer/README.md)

---

## Why this matters

There's no Rust in this lesson. It's about the thing that determines whether you finish this course.

A multi-month learning path doesn't fail because the material is too hard. It fails because you get **lost**. You work for two weeks, then life takes a week, then you come back and spend ten minutes working out where you were — and those ten minutes are demoralising enough that you don't open it the next day.

So this lesson builds a small amount of structure that turns those ten minutes into twenty seconds.

There's a second thing too: you're doing this on stream and the repo is public. Your git history is part of what people see. A `git log` where every line means something is a CV in itself.

---

## The concept

### Four things that tell you where you are

| Where | What it tells you |
|---|---|
| `PROGRESS.md` | the master checklist, one checkbox per lesson, grouped by phase |
| `.course-progress.json` | the same, for the web UI — plus time, confidence and notes |
| The web UI's `/en/progress` page | all of that as charts, with the concept-mastery grid |
| `git log` | what you actually finished, and when |

The first is manual, the second and third are automatic, the fourth is impossible to fake.

That fourth one matters most. A checkbox can be ticked optimistically. A commit means code got written and its tests went green.

### The commit convention

**One commit per finished lesson**, with a message scoped by that lesson's package name:

```
feat(p1-02-01-move-semantics): complete exercise
```

And tick that lesson's box in `PROGRESS.md` **in the same commit**:

```sh
git add phase1-fundamentals/02-ownership-and-memory/01-move-semantics PROGRESS.md
git commit -m "feat(p1-02-01-move-semantics): complete exercise"
```

Why one commit? Because then the checklist and the history can never drift apart. If six months later you wonder "did I actually finish this?":

```sh
git log --grep=move-semantics --oneline
```

gives you a definitive answer.

The `feat(scope): description` format is a convention called **Conventional Commits**, and it's everywhere in real projects. The common prefixes:

| Prefix | For |
|---|---|
| `feat` | a finished lesson, a new capability |
| `fix` | repairing something that was broken |
| `docs` | documentation-only changes |
| `refactor` | changing code without changing behaviour |
| `chore` | maintenance, dependency updates |

Build the habit now. In Phase 4, when we cover CI/CD, you'll see tools use these exact prefixes to generate changelogs and decide version numbers automatically.

### A commit message worth writing

The first line is *what*. If you made a decision, the body is *why*:

```
feat(p1-04-02-slices): complete exercise

The obvious `&s[0..n]` panics on Persian text — n lands mid-character.
Used `char_indices` to find a real boundary instead. Worth remembering
for anything that touches user-supplied text.
```

That second paragraph is for future-you. Six months on you'll look at the code and not remember why you wrote it that way; the commit will remind you.

Simple rule: **if you learned something while writing it, write that down.**

### Tagging phase completions

Small, free, motivating milestones:

```sh
git tag -a phase1-complete -m "Finished Phase 1: Language foundations"
```

Months later, this gives you a complete timeline of the journey:

```sh
git log --oneline --decorate --tags
```

If you're streaming, it's also a good moment for your audience.

### What CI does on every push

`.github/workflows/ci.yml` runs:

```sh
cargo fmt --all -- --check          # everything must be formatted
cargo clippy ... -- -D warnings     # finished code only
cargo test --workspace --no-run     # everything must compile
cargo test ...                      # finished code only must be green
cargo run -q -p lesson-lint         # the lesson standard
```

That "finished code only" split is deliberate and worth understanding:

- **Formatting** applies to *everything*. Even an unfinished skeleton must be formatted.
- **Compiling** applies to everything. A skeleton must compile; `todo!()` doesn't break that.
- **Strict clippy and green tests** apply only to code that's *supposed* to be finished. A lesson skeleton is deliberately unfinished: `todo!()` bodies leave parameters unused and tests panic. Those are the lesson's guidance, not breakage.

Without that split, CI would be permanently red on an untouched clone — and permanently-red CI is ignored CI.

### `lesson-lint` and that shrinking list

CI's last step is a tool specific to this repo:

```sh
cargo run -q -p lesson-lint
```

It checks that every lesson follows [`docs/lesson-standard.md`](../../docs/lesson-standard.md): the required sections, English/Persian parity, links that resolve, and — most importantly — that **no lesson uses a concept before it's taught**.

Lessons not yet rebuilt are listed in `docs/lesson-lint-allow.txt`. Their findings are counted but don't fail CI. That file **only ever shrinks**; when it's empty, the rebuild is done.

You can run it yourself whenever you like:

```sh
cargo run -q -p lesson-lint                        # the whole course
cargo run -q -p lesson-lint -- phase1-fundamentals # one phase
```

### Working solo versus practising the PR workflow

You're the only contributor, so there's no requirement to use branches or pull requests. Commit straight to `main` if you want.

But "job-market ready" is an explicit goal of this journey, and real team work never looks like that. Suggestion: **open one PR per phase.** Make a branch, work the phase, open a PR, read your own diff like a stranger would, then merge.

It costs nearly nothing and builds the skill every team expects on day one: reading your own diff before anyone else does.

---

## Exercises

### Warm up

<details>
<summary>Why does the <code>PROGRESS.md</code> tick go in the same commit as the lesson's code?</summary>

So they can never drift apart. Committed separately, one will eventually be forgotten, and then neither is trustworthy. Together, `git log` is a definitive answer.

</details>

<details>
<summary>You commit an unfinished skeleton full of <code>todo!()</code>. Does CI go red?</summary>

No. `cargo test --workspace --no-run` only checks that it compiles, and `todo!()` compiles. Strict clippy and green tests run on finished code only. But formatting applies to everything — so run `cargo fmt`.

</details>

<details>
<summary>What is <code>docs/lesson-lint-allow.txt</code> for, and which way does it move?</summary>

Lessons not yet rebuilt to the new standard. Their findings are counted but don't fail CI. It only shrinks — a lesson isn't finished while its line is still in it.

</details>

<details>
<summary>A lesson's path changes. What happens to your completion tick in the web UI?</summary>

It survives, but attached to the old path — an orphaned key. It's deliberately not pruned (`docs/adr/0001-web-ui-progress-state.md`): a rename costs one re-tick rather than losing history.

</details>

### Build

Design yourself a "start of session" routine and try it. Write it down once, then run the same thing every time you come back:

```sh
git log --oneline -5                    # what did I do last?
cargo run -p course-ui                  # open the progress page
```

The goal is that "where was I?" requires no decision. Decisions cost energy; habits don't.

Then commit one of these Phase 0 lessons with the full convention — scoped message, `PROGRESS.md` tick in the same commit — and run `git show` to see what it looks like.

### Challenge (optional)

Tag the end of Phase 0:

```sh
git tag -a phase0-complete -m "Finished Phase 0: Setup and orientation"
git log --oneline --decorate --tags -10
```

Then open a branch for Phase 1 and work there:

```sh
git switch -c phase1
```

When Phase 1 is done, open a PR and **read your own diff end to end** before merging. You'll probably find something you want to change. That's the entire point.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Conventional Commits | the `feat(scope): description` format | every commit, here and at work |
| scope | the lesson's package name in brackets | makes `git log --grep` useful |
| tag | a named pointer at a commit | the end of every phase |
| CI | automated checks on every push | `.github/workflows/ci.yml` |
| `-D warnings` | treat every warning as an error | finished code only |
| `lesson-lint` | the lesson standard, enforced | CI's last step |
| the allowlist | lessons not yet rebuilt | only ever shrinks |

### What you now know

- Four things tell you where you are, and `git log` is the only one you can't fool yourself with.
- One commit per lesson, with the `PROGRESS.md` tick in it.
- The `feat(scope): description` format and why it matters.
- CI checks formatting and compilation on everything, but clippy and tests only on finished code.
- `lesson-lint` stops any lesson using a concept before it's taught.
- Branch-per-phase is optional and good practice.

### What comes back later

- **Phase 1 — Language foundations** — where Rust actually begins: [Phase 1](../../phase1-fundamentals/README.md)
- **CI/CD properly**, with Docker builds and deployment: [Phase 4 — Deployment and operations](../../phase4-backend-advanced/07-deployment-and-operations/README.md)
- **Deployment strategies** — blue-green, canary and the rest: [Phase 5 — DevOps](../../phase5-system-design-mastery/06-devops-and-cloud-fundamentals/03-deployment-strategies/README.md)

### Can you explain?

- What are the four things that tell you where you are, and which one can't be faked?
- Why does the `PROGRESS.md` tick go in the same commit as the code?
- In `feat(p1-02-01-move-semantics): complete exercise`, what does each piece do?
- Why does CI lint only some crates strictly?
- What does `lesson-lint` check that the compiler can't?
- After three weeks away, what's the first command you run?

---

## Going further

- [Conventional Commits](https://www.conventionalcommits.org/) — the whole specification, one page.
- [Pro Git, chapter 2](https://git-scm.com/book/en/v2/Git-Basics-Recording-Changes-to-the-Repository) — if git still doesn't feel solid, this chapter fixes that.
- [`docs/conventions.md`](../../docs/conventions.md) — this repo's structural rules.
- [`docs/lesson-standard.md`](../../docs/lesson-standard.md) — the format `lesson-lint` enforces.

---

**Phase 0 is done.** You have Rust installed, you've written a program, you can read a compiler error, you know cargo and the tooling, and you have a working routine.

From here the language itself begins: [Phase 1 — Language foundations](../../phase1-fundamentals/README.md).
