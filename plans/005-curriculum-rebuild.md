# Plan 005: Rebuild the curriculum into a teaching course

> Planned at `d51d9e9` on 2026-08-20, after a full read of the repository.
> This plan supersedes the pedagogy assumptions in plans 001–004. Those plans
> delivered a Persian *translation layer* over the existing lessons; this one
> rebuilds what is being translated.

## 1. Why this plan exists

Matin is at `phase1-fundamentals/01-variables-types-control-flow/03-...` —
eight lessons into roughly 130. Almost nothing is sunk cost, so the curriculum
can be restructured now at close to zero waste. It cannot be restructured
cheaply in three months.

### 1.1 What the audit found

**The teaching is thin exactly where the learner is weakest.**

| Phase | Avg. lesson README |
|---|---|
| Phase 0–1 | 40–70 lines |
| Phase 2 | ~95 lines |
| Phase 3+ | 150+ lines, noticeably better taught |

`phase1-fundamentals/01-variables-types-control-flow/01-variables-mutability-shadowing/README.md`
is **64 lines** and covers `let`, `mut`, shadowing *and* `const`. It states
rules. It never shows a program running, never shows a compiler error, never
shows output, and offers no worked example between "here is the rule" and
"implement two `todo!()`s".

**Concept ordering is broken — concepts are used before they are taught.**

- `phase1-fundamentals/02-ownership-and-memory/01-move-semantics/src/lib.rs`
  requires `Vec<String>`. `Vec` is taught in
  `phase2-intermediate/01-collections/01-vec-and-hashmap`.
- Lesson 1.1.1 uses `.parse()` (returns `Result`) and `format!`. `Result` is
  taught in 1.6.2.
- `Copy` and `Clone` are taught in 1.2.2 as if they were language keywords;
  they are traits, and traits arrive in 2.3.

This is the concrete cause of "the lessons have no coherence."

**Exercise specs are simultaneously unknowable and spoiled.**

- `phase1-fundamentals/03-borrowing-and-references/02-borrow-checker-rules/src/lib.rs`
  — `describe_and_grow`'s test asserts exactly `"4 chars, starts with 'r'"`.
  The doc comment never states that format. The exercise is unpassable
  without reading the test file.
- `phase1-fundamentals/06-option-result-error-basics/02-result-and-question-mark/src/lib.rs`
  — the `todo!()` message is literally
  `"s.parse::<u32>().map_err(|e| e.to_string())"`. The answer is the prompt.

So the learner is asked to guess an unstated spec in one lesson and to copy a
given answer in the next. Neither builds understanding, and both make the
exercise — rather than the concept — feel like the point of the lesson.

**182 `CHECKPOINT.md` files quiz rather than teach.** Sample:
"Paste the key line of the error." This is the "school teacher trying to catch
you out" dynamic. Removed entirely by this plan.

**Roughly 35–40% of what a professional Rust backend engineer needs is
missing or reduced to a passing mention.** Verified by grep across
`phase0`–`phase4`:

| Topic | Files mentioning it |
|---|---|
| `PhantomData` | 0 |
| orphan rule | 0 |
| object safety | 0 |
| `Weak<T>` / reference cycles | 0 |
| `JoinSet` | 0 |
| `select!` / cancellation safety | 0 |
| property testing (`proptest`) | 0 |
| associated types | 1 |
| supertraits | 2 |
| blanket impls | 2 |

`From`/`Into` appear incidentally in 22 files but have **no lesson** — despite
being the machinery that makes `?` work, which the course teaches in 1.6.2
without explaining.

**Web UI.** `web-ui/src/style.rs:7` applies
`ui-monospace, SFMono-Regular, Consolas, monospace` to
`code, pre, kbd, bdi, [dir=ltr]`. Inline English terms inside Persian prose are
wrapped in `bdi`/`[dir=ltr]`, so the typeface visibly switches mid-sentence.
`style.rs` is a single minified blob with four appended override layers;
`.hero h1` is redefined three times (lines 11, 18, 21). Progress is one boolean
per lesson — no per-exercise state, no time, no dashboard beyond `3/6` counts.

