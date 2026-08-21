---
name: senpai-rust-course-author
description: Author and review SenPai Rust lessons to docs/lesson-standard.md — Persian-first pedagogy, an English mirror, accurate Rust semantics, real compiler diagnostics, a graded exercise ladder, and machine-verified concept ordering.
---

# SenPai Rust Course Author

Use when adding, rewriting, or reviewing curriculum content in this repo.

**Read [`docs/lesson-standard.md`](../../../docs/lesson-standard.md) first.** It
is the specification; this file is the craft guidance that the specification
cannot check mechanically. `cargo run -p lesson-lint` enforces the former. Only
you can enforce the latter.

## The three rules that matter most

1. **Persian is authored, not translated.** Write `README.fa.md` from the lesson
   plan as Persian teaching prose. Then write `README.md` from the *same plan*,
   not from the Persian text. Reconcile afterwards: same headings, same code,
   same exercises.

2. **Never show a code block without its result.** Every snippet is followed by
   its real output or its real compiler error. If you have not run it, do not
   claim what it prints.

3. **An exercise must be solvable from its specification alone.** If a test
   asserts an exact string, the doc comment states that exact string. The
   previous curriculum failed this and it is the single most common way a
   lesson goes wrong.

## Teaching order inside a lesson

Outcome → the problem it solves → the Python/Django bridge → the exact Rust
rule → the compiler error you will hit → run it → the exercise ladder → recap
with forward links.

Show behaviour before naming it. A reader who has watched the thing happen has
somewhere to attach the term; a reader given the term first has only a word.

## Compiler errors are the lesson, not an appendix

`## Errors you will meet` is where most of the learning lives. For every error:
paste the verbatim `rustc` output including its code, explain what the compiler
is actually objecting to, then give the fix and why it *is* the fix.

Put the broken code in `examples/`, marked
`//! DELIBERATELY BROKEN — expected: E0382`, so the reader produces the error
themselves rather than reading about it.

Run the code and copy the real diagnostic. Never write one from memory —
`rustc`'s wording, spans and help text change between releases, and an
invented diagnostic is worse than none.

An error that stops the *parser* cannot live in `examples/`: `cargo fmt --check`
covers every workspace target and rustfmt cannot format what it cannot parse.
Capture those from a scratch file named after the mistake — the filename shows
up in the diagnostic, so `statement-as-value.rs` reads far better than `a.rs` —
and say in the lesson that there is no example file for it.

## The exercise ladder

Five rungs, always: warm up (predict, zero typing) → repair (fix a broken
program) → implement (fully specified, tested) → build (small, open) →
challenge (optional).

A `todo!()` message says *what*, never *how*:

```rust
// Wrong — the answer is the prompt:
todo!("s.parse::<u32>().map_err(|e| e.to_string())")

// Right:
todo!("parse `s` as a u32; on failure return the parse error's message as the Err")
```

## Continuity

`### What comes back later` is mandatory and load-bearing. Any concept a lesson
touches without finishing must link to the lesson that finishes it. This is
what turns a pile of lessons into a course.

Register every concept in [`docs/concept-map.toml`](../../../docs/concept-map.toml)
with a truthful `introduced_in`. If a lesson needs something taught later,
choose deliberately: move the concept earlier, teach the minimum inline and add
this lesson to its `deepened_in`, or avoid it. `lesson-lint` will not let you
choose "silently".

## Persian voice

- Translate meaning and intent, not English word order. Read the whole lesson
  and its code before choosing the Persian sentence.
- Address Matin directly in a warm, precise, conversational voice. Short active
  sentences, natural Persian verbs, no stiff passive prose.
- Keep technical English only when it is an identifier, an established Rust
  term, or genuinely clearer. Introduce it once as
  `معادل فارسی (English term)`, then stay with that form — never rotate among
  several translations later.
- Persian ی and ک, Persian punctuation and digits in prose, ZWNJ in forms such
  as `می‌شود`, `به‌جای`, `همه‌ی`.
- A familiar Iranian example must clarify the exact rule, and you must say
  where it stops being exact. Do not invent colourful idioms for liveliness.
- Preserve every factual qualification from the plan. Do not soften or
  strengthen a technical claim to make a sentence flow.
- After drafting, read the Persian paragraph on its own. If it sounds
  translated, rewrite it as a Persian teacher would say it aloud.

### Bidi hazard — read this before typing any inline code

Persian is RTL and code is LTR. When a mixed line is *rendered*, brackets, `.`,
`&` and quotes at the edges of a Latin run move to the other end and mirror.
That is correct display. It becomes corruption if the rendered form is what
gets saved.

It happened at scale in this repository: `` `&str` `` was on disk as
`` `str&` ``, `` `.bind()` `` as `` `()bind.` ``, `` `char_count(s)` `` as
`` `(char_count(s` ``. Roughly 550 spans across the Persian curriculum.

So: type inline code in logical order, and never copy code out of a rendered
RTL view back into a file. `cargo run -p lesson-lint` catches the known shapes,
and `--fix-rtl-code` repairs the ones whose English original is unambiguous —
but the fix is not always derivable, so do not rely on it.

Code and paths stay LTR. Be especially careful in lessons about bytes,
`.len()`, slicing and character counts, where Persian text is itself the
teaching material.

## Figures

At least one valid `senpai-visual` fence wherever memory, ownership, lifetimes,
async or distributed behaviour is involved — those are the places a sentence
genuinely cannot do the work. Valid `kind` values are listed in
`web-ui/src/visual.rs`. Do not add a decorative figure to a lesson that does
not need one.

## Before marking a batch complete

```sh
cargo run -p lesson-lint -- <phase-path>
cargo fmt --all -- --check
cargo test --manifest-path <lesson>/solution/Cargo.toml
cargo test --workspace --no-run
cargo clippy -p course-ui -p lesson-lint --all-targets -- -D warnings
```

Then delete the migrated lessons from `docs/lesson-lint-allow.txt`. That file
only ever shrinks; a lesson is not done while its line is still in it.
