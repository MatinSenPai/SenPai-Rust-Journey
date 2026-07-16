# Start here — a newcomer's guide

Welcome. If you've never written a line of Rust — or never done systems
programming at all — this page is for you. It explains what this project is,
how it works, and exactly what to do on your first day. Read it once, top to
bottom. It takes about ten minutes, and it'll save you hours of "wait, how am
I supposed to use this?"

The main [`README.md`](../README.md) is the *map* of the journey. This is the
*how to walk it*.

---

## 1. What is this project?

It's a **self-paced course that turns you into a backend engineer using Rust**,
built out of small hands-on exercises you complete on your own machine. You
start at "what even is a compiled language" and end by having built
**TaskForge**, a real Postgres-backed job-queue service — the kind of thing
companies actually pay engineers to build.

It is **not** a video course or a book you passively read. Every lesson is code
you write yourself, checked by tests that go green when you get it right. You
learn by doing, and you can't fool the compiler.

**How long does it take?** Think months of steady evenings, not a weekend. That's
normal and it's the point — you're building durable skill, not cramming syntax.

---

## 2. What is Rust, in one minute?

Rust is a programming language known for being **fast** and **safe at the same
time** — a combination that used to mean "pick one."

- **Compiled.** You run your code through a *compiler* (`cargo`) which turns it
  into a standalone program before it runs. (Python, by contrast, is
  interpreted line-by-line as it runs.)
- **No garbage collector.** Languages like Python, Java, and Go have a
  background process that cleans up unused memory for you, which costs speed and
  predictability. Rust has none. Instead, the *compiler* figures out exactly
  when to free memory, at compile time, by enforcing a set of rules called
  **ownership**. This is the famous hard part — and the thing this course spends
  real time teaching you, because once it clicks, it's a superpower.
- **Safe.** That same compiler refuses to build code with entire categories of
  bugs (use-after-free, data races, null-pointer crashes). If it compiles, a
  whole class of 3 a.m. production incidents simply can't happen.

The trade-off: the compiler is *strict* and will reject code that other
languages would happily run and crash on later. Early on this feels like
fighting it. By Phase 2 it feels like a pair-programming partner who catches
your mistakes before anyone else sees them.

Every unfamiliar word in the lessons is defined in plain English in
[`docs/glossary.md`](glossary.md) — keep it open in a tab.

---

## 3. Do I need to know Python?

**No.** You need to be comfortable with *some* programming — variables, loops,
functions, the idea of an API and a database — but not any specific language.

The lessons frequently say "in Python you'd do X; in Rust you do Y." That's
because the course was originally written for a Python/Django developer, and
those contrasts are a helpful teaching tool. If you know Python, they're a
shortcut. If you don't, just read them as "here's a common way other languages
do this, and here's why Rust is different" — you lose nothing.

---

## 4. Set up your machine (once)

Three things, in order. The full walk-through with exact commands is in
[`docs/setup-guide.md`](setup-guide.md); here's the short version:

1. **Install Rust** via `rustup` (the official installer/version-manager).
   - macOS/Linux: run the one-line `curl` command from the setup guide.
   - Windows: download and run `rustup-init.exe` from
     <https://rustup.rs>. Accept the defaults.
   - Then close and reopen your terminal and check it worked:
     `rustc --version` and `cargo --version` should both print a version.
2. **Install an editor with rust-analyzer.** VS Code + the "rust-analyzer"
   extension is the easy default. This gives you red squiggles under mistakes
   *as you type*, which is the single biggest accelerator when you're learning.
3. **Prove it all works.** From the project's top folder, run:
   ```sh
   cargo build --workspace
   ```
   This compiles every lesson at once. The first time is slow (it downloads and
   builds a lot) — that's expected. When it finishes without errors, you're
   ready.

You do **not** need PostgreSQL, Docker, or anything else until Phase 3. Each
lesson tells you what to install right when you need it.

---

## 5. The shape of the journey

You move through **phases**, in order. Each phase folder has its own `README.md`
listing its lessons. Here's the whole arc and what you'll be able to *do* at the
end of each:

| Phase | You'll be able to… |
|---|---|
| **0 — Setup** | Install the tools, run your first Rust program, use `cargo`. |
| **1 — Fundamentals** | Read and write basic Rust; understand ownership, borrowing, `Option`/`Result` — the concepts everything else builds on. |
| **2 — Intermediate & idiomatic** | Write Rust the way pros do: iterators, traits, generics, error handling, and concurrency/`async`. Includes a **Rust toolbox** module (pattern matching, macros, `TryFrom`, Cargo features). |
| **3 — Backend foundations** | Build a real web API with `axum`, talk to PostgreSQL, do auth, transactions, and pagination. |
| **4 — Backend advanced** | Add caching, rate limiting, background jobs, gRPC/GraphQL, metrics, config, and deployment. |
| **Capstone — TaskForge** | Study a production-grade job-queue codebase, then make it *run* and *scale*. Your portfolio centerpiece. |
| **5 — System design** | Talk fluently about databases at scale, caching, distributed systems, and design real systems in interviews. |

Sprinkled between phases are **side-quests** (in [`side-quests/`](../side-quests)) —
smaller, fun projects (a Telegram bot, an anime/manga API) that put the fresh
skills to work. They're optional but strongly recommended: they're where the
abstract stuff becomes real.