**Phase 5 has 38 lessons and zero Persian translation.**

### 1.2 Decisions taken

| Decision | Choice |
|---|---|
| Canonical language | **Persian-first.** `README.fa.md` is authored as native Persian pedagogy; `README.md` is written from the same lesson plan, not back-translated. |
| Sequencing | **Just-in-time.** Rebuild shared infrastructure + Phase 0 + Phase 1 now; rebuild each later phase before it is reached. |
| Web UI | **Full redesign** — one typeface system, structured CSS, light/dark, and a real progress system. |

Streaming resumes after WP-2 (8 lessons), not after the whole rebuild.

## 2. The new lesson standard

### 2.1 Directory shape

```
NN-slug/
├── Cargo.toml
├── README.fa.md          # canonical teaching text
├── README.md             # English mirror
├── examples/
│   ├── 01-<demo>.rs      # runnable, referenced from the lesson body
│   └── 02-<broken>.rs    # deliberately broken; run it, read the error
├── src/
│   └── lib.rs            # graded exercise ladder + tests
└── solution/
    ├── Cargo.toml
    ├── src/lib.rs
    ├── SOLUTION.fa.md
    └── SOLUTION.md
```

No `CHECKPOINT.md`. All 182 are deleted in WP-0.

`examples/` is new and load-bearing: a lesson must contain something you *run*
before you are asked to write anything.

### 2.2 Required section order (linter-enforced, both languages)

1. **`## در یک نگاه`** — three "after this you can…" bullets, a time estimate,
   and prerequisite links *backwards* to the lessons this depends on.
2. **`## چرا اهمیت دارد`** — the concrete problem, framed against the
   Python/Django equivalent the learner already has.
3. **`## مفهوم`** — the teaching body.
   - One idea per subsection, escalating.
   - Every code block ≤ 15 lines and **followed by its actual output or its
     actual compiler error**.
   - A `senpai-visual` figure wherever memory, ownership or flow is involved
     (the renderer already supports this — `web-ui/src/visual.rs`).
   - Each new term introduced once as `معادل فارسی (English term)`, then the
     glossary form thereafter.
4. **`## دست‌به‌کد`** — walk through `examples/`: run this, here is the output,
   now change X and predict what happens.
5. **`## خطاهایی که خواهی دید`** — verbatim `rustc` diagnostics with error
   codes (`E0382`, `E0502`, …), what each one means, and the fix. **The current
   course has no equivalent section anywhere.** This is the single biggest
   quality lever in the rebuild: the compiler is Rust's primary teacher, and
   the course currently never teaches you to read it.
6. **`## تمرین`** — a fixed ladder, always in this order:
   - **گرم‌کردن** — predict the output / will this compile? Answers in
     `<details>`. Zero typing.
   - **تعمیر** — fix broken code producing an error you just learned to read.
   - **پیاده‌سازی** — fully specified functions with tests. **The spec must be
     complete enough to pass without opening the test file.** `todo!()`
     messages state *what*, never *how*.
   - **بساز** — one small open-ended piece.
   - **چالش (اختیاری)** — stretch; may reach forward, and says so.
7. **`## جمع‌بندی`** — a term table (term / meaning / where it is used), then:
   - `### الان می‌دانی` — the concrete list of what is now yours.
   - `### بعداً کامل‌تر می‌بینی` — **explicit forward links.** Every concept
     touched but not finished here names the lesson that finishes it. This is
     the mechanism that makes the course feel continuous rather than
     episodic.
   - `### می‌توانی توضیح بدهی؟` — a self-assessment list, replacing
     `CHECKPOINT.md`. No answers demanded, no gotchas. It doubles as the recap
     script for a stream.
8. **`## بیشتر`** — TRPL / std docs / Rustonomicon / relevant blog links.

### 2.3 Continuity, mechanically enforced

`docs/concept-map.yaml` records every concept in the course:

```yaml
- id: vec
  fa: وکتور
  en: Vec<T>
  introduced_in: phase1-fundamentals/01-.../06-vec-and-string-basics
  deepened_in:
    - phase2-intermediate/01-collections/01-vec-depth
  mastered_in: phase2-intermediate/01-collections/04-choosing-a-collection
```

