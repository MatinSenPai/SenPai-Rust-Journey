# 01 — What is a compiled language?

No code in this lesson. Read it before installing anything — the rest of the
journey makes a lot more sense once this clicks.

## Where Python starts

When you run `python manage.py runserver`, here's what actually happens:
`python3` (an interpreter — itself a compiled program, but that's a detail
you never think about) reads your `.py` files as text, and line by line,
translates each one into an action and immediately performs it. Every single
time you run the file, this translation happens again, fresh, from the text.

This is why:
- You don't need a separate "build" step — `python foo.py` just works.
- A typo in a function you never call won't be caught until that line
  actually executes (or not at all, if it's never hit).
- Distributing a Python program to someone means either giving them your
  source `.py` files *and* making sure they have a compatible Python
  installed, or bundling a whole interpreter with tools like PyInstaller.

## Where Rust starts

Rust is **compiled, ahead of time**. `cargo build` runs the Rust compiler
(`rustc`) once, which reads your entire program, checks it exhaustively, and
translates it directly into a **binary** — a file of actual machine
instructions your CPU can execute directly, with no translation step left to
do at run time.

Consequences that will matter constantly for the next few months:

- **A typo, a type mismatch, a borrow-checker violation — anywhere in your
  program — is caught before it ever runs**, not just on the line you
  happen to execute. This is the trade you're making: more friction *before*
  running the program, in exchange for entire categories of bugs (null
  pointer errors, most data races, use-after-free) becoming compile errors
  instead of 3am production incidents.
- **Running the program is instant and needs nothing installed** — the
  compiled binary under `target/debug/your_program` runs directly on the
  OS. You could copy that one file to another Linux machine (same
  architecture) with no Rust toolchain installed at all, and it would run.
- **Compiling takes real time** (seconds to minutes, growing with project
  size), and it's a genuinely separate step from running. `cargo run` does
  both for convenience, but it's still "compile, then execute," not
  "execute directly."

## A mental model, not a metaphor

Think of it like the difference between:
- **Python**: handing someone a recipe written in your native language and a
  translator who reads it out loud to them step by step, every single time
  they cook it — the translator can catch a nonsensical instruction only
  when they reach it out loud.
- **Rust**: handing someone the recipe, having an extremely pedantic editor
  read the *entire* recipe first and refuse to hand it over until every
  single instruction is unambiguous and physically possible to perform, and
  only then do they get a clean, standalone card they can cook from as many
  times as they want without the editor present.

That "extremely pedantic editor" is the compiler, and specifically its
**borrow checker** (Phase 1) is the part of it most unique to Rust — no
other mainstream language enforces memory-safety rules this way at compile
time instead of at run time (garbage collection) or not at all (C/C++).

## Checkpoint

Move to `CHECKPOINT.md` in this folder before continuing to lesson 02.
