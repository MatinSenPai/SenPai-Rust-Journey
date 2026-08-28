# The lesson standard

Every lesson in this repository is built to this specification. It is not a
style preference — `cargo run -p lesson-lint` enforces most of it, and CI fails
when a lesson drifts.

Persian (`README.fa.md`) is the **canonical** teaching text: it is authored as
Persian pedagogy, not translated from English. `README.md` is written from the
same lesson plan afterwards. Neither is a back-translation of the other, and
both must teach the same thing in the same order.

The reasoning behind this standard is in
[`plans/005-curriculum-rebuild.md`](../plans/005-curriculum-rebuild.md).

## 1. Directory shape

```
NN-slug/
├── Cargo.toml
├── README.fa.md          # canonical teaching text
├── README.md             # English mirror
├── examples/
│   ├── 01-<name>.rs      # runnable, referenced from the lesson body
│   └── 02-<name>.rs      # may be deliberately broken — see §5
├── src/
│   └── lib.rs            # the exercise ladder + its tests
└── solution/
    ├── Cargo.toml        # own `[workspace]` table — see docs/conventions.md
    ├── src/lib.rs
    ├── SOLUTION.fa.md
    └── SOLUTION.md
```

There is **no `CHECKPOINT.md`**. It was removed in full; recall now lives in
§7's self-assessment list, which asks you to explain things rather than to
prove you were paying attention.

`examples/` is mandatory for any lesson with code. A lesson must contain
something you *run and observe* before it asks you to write anything.

