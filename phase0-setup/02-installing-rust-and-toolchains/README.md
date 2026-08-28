# 02 — Installing Rust and toolchains

## At a glance

After this lesson you can:

- Install Rust and prove with one command that it really worked.
- Say what `rustup`, `rustc` and `cargo` each do and why they're three separate names.
- Explain the difference between a channel and an edition, and why this repo's `rust-toolchain.toml` matters.

**Time:** ~30 minutes, mostly waiting for downloads · **Prerequisites:** [01 — What is a compiled language?](../01-what-is-a-compiled-language/README.md)

> The exact install commands live in [`docs/setup-guide.md`](../../docs/setup-guide.md) so they stay in one place you can re-check later. **Go and install now**, then come back and finish reading this page.

---

## Why this matters

You'll be tempted to skip this one. "It's installed, it works, move on."

But three things here will each cost you an afternoon later if you don't know them:

- When a GitHub project says "requires nightly" and you don't know what that means.
- When you hit `linker 'link.exe' not found` on Windows and the message doesn't help at all.
- When a tutorial says `edition = "2024"` and your `Cargo.toml` says `2021`, and you don't know whether that's a problem.

All three take twenty minutes to explain. Let's explain them now.

---

## The concept

### `rustup` isn't an installer, it's a manager

You don't install "Rust" directly. You install `rustup`, and `rustup` fetches everything else.

The closest Python analogy is `pyenv` — the tool that lets you keep Python 3.9, 3.11 and 3.13 side by side and switch between them. With one important difference: `pyenv` is an optional third-party convenience, whereas `rustup` is the **official, universally-used** way to manage Rust versions. Effectively everyone uses it.

The three names you'll keep seeing:

| Name | What it is | Rough Python analogy |
|---|---|---|
| `rustup` | manages versions and components | `pyenv` |
| `rustc` | the compiler itself | `python` (the interpreter) |
| `cargo` | build tool and package manager | `pip` + `venv` + `build` + `pytest`, in one |

