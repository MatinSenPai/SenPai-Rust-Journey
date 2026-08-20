# 04 — Cargo basics

## At a glance

After this lesson you can:

- Read every part of a `Cargo.toml` and say what it does — from `[package]` to `[dependencies]`.
- Add a dependency, see where it came from, and say what `Cargo.lock` guarantees.
- Tell apart the six commands you'll run every day and know when each is the right one.

**Time:** ~40 minutes · **Prerequisites:** [03 — Hello, Rust](../03-hello-rust/README.md)

---

## Why this matters

Last lesson you typed `cargo` a lot without being told what it is. Now it's time.

You're coming from Python, where this work is spread across several tools: `pip` installs packages, `venv` makes environments, a `Makefile` or `tox.ini` collects commands, `pytest` runs tests, `black` formats, `flake8` lints. Each with its own config, its own docs.

`cargo` is all of them, in one tool, with one config file, identical in every Rust project in the world.

It's one of the reasons Rust is a pleasure to work in despite being a hard language. It's worth an afternoon.

---

## The concept

### `cargo` replaces several tools at once

| What you're doing | Python | Rust |
|---|---|---|
| Start a new project | `mkdir` + fiddling | `cargo new` |
| Install a dependency | `pip install` | `cargo add` |
| Isolate environments | `venv` | automatic; every project is separate |
| Build | `setup.py` / `build` | `cargo build` |
| Run | `python app.py` | `cargo run` |
| Test | `pytest` | `cargo test` |
| Format | `black` | `cargo fmt` |
| Lint | `flake8` / `ruff` | `cargo clippy` |
| Lock versions | `pip freeze` / `poetry.lock` | `Cargo.lock` (automatic) |

Look at the "isolate environments" row: Rust has no virtual environments at all, because you never install dependencies "globally". Every project has its own, always. That entire category of problem disappears.

### Anatomy of `Cargo.toml`

Open this lesson's `Cargo.toml`:

```toml
[package]
name = "p0-04-cargo-basics"
version = "0.1.0"
edition.workspace = true

[dependencies]
rand.workspace = true
```

- **`[package]`** — metadata about *this* crate: its name (how others would depend on it), its version, and which language edition it targets.
- **`[dependencies]`** — other crates this one needs. The first time you build, they're downloaded from [crates.io](https://crates.io) — Rust's PyPI.

`edition.workspace = true` and `rand.workspace = true` both mean "use whatever the repo root's `Cargo.toml` pins".

### Crate, package, workspace

Three words people constantly mix up:

- **crate** — a unit of compilation. Either a library or a binary. `src/lib.rs` is one crate; `src/main.rs` is another.
- **package** — anything that has a `Cargo.toml`. It can hold at most one library crate and any number of binary crates. This lesson is one package containing two crates.
- **workspace** — several packages sharing one `Cargo.lock` and one `target/` directory.

**This whole repo is one workspace.** That means when `tokio` compiles once, all hundred lessons that need it share that build — rather than compiling it a hundred times. It's also why you see `-p` in the commands: `-p` says "which package".

```sh
cargo test                          # every package in the workspace
cargo test -p p0-04-cargo-basics    # just this one
```

The full naming rules, and why each lesson has a prefixed package name, are in [`docs/conventions.md`](../../docs/conventions.md).

### Dependencies and crates.io

Adding a dependency is one command:

```sh
cargo add rand
```

That edits `Cargo.toml` for you, finds the latest compatible version, and downloads it on your next build. You don't need to open the file by hand.

See what this lesson pulls in:

```sh
cargo tree -p p0-04-cargo-basics --depth 1
```

```text
p0-04-cargo-basics v0.1.0 (M:\SenPai-Rust-Journey\phase0-setup\04-cargo-basics)
└── rand v0.8.6
```

Drop `--depth 1` to see the whole tree — dependencies of dependencies. It's a good habit: before adding a crate to a real project, look at what it drags along.

### Semantic versioning and `Cargo.lock`

`version = "0.1.0"` is `MAJOR.MINOR.PATCH`. The convention:

| Bump | What it means | Example |
|---|---|---|
| PATCH (`1.2.3` → `1.2.4`) | bug fix, nothing breaks | always safe |
| MINOR (`1.2.3` → `1.3.0`) | new feature, nothing breaks | safe |
| MAJOR (`1.2.3` → `2.0.0`) | may break things | read the notes |

**Before `1.0.0` the rule differs:** for `0.x.y`, a MINOR bump *may* break. So `rand 0.8` can be incompatible with `rand 0.9`. That's the signal a crate gives you that it's still stabilising.