Pure-reading lessons (Phase 0's first lessons, most of Phase 5) have no
`Cargo.toml`, no `src/`, no `solution/`. They still follow §2–§7, minus the
sections that require code.

## 2. Required sections, in this order

Both language files carry the same eight headings in the same order. The linter
checks presence, order, and cross-language parity.

| # | Persian heading | English heading |
|---|---|---|
| 1 | `## در یک نگاه` | `## At a glance` |
| 2 | `## چرا اهمیت دارد` | `## Why this matters` |
| 3 | `## مفهوم` | `## The concept` |
| 4 | `## دست‌به‌کد` | `## Hands on` |
| 5 | `## خطاهایی که خواهی دید` | `## Errors you will meet` |
| 6 | `## تمرین` | `## Exercises` |
| 7 | `## جمع‌بندی` | `## Wrapping up` |
| 8 | `## بیشتر` | `## Going further` |

A pure-reading lesson may omit §4 and §5. Nothing else is optional.

## 3. §1 — At a glance

Three bullets, each starting with a verb, each describing something the reader
can *do* afterwards. Not topics — capabilities.

Then a time estimate and a prerequisite list linking **backwards** to the
lessons this one depends on:

```markdown
## At a glance

After this lesson you can:

- Explain why `let x = 5` refuses a second assignment, without saying "because
  Rust is strict".
- Choose between `mut` and shadowing for a given piece of real code.
- Read and fix `E0384` on your own.

**Time:** ~35 minutes · **Prerequisites:**
[0.6 — Hello, Rust](../../../phase0-setup/06-hello-rust/README.md)
```

The prerequisite links are load-bearing: `lesson-lint` resolves them and the
web UI uses them.

## 4. §3 — The concept

This is the teaching body and where most of the lesson's words go. Rules:

1. **One idea per `###` subsection**, in escalating order. If a subsection
   introduces two ideas, split it.
2. **Every code block is at most 15 lines** and is immediately followed by
   either its actual output or its actual compiler error. Never a code block
   whose result the reader has to imagine.
3. **Show, then name.** Demonstrate the behaviour first, give it its
   terminology second. Not the reverse.
4. **Introduce each term once** as `معادل فارسی (English term)` in the Persian
   text, then use the established glossary form. Add it to
   [`docs/glossary.md`](glossary.md) in the same commit.
5. **A figure wherever memory, ownership, or control flow is involved.** Use a
   `senpai-visual` fence — the renderer is in `web-ui/src/visual.rs` and the
   supported `kind` values are listed there.
6. **The Python/Django bridge is used where it clarifies, and its limits are
   stated.** An analogy that is not told where it breaks is a future bug in the
   reader's mental model.

### Output blocks

Show output as a fenced block tagged `text`, exactly as the terminal prints it:

````markdown
```rust
let x = 5;
println!("{x}");
```

```text
5
```
````

## 5. §5 — Errors you will meet

**This is the highest-value section in the standard and the one the old course
lacked entirely.** The compiler is Rust's primary teacher; a lesson that never
shows you a diagnostic has not taught you to work with it.

Every lesson lists the errors a learner will genuinely hit on this material,
each with:

- the **verbatim** `rustc` output, including the error code
- what the compiler is actually complaining about, in plain language
- the fix, and *why* it is the fix

````markdown
### `E0384` — cannot assign twice to immutable variable

```text
error[E0384]: cannot assign twice to immutable variable `x`
 --> src/main.rs:3:5
  |
2 |     let x = 5;
  |         - first assignment to `x`
3 |     x = 6;
  |     ^^^^^ cannot assign twice to immutable variable
  |
help: consider making this binding mutable
  |
2 |     let mut x = 5;
  |         +++
```

`rustc` tracked where `x` was first bound and is telling you that binding is
final. …
````

Deliberately-broken code lives in `examples/` so the reader can produce the
error themselves. Mark such a file with a header comment:

```rust
//! DELIBERATELY BROKEN — expected: E0384
//! Run `cargo run --example 02-reassign` and read the error.
```

`lesson-lint` reads that marker: the file is excluded from "must compile" and
instead asserted to fail with exactly that error code. A broken example without
the marker is a lint failure.

**One exception: an error that stops the parser cannot be an example.**
`cargo fmt --check` runs over every target in the workspace, and rustfmt cannot
format a file it cannot parse — so a committed example containing, say,
`let x = (let y = 6);` breaks the format gate for the whole repository. Type
errors, borrow errors and lint denials all parse fine and are safe. For the
handful that do not, capture the diagnostic from a scratch file named after the
mistake, show it in §5, and say in one line that there is no example file for
it and why.

### Index pages are checked too

Phase and module `README`s are not lessons, so the per-lesson link check never
sees them. `lesson-lint` checks them separately, because a phase restructure
once left every Phase 1 index pointing at directory names that no longer
existed with the whole build green.

Two outcomes, deliberately separated. A Persian page linking to a
`README.fa.md` whose English companion exists is reported as
`translation-pending` and does **not** fail the run — it is a progress counter
for the phases still to be translated. Everything else dangling blocks.

## 6. §6 — The exercise ladder

Five rungs, always in this order, always with these headings. The point is that
you are never asked to jump from prose to a blank function body.

| Rung | Persian | English | What it asks |
|---|---|---|---|
| 1 | `### گرم‌کردن` | `### Warm up` | Predict the output / will this compile? **Zero typing.** Answers in `<details>`. |
| 2 | `### تعمیر` | `### Repair` | Fix broken code producing an error from §5. |
| 3 | `### پیاده‌سازی` | `### Implement` | Fully specified functions with tests. |
| 4 | `### بساز` | `### Build` | One small open-ended piece. |
| 5 | `### چالش` | `### Challenge` | Optional stretch. May reach forward, and says so. |

### The specification rule

> **An implementation exercise must be passable without opening the test file.**

If a test asserts `"4 chars, starts with 'r'"`, the doc comment states that
exact format. Anything a test checks that the spec does not state is a defect,
not a challenge. `lesson-lint` cannot check this automatically — it is on the
author, and it is the single most common way a lesson goes wrong.

### The `todo!()` rule

> **A `todo!()` message describes *what*, never *how*.**

```rust
// Wrong — this is the answer, not a prompt:
todo!("s.parse::<u32>().map_err(|e| e.to_string())")

// Right — this is the goal:
todo!("parse `s` as a u32; on failure return the parse error's message as the Err")
```

`lesson-lint` flags `todo!()` messages containing `::<`, `.map_err(`, `|` or
other give-away syntax.

## 7. §7 — Wrapping up

Three fixed subsections.

**A term table** — every term this lesson introduced:

```markdown
| Term | What it means | Where you'll use it |
|---|---|---|
| shadowing | re-binding a name with a new `let` | pipelines of transformations |
```

**`### الان می‌دانی` / `### What you now know`** — the concrete list.

**`### بعداً کامل‌تر می‌بینی` / `### What comes back later`** — every concept
this lesson touched but did not finish, each naming the lesson that finishes
it, as a link. This is what makes the course continuous instead of episodic,
and it is mandatory: if a lesson mentions `Result` in passing, it links to the
lesson that teaches `Result`.

**`### می‌توانی توضیح بدهی؟` / `### Can you explain?`** — the replacement for
`CHECKPOINT.md`. A short list of things to say out loud, in your own words. No
answers are demanded, no error output must be pasted, nothing is graded. It is
a self-check, and it doubles as the recap script for a stream.

## 8. Concept ordering

Nothing may be used before it is taught. `docs/concept-map.yaml` records where
each concept is introduced; `lesson-lint` fails when a lesson's prose or code
uses a concept whose `introduced_in` is later in the course.

If a lesson genuinely needs a forward concept, the fix is one of:

1. move the concept earlier (usually correct — this is why `Vec` and `String`
   now live in Phase 1);
2. teach the minimum needed inline and register the lesson under that concept's
   `deepened_in`;
3. avoid it.

Silently using it is not an option, and CI will say so.

## 9. Writing the pair

1. Plan the lesson once: outcomes, concept order, the errors, the ladder.
2. Write `README.fa.md` from that plan, as Persian teaching prose.
3. Write `README.md` from the *same plan* — not from the Persian text.
4. Reconcile: same headings, same code, same examples, same exercises. If one
   language explains something better, backport the improvement to the other.

Persian voice rules (Persian ی/ک, ZWNJ, term introduction, where analogies
stop) are in
[`.agents/skills/senpai-rust-course-author/SKILL.md`](../.agents/skills/senpai-rust-course-author/SKILL.md).

## 10. Verification

```sh
cargo run -p lesson-lint              # the whole repo
cargo run -p lesson-lint -- phase1-fundamentals   # one subtree
cargo fmt --all -- --check
cargo test --workspace --no-run
```

`lesson-lint` checks:

1. section presence and order, both languages
2. FA/EN structural parity — same heading tree, same number of code blocks
3. every Rust fence in a README compiles (unless marked broken, in which case
   it must fail with the stated error code)
4. every `examples/*.rs` builds, and runs unless marked broken
5. concept-map ordering
6. every internal markdown link resolves
7. no `CHECKPOINT*.md` exists
8. `todo!()` messages describe *what*, not *how*
9. every lesson with code has at least one `examples/` file
10. §1 lists prerequisites and they resolve