You will almost never invoke `rustc` directly. You work with `cargo`, and `cargo` invokes `rustc`. (The exception is `rustc --explain`, which you'll meet in lesson 05.)

A **toolchain** is the bundle of `rustc`, `cargo` and a few other pieces, all versioned together.

### Channels: stable, beta, nightly

Rust ships on three channels:

- **stable** — a new stable release every six weeks. Everything in this repo, and almost every job, is on this.
- **beta** — the next stable release, in testing.
- **nightly** — built each night from the latest code, with experimental features locked behind `#![feature(...)]`.

That six-week rhythm is deliberate: Rust doesn't do big noisy releases, it ships small things regularly. Which means upgrading is usually painless.

**When will you need nightly?** In this course, never. Some tools (certain clippy lints, some macro libraries) give a better experience on nightly, but none of it is required. If a tutorial says nightly is needed, go and find out why first — it's usually one specific feature, not "nightly" in general.

### `rust-toolchain.toml` — why your version matches mine

There's a file in the root of this repo:

```toml
[toolchain]
channel = "stable"
```

The effect: **any `cargo` or `rustc` command you run inside this repo uses stable**, regardless of what your global default is. `rustup` sees the file and switches for you.

It's the same idea as `.python-version` for `pyenv`, or a base image tag in a `Dockerfile`: **pin the version, don't rely on "whatever's installed."** When you're working on stream and someone wants to follow along, this is what makes their output match yours.

### Editions — and why they're not Python 2 versus 3

Every `Cargo.toml` has this:

```toml
edition = "2021"
```

An **edition** is how Rust introduces small, opt-in breaking changes every few years without breaking existing code. There have been 2015, 2018, 2021 and 2024 so far.

The key point — and what separates this from Python 2 to 3: **old-edition code keeps compiling forever**, and different crates within a single program can use different editions. `cargo` just needs to know which one each crate targets.

So if a tutorial has `edition = "2024"` and yours says `2021`, nothing is broken. A few syntactic details may differ.

### What gets installed where

Worth knowing, because one day you'll need it:

| Path | What's in it |
|---|---|
| `~/.rustup/` | the toolchains themselves. This is what gets large. |
| `~/.cargo/bin/` | the executables: `cargo`, `rustc`, `rustup`, and anything you `cargo install` |
| `~/.cargo/registry/` | downloaded crate sources. A cache shared by all your projects. |
| `target/` (per project) | build output. Gitignored, and safe to delete any time. |

On Windows, `~` is `C:\Users\<your name>`.

`~/.cargo/bin` needs to be on your `PATH` — the installer adds it, but **you have to close and reopen your terminal** for it to take effect. That's the most common "it didn't install" in the world.

### Two commands you'll want later

```sh
rustup show      # what's installed and which is active right now
rustup update    # bring everything to the latest
```

`rustup show` is the definitive answer whenever you're confused about which version is actually running.

And one more that fewer people know and is genuinely good:

```sh
rustup doc       # opens the entire Rust documentation offline, in your browser
```

The official book, the standard library reference, all of it. No internet needed. If your connection isn't reliable, this is worth a lot.

---

## Hands on

Install (following [`docs/setup-guide.md`](../../docs/setup-guide.md)), then run these four. All four should print a version number:

```sh
rustc --version
cargo --version
rustup --version
rustup show
```

Sample output (your numbers will differ):

```text
rustc 1.97.0 (a3b1b2c4d 2026-07-14)
cargo 1.97.0 (b8f2e1a90 2026-07-09)
rustup 1.28.1 (2026-05-22)
```

Then run this from inside this repo, and watch `rust-toolchain.toml` do its job:

```sh
rustup show
```

It should report stable as the active toolchain, pointing at this directory.

Finally, open the docs offline so you know they're there:

```sh
rustup doc
```

---

## Errors you will meet

### `cargo: command not found` immediately after installing

The install worked; your current terminal just hasn't seen the updated `PATH`.

**The fix:** close the terminal completely and open a new one. If you use VS Code, restart VS Code itself, not just its terminal panel.

**Why:** the installer adds `~/.cargo/bin` to your `PATH`, but a running process doesn't re-read its environment. This isn't Rust-specific; the same happens with any tool.

### On Windows: `linker 'link.exe' not found`

```text
error: linker `link.exe` not found
  |
  = note: program not found

note: the msvc targets depend on the msvc linker but `link.exe` was not found
note: please ensure that Visual Studio 2017 or later, or Build Tools for Visual Studio
      were installed with the Visual C++ option.
```

**What the compiler is objecting to:** `rustc` produced machine code, but turning that into an `.exe` needs a **linker** — and on Windows the linker is part of Microsoft's tooling, not part of Rust.

**The fix:** install "Build Tools for Visual Studio", making sure to tick the "Desktop development with C++" workload. (You don't need full Visual Studio, just the Build Tools.)

**Why that's the fix:** Rust does the compiling, not the linking. On Linux the same role is played by `cc`, which is usually already there; on Windows you have to bring it yourself. It's the most irritating step of installing on Windows, and it's a one-time thing.

### `error: no override and no default toolchain set`

`rustup` is installed but no toolchain is active.

**The fix:** `rustup default stable`

**Why:** this usually happens when an install was interrupted or hand-edited. One command fixes it.

---

## Exercises

### Warm up

<details>
<summary><code>rustup</code>, <code>rustc</code>, <code>cargo</code> — which do you invoke day to day?</summary>

`cargo`, almost always. `cargo` invokes `rustc` for you, and you only need `rustup` when changing or updating versions.

</details>

<details>
<summary>A tutorial says "requires nightly". Do you need to switch for this course?</summary>

No. Everything here works on stable. If a project does want nightly, find out which specific feature it needs first — it's usually one thing, not "nightly" in general.

</details>

<details>
<summary>Your global default is nightly. You run <code>cargo build</code> inside this repo. Which version runs?</summary>

Stable. The `rust-toolchain.toml` at the repo root overrides your global default for anything done inside this directory.

</details>

<details>
<summary>Are editions 2021 and 2024 like Python 2 and 3?</summary>

No. Old-edition code compiles forever, and different crates in one program can use different editions. Editions were designed specifically so that painful migration never repeats.

</details>

### Build

Run `rustup show` and read the output. Write down for yourself:

1. How many toolchains do you have installed?
2. Which is active, and **why** that one?
3. What is the `default host` it reports? (That's your **target** — the CPU architecture and operating system you're compiling for. You'll meet it again in Phase 4 when you build for Linux.)

### Challenge (optional)

Run `rustup component list`. You'll get a long list of optional pieces that can be added to a toolchain.

Find `clippy`, `rustfmt` and `rust-src` in it, and whether they're installed. You'll use the first two in lesson 06; the third is what lets rust-analyzer show you the source of a standard-library function when you click through to it.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `rustup` | the toolchain manager | switching or updating versions |
| `rustc` | the compiler itself | almost always indirectly, via `cargo` |
| `cargo` | build tool and package manager | every day |
| toolchain | `rustc` + `cargo` + components, versioned together | `rustup show` |
| channel (stable/beta/nightly) | release cadence | reading a project's requirements |
| `rust-toolchain.toml` | pins the version for one repo | this repo; any team project |
| edition | opt-in breaking changes, every few years | every `Cargo.toml` |
| linker | what turns machine code into an executable | the Windows install error |

### What you now know

- `rustup` manages, `rustc` compiles, `cargo` is what you actually invoke.
- Stable is what you use; you don't need nightly for this course.
- `rust-toolchain.toml` makes everyone's version match inside this repo.
- Editions aren't Python 2 to 3 — old code compiles forever.
- `rustup doc` has all the documentation offline.

### What comes back later

- **`cargo` properly** — creating projects, dependencies, tests: [03 — Cargo basics](../04-cargo-basics/README.md)
- **`clippy` and `rustfmt`, the components you just listed** — [06 — Tooling](../06-tooling-clippy-fmt-rust-analyzer/README.md)
- **Targets and cross-compiling for Linux** — [Phase 4 — Deployment and operations](../../phase4-backend-advanced/07-deployment-and-operations/README.md)

### Can you explain?

- What's the difference between `rustup` and `cargo`?
- What are Rust's three channels, and which do you use?
- What does `rust-toolchain.toml` do, and why does it matter for teamwork?
- Why aren't Rust editions like Python 2 versus 3?
- On Windows, what provides `link.exe` and why doesn't Rust ship it?

---

## Going further

- [`rustup` book](https://rust-lang.github.io/rustup/) — the full reference for the tool.
- [Rust editions guide](https://doc.rust-lang.org/edition-guide/) — what changed in each edition and how you migrate.
- [Rust release channels](https://forge.rust-lang.org/infra/channel-layout.html) — background on how releases are built.