Python packages *often* follow the same convention, but cargo and crates.io lean on it far harder — dependency resolution is built directly on it.

**`Cargo.lock`** records the *exact* version that was actually chosen. `Cargo.toml` says "any `0.8.x`"; `Cargo.lock` says "exactly `0.8.6`". The rule:

- For a **binary** (something you deploy): commit `Cargo.lock`. You want the server building exactly what you tested.
- For a **library** (something you publish): don't. Your consumers need to pick their own.

This repo commits it, because everyone should get an identical build.

### The six commands you'll run every day

| Command | What it does | When |
|---|---|---|
| `cargo check` | compile-checks only, no binary | constantly, while writing |
| `cargo build` | produces a binary under `target/debug/` | when you actually want one |
| `cargo run` | builds if needed, then runs | trying the program |
| `cargo test` | runs every `#[test]` function | when you think it's right |
| `cargo fmt` | formats to the standard style | before every commit |
| `cargo clippy` | lints for non-idiomatic patterns | before every commit (lesson 06) |

On a project this small the `check` versus `build` gap barely registers:

```text
cargo check → 0.87s
cargo build → 1.20s
```

Negligible at this size. On a real project with fifty dependencies, that same gap becomes ten seconds versus two minutes. Which is why you reach for `check`.

### What `target/` is and why it gets big

Everything `cargo` builds goes into `target/`: your compiled crate, every compiled dependency, test executables, all of it.

It easily reaches several gigabytes. That's normal, and it's gitignored.

```sh
cargo clean    # delete all of it
```

The next build then starts from scratch and is slow. Run `cargo clean` when you need disk space, or when you suspect a corrupted build cache — not as a habit.

---

## Hands on

First see what a dependency buys you:

```sh
cargo run -p p0-04-cargo-basics --example 01-dependency-tour
```

```text
a random number: 215
a coin flip:     true

Neither of those lines is in the standard library.
They work because `Cargo.toml` says `rand.workspace = true`.
```