---

## 6. How a single lesson works — the most important section

Every code lesson is a little **workbook**: some code is written for you, and
some is left blank for you to fill in, with tests that tell you when you got it
right. Open any lesson folder (say
`phase1-fundamentals/02-ownership-and-memory/01-move-semantics/`) and you'll see:

| File | What it is |
|---|---|
| `README.md` | **Read this first.** The theory, why it matters, and what you're about to build. |
| `src/lib.rs` | The code. Function *signatures* are written; the *bodies* are left as `todo!("hint…")` for you to complete. |
| `tests` (inside `src/lib.rs` or a `tests/` folder) | Automated checks. They pass only when your code is correct — this is your answer key. |
| `CHECKPOINT.md` | A few questions to answer *in your own words* (out loud is fine). Tests check your code; these check your *understanding*. |
| `solution/` | A complete reference answer plus `SOLUTION.md` explaining the *reasoning*. Only open it after you've tried. |

**`todo!()` is the heart of it.** It's a real Rust command that means "not
written yet." The code compiles with it in place, but if it runs, it stops
there. Your job each lesson is to replace each `todo!("hint")` with real code —
the hint text is practically the answer, so you're never truly stuck.

### The loop you'll repeat, lesson after lesson

1. **Read** the lesson's `README.md` fully before touching code.
2. **Open `src/lib.rs`** and replace each `todo!(...)` with real code.
3. **Run the tests** until they're green:
   ```sh
   cargo test -p <package-name>
   ```
   (Where to find `<package-name>`? It's the `name = "…"` line at the top of the
   lesson's `Cargo.toml` — they follow a pattern like `p1-02-01-move-semantics`:
   phase 1, module 02, lesson 01. More on the naming in
   [`docs/conventions.md`](conventions.md).)
4. **Run the linter** and actually read what it says — it teaches idiomatic Rust:
   ```sh
   cargo clippy -p <package-name>
   ```
5. **Answer `CHECKPOINT.md`** in your own words. If you can't, re-read the README.
6. **Only now** open `solution/SOLUTION.md` and compare your *reasoning*, not
   just whether the characters match.
7. **Tick the box** for that lesson in [`PROGRESS.md`](../PROGRESS.md).

That's it. Every lesson is that same rhythm.

---

## 7. The handful of commands you'll actually use

You can run everything from the project's top folder. `-p` means "just this one
package."

```sh
cargo build --workspace          # compile everything (used once at setup)
cargo test  -p <package-name>    # run one lesson's tests — your main command
cargo clippy -p <package-name>   # lint one lesson — read every suggestion
cargo fmt   -p <package-name>    # auto-format your code neatly
cargo run   -p <package-name>    # actually run a lesson that's a program
```

If you're not sure of a package name, you can also `cd` into the lesson folder
and run `cargo test` (no `-p` needed) — it acts on whatever crate you're inside.

---

## 8. When it doesn't compile (it will, constantly — that's fine)

This is the mental shift that makes or breaks people learning Rust: **the
compiler is not your enemy, it's your teacher.** It fails *loudly and early* so
your program doesn't fail *quietly and late*.

- **Read the whole error, top to bottom.** Rust's error messages are unusually
  good — they usually name the exact problem and often print the fix.
- **Warnings on unfinished lessons are expected.** A skeleton full of `todo!()`
  will show "unused variable" warnings — that's just the compiler noting you
  haven't used something *yet*. They disappear as you fill the lesson in. Don't
  panic at yellow text.
- **Unknown word?** Check [`docs/glossary.md`](glossary.md) before anything else.
- **Truly stuck?** The hint in the `todo!()`, then the lesson README, then
  `solution/SOLUTION.md`. And the free official [Rust Book](https://doc.rust-lang.org/book/)
  is an excellent second explanation of any core concept.

Fighting the "borrow checker" (the part of the compiler that enforces
ownership) is a rite of passage. Everyone does it. It stops being a fight
surprisingly soon.

---

## 9. Tracking where you are

[`PROGRESS.md`](../PROGRESS.md) is your checklist for the entire journey. Tick a
box the moment you finish a lesson. When you come back after a break, it's the
first place to look — "where was I?" is answered in one glance.

If you use git, the project suggests one commit per finished lesson (the exact
convention is in [`docs/conventions.md`](conventions.md)). This isn't required to
learn, but it builds a habit real teams use, and your commit history becomes a
visible record of the mountain you climbed.

---

## 10. Your literal first hour

1. Do the setup in section 4 (install Rust, editor, `cargo build --workspace`).
2. Open [`phase0-setup/README.md`](../phase0-setup/README.md) and read it.
3. Start at lesson 1, `phase0-setup/01-what-is-a-compiled-language/` — it's pure
   reading, no code, just orientation.
4. Continue to `03-cargo-basics/` — your first real code lesson. Do the loop from
   section 6. When its tests go green, you've officially written and verified
   your first Rust. 🎉
5. Keep going, one lesson at a time, at whatever pace fits your life.

There's no rush and no deadline. The person who does one lesson a day for three
months finishes having built a genuinely impressive backend service — and
understands every line of it.

Welcome aboard. Open [`phase0-setup/README.md`](../phase0-setup/README.md) and
begin.