`scripts/lint-lessons` fails CI when a lesson's code or prose uses a concept
before its `introduced_in`. This permanently kills the `Vec`-before-`Vec` class
of defect, and it is what feeds the concept-mastery grid in the web UI.

## 3. Restructured syllabus

~130 lessons → ~180. New or substantially rewritten lessons marked **NEW**.

### Phase 0 — راه‌اندازی (6 → 8)

1. What is a compiled language *(expand: what a binary actually is, why "no GC"
   changes how you think, zero-cost abstractions)*
2. Installing Rust & toolchains *(expand: channels, components, targets,
   `rust-toolchain.toml`, Windows specifics)*
3. Cargo basics *(expand: manifest anatomy, semver, lockfile, profiles,
   `cargo add`/`tree`/`why`)*
4. Tooling — clippy, fmt, rust-analyzer *(expand: inlay hints, `cargo watch`,
   `nextest`, `expand`)*
5. **NEW — How to read a Rust compiler error.** Anatomy of a diagnostic, error
   codes, `rustc --explain`, `cargo check` vs `build`, reading a
   borrow-checker error's spans/labels/notes/helps. The highest-leverage
   lesson in Phase 0 and it does not currently exist.
6. Hello, Rust *(expand: `main`, `println!` vs `format!`, expression preview)*
7. Git & repo workflow
8. **NEW — How this course works.** The learning loop, the lesson anatomy, the
   exercise ladder, the concept map, the web UI, the stream workflow.

### Phase 1 — پایه‌های زبان (23 → 33, reordered)

**1.1 مبانی (6)** — variables/`mut`/shadowing/`const` vs `static`; scalar types
+ overflow semantics + `checked_`/`wrapping_`/`saturating_` + float equality;
compound types & destructuring; **NEW — functions and the expression language**
(statements vs expressions, blocks return values, `if` as an expression,
`loop`/`break value`, labeled breaks, `()`, `!`); control flow;
**NEW — `Vec` and `String`, just enough** *(moved forward from Phase 2 —
everything after lesson 1.2 needs them; full depth stays in Phase 2 and the
lesson says so)*.

**1.2 حافظه و مالکیت (5)** — **NEW — stack vs heap, concretely** (what a
`String` *is*: ptr/len/cap, with a figure); move semantics; `Copy`/`Clone`;
**NEW — ownership across function boundaries**; `Drop` and RAII.