(The numbers change each run — that's `rand` doing its job.)

Then run these and read the output:

```sh
cargo tree -p p0-04-cargo-basics                 # every dependency, all the way down
cargo tree -p p0-04-cargo-basics --depth 1       # direct dependencies only
```

Finally, create a completely separate project elsewhere so you see `cargo new`:

```sh
cd /tmp            # or anywhere outside this repo
cargo new hello-cargo
cd hello-cargo
cargo run
```

Four files got created. Look at all four — in particular, note that `cargo new` ran `git init` for you.

---

## Errors you will meet

### `error: package ID specification ... did not match any packages`

```text
error: package ID specification `p0-04-cargo` did not match any packages

help: a package with a similar name exists: `p0-04-cargo-basics`
```

**What cargo is objecting to:** the name you gave `-p` isn't the name of any package in this workspace.

**The fix:** use the full name. Cargo suggests the nearest match.

**Why that's the fix:** `-p` takes a **package** name (the `name` under `[package]`), not a directory path. They usually look similar but aren't the same — directory names start with a digit and package names can't. `cargo metadata --no-deps` lists every name available.

### A dependency that isn't in `Cargo.toml`

If your code uses `rand` but you remove it from `Cargo.toml`:

```text
error[E0432]: unresolved import `rand`
 --> src/lib.rs:1:5
  |
1 | use rand::random;
  |     ^^^^ use of unresolved module or unlinked crate `rand`
  |
  = help: you might be missing a crate `rand`
```

**What the compiler is objecting to:** your code names a crate this package doesn't depend on.

**The fix:** `cargo add rand` — or in this repo, put `rand.workspace = true` back.

**Why that's the fix:** unlike Python, where anything installed on the machine is importable, every Rust package must declare what it depends on explicitly. It's stricter, and it's why "it worked on my machine" almost never happens in Rust.

---

## Exercises

### Warm up

<details>
<summary>What's the difference between a crate and a package?</summary>

A package is anything with a `Cargo.toml`. A crate is a unit of compilation. One package can hold a library crate and several binary crates — this lesson is one package with two crates (`lib.rs` and `main.rs`).

</details>

<details>
<summary>Why is this repo one workspace instead of a hundred separate projects?</summary>

One `Cargo.lock` and one `target/`. Every shared dependency compiles once, not a hundred times. Separate projects would mean a hundred copies of `tokio` on your disk.

</details>

<details>
<summary><code>Cargo.toml</code> says <code>rand = "0.8"</code>. Will you get <code>0.9</code>?</summary>

No. For `0.x` versions a MINOR bump may break, so `"0.8"` means "any `0.8.x`", not "anything below `1.0`". After `1.0` it behaves differently: `"1.2"` means any `1.x` that's at least `1.2`.

</details>

<details>
<summary>You've written a web service you deploy. Do you commit <code>Cargo.lock</code>?</summary>

Yes. For binaries you want the server building exactly the versions you tested. For libraries you publish, no — let the consumer choose.

</details>

### Repair

In this lesson's `Cargo.toml`, comment out the `rand.workspace = true` line and run `cargo check -p p0-04-cargo-basics`.

Read the error you get and say, from the message alone, which file and which line broke. Then put it back.

### Implement

Two functions in `src/lib.rs`:

```sh
cargo test -p p0-04-cargo-basics
```

The second one — `pick_encouragement` — deliberately needs `rand`. Open its documentation on your own machine:

```sh
cargo doc -p p0-04-cargo-basics --open
```

That builds the docs for every one of your dependencies and opens them in a browser. Get into the habit of using this instead of searching the web — this version is exactly the one you're using, not one published three years ago.

Then run the program, with and without an argument:

```sh
cargo run -p p0-04-cargo-basics
cargo run -p p0-04-cargo-basics -- Matin
```

That `--` matters: everything before it is for `cargo`, everything after it is for your program.

### Build

In `/tmp` (or anywhere outside this repo), create a new project and add a dependency to it:

```sh
cargo new my-first-crate
cd my-first-crate
cargo add rand
```

Look at `Cargo.toml` before and after `cargo add`. Then write something in `main.rs` that genuinely uses `rand`, and run it.

The point is to have done this once from nothing, so that when you build a service in Phase 3, this part is already familiar.

### Challenge (optional)

Run `cargo tree -p p0-04-cargo-basics` in full. `rand` brings dependencies of its own.

Now look up two crates on [crates.io](https://crates.io) and compare their dependency counts: `serde_json` and `reqwest`. Then ask yourself why adding a crate to a real project is a decision worth thinking about — not just for build time, but for whose code you're choosing to trust.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| crate | a unit of compilation: one library or one binary | every `lib.rs` or `main.rs` |
| package | anything with a `Cargo.toml` | every lesson directory |
| workspace | packages sharing a `Cargo.lock` and `target/` | this entire repo |
| `-p <name>` | "work on this package" | every command in this repo |
| crates.io | the public package registry | Rust's PyPI |
| semantic versioning | `MAJOR.MINOR.PATCH` and the promise it makes | reading any `Cargo.toml` |
| `Cargo.lock` | the exact versions actually chosen | binaries commit it, libraries don't |
| `--` | the boundary between cargo's arguments and your program's | `cargo run -- ...` |

### What you now know

- `cargo` covers what `pip`, `venv`, `pytest`, `black`, `flake8` and a `Makefile` do between them.
- A package has a `Cargo.toml`; a crate is a compilation unit; this repo is a workspace.
- `cargo add` adds a dependency and `cargo tree` shows what came with it.
- `0.x` versions follow different rules, and `Cargo.lock` makes your build reproducible.
- `cargo check` is fast; `cargo build` produces a binary.
- `target/` gets big, and that's fine.

### What comes back later

- **Reading the errors these commands print** — [05 — Reading compiler errors](../05-reading-compiler-errors/README.md)
- **`cargo fmt` and `cargo clippy` properly** — [06 — Tooling](../06-tooling-clippy-fmt-rust-analyzer/README.md)
- **Modules, visibility, and building your own workspace** — [Phase 2 — Project organisation](../../phase2-intermediate/06-project-organization-and-testing/01-modules-visibility-workspaces/README.md)
- **Cargo features** — the `--features` you saw in lesson 05: [Phase 2 — Toolbox](../../phase2-intermediate/08-rust-toolbox/03-cargo-features/README.md)
- **`Cargo.lock` and reproducible production builds** — [Phase 4 — Deployment](../../phase4-backend-advanced/07-deployment-and-operations/README.md)

### Can you explain?

- Which Python tools does `cargo` replace?
- What's the difference between a crate, a package and a workspace?
- What does `-p` do, and why is it needed in this repo?
- What's the difference between `Cargo.toml` and `Cargo.lock`, and which do you commit?
- Why doesn't `rand = "0.8"` get you `0.9`?
- In `cargo run -- Matin`, what does the `--` do?

---

## Going further

- [The Cargo Book](https://doc.rust-lang.org/cargo/) — the full reference. Don't read it cover to cover; go to it when you have a question.
- [crates.io](https://crates.io) — browse. See how well-regarded crates document themselves.
- [Cargo Book — Dependency resolution](https://doc.rust-lang.org/cargo/reference/resolver.html) — for the day two crates want two different versions of the same thing.