**1.3 قرض‌گرفتن (4)** — shared & mutable refs; borrow-checker rules;
**NEW — borrow scopes and NLL** (*"why does this compile when the book says it
shouldn't?"*); slices.

**1.4 متن (4)** — `String` vs `&str` in depth; **NEW — UTF-8, bytes, chars and
graphemes, using Persian text** (`"سلام".len() == 8`, `.chars().count() == 4`,
why `&s[0..1]` panics, `char_indices`, grapheme clusters); building &
transforming strings; slicing safely.

**1.5 انواعِ خودت (5)** — structs, `impl`, methods, associated functions;
**NEW — tuple/unit structs and the newtype pattern**; enums as data;
`match` in depth (exhaustiveness, guards, `@`, ranges, or-patterns);
`if let`/`while let`/**`let else`**.

**1.6 نبودن و شکست (5)** — `Option`; **NEW — `Option` combinators**;
`Result` and `?`; `panic!` vs `Result`; **NEW — `From` for error conversion and
how `?` actually works**. The keystone lesson the course currently omits.

**1.7 سرهم‌بندی (2)** — **NEW — a guided mini-project** built across the whole
lesson, line by line, never from a blank page; **NEW — Phase 1 review**: the
concept ledger plus a diagnostic exercise set. Not a quiz.

→ Side-quest 1 (reworked to use only Phase-1 concepts).

### Phase 2 — Rust اصطلاحی (29 → 42)

**2.1 Collections (4)** — `Vec` depth (capacity, `retain`, `drain`, `dedup`,
`binary_search`); `HashMap` depth (the `entry` API, hashers, `&str` lookup in a
`HashMap<String, _>`); `BTreeMap`/`HashSet`/`VecDeque`/`BinaryHeap`;
**NEW — choosing a collection** (complexity table, when each wins).

**2.2 Iterators & closures (5)** — closures, `Fn`/`FnMut`/`FnOnce`, `move`;
adapters; **NEW — consuming and collecting** (into `Vec`/`String`/`HashMap`/
`Result<Vec<_>, E>` — the last is magic and currently unexplained);
**NEW — implementing `Iterator` and `IntoIterator` for your own type**;
**NEW — laziness and iterator performance**.

**2.3 Traits (7)** — defining & implementing; generics, bounds, `where`;
**NEW — `From`/`Into`/`TryFrom`/`TryInto`**; **NEW — the standard derives and
implementing them by hand** (`Debug`, `Display`, `Default`, `PartialEq`/`Eq`,
`PartialOrd`/`Ord`, `Hash`, `Clone`); **NEW — associated types vs generic
parameters**; **NEW — supertraits, blanket impls, the orphan rule and the
newtype escape hatch**; static vs dynamic dispatch **+ object-safety rules**.

**2.4 Borrowing at the type level (4)** — lifetime basics & elision;
**NEW — lifetimes in structs and methods**; **NEW — `Deref`/`AsRef`/`Borrow`/
`ToOwned`**; **NEW — `Cow<'_, str>`** (backend-critical, currently absent).

**2.5 Errors (4)** — custom error types & `std::error::Error`;
**NEW — source chains and `Box<dyn Error>`**; `thiserror` vs `anyhow` and the
library/binary boundary rule; **NEW — designing an error taxonomy for a
service**.

**2.6 Smart pointers & shared state (5)** — `Box`; **NEW — recursive types and
boxed trait objects**; `Rc`/`Arc`; **NEW — `Weak` and reference cycles**;
`RefCell`/`Cell` and the runtime-panic trade.

**2.7 Structure & testing (5)** — modules, visibility, re-exports, workspaces;
unit/integration/doc tests; **NEW — test doubles in Rust** (traits and
`#[cfg(test)]` impls — why you rarely need a mocking framework);
**NEW — property testing with `proptest` and snapshot testing with `insta`**;
**NEW — benchmarking with `criterion`** as its own lesson.

**2.8 Concurrency (6)** — threads/`Mutex`/`Arc`; **NEW — `RwLock`, `Semaphore`,
`OnceLock`/`LazyLock`, atomics**; channels; **NEW — `Send`/`Sync`: what they
actually are and why your type isn't `Send`**; futures & runtimes (what
`async fn` desugars to, the state machine, `Pin` explained *usefully*);
tokio basics.

**2.9 Async in practice (4, mostly NEW)** — `spawn`, `JoinSet`, structured
concurrency; **`select!` and cancellation safety**; **streams**
(`futures::Stream`, `tokio-stream`); **async traits** (`async-trait` vs native
AFIT) and `spawn_blocking`.

**2.10 Toolbox (4)** — pattern matching depth; `macro_rules!`; cargo features &
conditional compilation; **`unsafe` for real** (raw pointers, UB, upholding
invariants, when a backend engineer actually touches it) — the current lesson
is an "overview".

→ Side-quest 2.

### Phase 3 — بک‌اند (24 → 33)

**3.1 Networking (3)** — TCP echo server; hand-rolled HTTP parser;
**NEW — HTTP semantics you must know** (methods, status codes, headers,
content negotiation, keep-alive, chunked).

**3.2 axum & REST (5)** — routing/handlers/extractors; **NEW — writing your own
extractor (`FromRequestParts`)**; CRUD in memory; **NEW — `tower::Service` /
`Layer`: write middleware by hand**; CORS & frontend integration.

**3.3 Data in and out (4)** — serde depth (`rename_all`, `flatten`,
`skip_serializing_if`, custom `Serialize`, `untagged`); validation;
**NEW — API contracts & OpenAPI (`utoipa`)**; **NEW — API versioning and
evolution**.

**3.4 Configuration & app structure (3, NEW group)** — 12-factor config and
secrets in Rust (layered config, `SecretString`); application state and
dependency wiring (`AppState`, traits vs concrete, when DI hurts in Rust);
graceful shutdown, health and readiness.

**3.5 Postgres & sqlx (5)** — connecting & pooling (pool-sizing math);
migrations; Postgres-backed CRUD; transactions & isolation;
**NEW — the repository pattern and testable data access**.

**3.6 Query performance (3)** — indexes, `EXPLAIN ANALYZE`, N+1; pagination
(offset vs keyset, done properly); **NEW — schema design for a real service**.

**3.7 Auth & security (5)** — Argon2 password hashing; **NEW — sessions vs JWT:
the real trade-off**; JWT + tower middleware; **NEW — refresh-token rotation
and revocation**; **NEW — modelling RBAC and permissions**. OWASP threaded
through rather than bolted on.

**3.8 Errors, tracing & testing (5)** — consistent error envelopes;
**request tracing and correlation IDs** *(moved forward from Phase 4 — you need
it to debug Phase 3)*; integration tests with testcontainers;
**NEW — test data factories and fixtures**; **NEW — WebSockets and SSE in
axum**.

→ Side-quest 3.

### Phase 4 — بک‌اند پیشرفته (21 → 30)

**4.1 Caching (4)** — Redis from Rust; strategies & invalidation;
**NEW — stampede protection, single-flight, TTL jitter**; **NEW — in-process
caching (`moka`) and the two-tier pattern**.

**4.2 Rate limiting & backpressure (3)** — algorithms; distributed limiting via
Redis + Lua; backpressure, load shedding, queue depth.

**4.3 Jobs & queues (5)** — job queue design; broker concepts
(RabbitMQ/Kafka/NATS); **NEW — at-least-once delivery, idempotency and
dedupe**; **NEW — the transactional outbox pattern**; retries, DLQ, poison
messages.

**4.4 Service-to-service (3)** — gRPC with tonic; GraphQL with async-graphql;
**NEW — choosing between REST / gRPC / GraphQL / events**.

**4.5 Observability (4)** — structured logging with `tracing`; metrics
(RED/USE) & Prometheus; **NEW — distributed tracing with OpenTelemetry,
end to end**; **NEW — SLOs, alerting and on-call reality**.

**4.6 Distributed data patterns (3, NEW group)** — sagas and compensating
transactions; event sourcing & CQRS (a lite, honest treatment that says when
*not* to); multi-tenancy models.

**4.7 System design fundamentals (2)** — CAP, scaling, LB, idempotency,
locking; **NEW — capacity estimation and back-of-envelope math**.

**4.8 Deployment & operations (4)** — Docker for Rust (multi-stage, distroless);
CI/CD; **NEW — secrets, config and environments in production**;
**NEW — zero-downtime deploys, migrations and rollback**.

**4.9 Performance (4)** — profiling & flamegraphs; **NEW — allocation awareness
and hot-path Rust**; **NEW — async performance traps** (blocking the runtime,
unbounded spawn); load testing and finding your real limits.

→ Side-quest 4 → Capstone (TaskForge).

### Phase 5 — System design mastery (38 → 41)

Keep the ByteByteGo-derived syllabus, and:

- **Translate all 38 lessons to Persian** (currently zero are translated).
- **Attach a Rust artifact to every reading lesson** so none is pure prose — a
  small simulation, a benchmark, a generated diagram, or a written design doc.
- **NEW group `08-interview-and-communication` (3)** — writing a design doc and
  an ADR; whiteboarding a system out loud; mock design-interview prompts with
  rubrics.

## 4. Web UI rebuild

### 4.1 Typography

- **Vazirmatn for all prose, Latin included** — it ships Latin glyphs, so a
  Persian sentence containing English terms stays in one typeface.
- **Remove `bdi` and `[dir=ltr]` from the monospace selector**
  (`web-ui/src/style.rs:7`). Monospace applies to `code`, `pre`, `kbd` only.
- **Self-host one code face** (JetBrains Mono or Fira Code) so code has stable
  metrics against Persian line-height, instead of falling through to whatever
  Consolas the OS provides.
- Tabular numerals for all progress figures.

### 4.2 Stylesheet

`web-ui/src/style.rs` becomes a layered stylesheet — `tokens / base / layout /
components / pages` — served the same way, with no minified blob and no
appended override layers. `.hero h1` is defined once. Light theme added with a
persisted toggle. The existing `prefers-reduced-motion` and `forced-colors`
handling is good and is preserved.

### 4.3 Progress system (schema v2, migrated from v1)

Per lesson: `status` (untouched / in-progress / done), `started_at`,
`completed_at`, `time_spent_seconds`, `exercises_done[]`, `note`,
`confidence` (1–3, self-rated).

`/fa/progress` dashboard:

- overall %, per-phase bars, current streak, hours logged
- a "next up" card and the last ten activity entries
- estimated time remaining, from per-lesson estimates
- **a concept-mastery grid driven by `docs/concept-map.yaml`** — which Rust
  concepts you have met, deepened and mastered. This is the direct answer to
  "how do I know I actually know this?"

Lesson pages gain: in-page exercise checkboxes, a note field, a confidence
selector, prev/next, a sticky table of contents, and copy-code buttons.

Still SSR Rust + axum, still zero build step. The UI stays a teaching artifact
rather than becoming a second stack to maintain.

## 5. Quality gates

`scripts/lint-lessons` (new), run in CI:

1. Section presence and order in every `README.fa.md` / `README.md`.
2. FA/EN structural parity — same heading tree, same code blocks.
3. **Every Rust fence in every README compiles.** Deliberately-broken snippets
   are marked and must instead be asserted to produce the stated error code.
4. Every `examples/*.rs` builds and runs.
5. Concept-map ordering holds (no concept used before `introduced_in`).
6. Every internal markdown link resolves.
7. No `CHECKPOINT*.md` exists anywhere.
8. Every `todo!()` message describes *what*, not *how*.

Existing gates are kept: `cargo fmt --all --check`, strict clippy on finished
code, `cargo test -p course-ui`, `cargo test --workspace --no-run`.

## 6. Work packages

| WP | Scope | Gate |
|---|---|---|
| **WP-0** | Foundation: delete all 182 `CHECKPOINT*.md`; write `docs/lesson-standard.{fa.md,md}`; seed `docs/concept-map.yaml` for Phases 0–2; build `scripts/lint-lessons`; rewrite `.agents/skills/senpai-rust-course-author/SKILL.md` to the new standard; wire CI. | linter green, CI passes |
| **WP-1** | Web UI: fonts, CSS rewrite, theme toggle, progress v2 + migration, dashboard, lesson-page upgrades. Runs in parallel with WP-0 — no content dependency. | `cargo test -p course-ui`, strict clippy, visual check |
| **WP-2** | **Phase 0 rebuild** — 8 lessons, FA canonical + EN mirror, including "how to read a Rust error" and "how this course works". | full lesson linter |
| **WP-3** | **Phase 1 rebuild** — 33 lessons, 7 groups, reordered; `Vec`/`String` moved early; `From`/`?` added; the Persian-UTF-8 lesson; mini-project + review. | linter + `cargo test` on every solution crate + concept-map check |
| **WP-4** | Side-quest 1 realigned to Phase-1-only concepts, plus a format retro before scaling. | side-quest builds and runs |
| **WP-5** | Phase 2 rebuild (42 lessons). | as WP-3 |
| **WP-6** | Phase 3 rebuild (33 lessons). | as WP-3 |
| **WP-7** | Phase 4 rebuild (30 lessons). | as WP-3 |
| **WP-8** | Phase 5 — Persian translation, Rust artifacts, interview group. | as WP-3 |
| **WP-9** | Capstone and side-quests realigned to the final syllabus. | capstone builds, loadtest runs |

**Streaming resumes after WP-2.** WP-3 lands before Phase 0 is exhausted.

## 7. Non-goals

- No React, Tailwind, CDN, or build step for the web UI.
- No user accounts or cloud progress sync.
- No change to "one Cargo workspace, one lockfile, one `target/`".
- No rewrite of the TaskForge capstone architecture — only its sequencing and
  prerequisites are realigned.
- English is not dropped. It stays the repo's public face.
