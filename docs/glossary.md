# Glossary

Plain-English definitions, added to as new jargon shows up in lessons. If a
term confuses you and it's not here yet, add it once you've figured it out —
future-you (and anyone else following this repo) will thank you.

- **Compiler** — a program that translates source code (Rust) into machine
  code (a binary your OS can run directly), *before* you run it. Contrast
  with Python, which is interpreted line-by-line at run time by `python3`.
- **Binary / executable** — the compiled output file `cargo build` produces
  (under `target/debug/` or `target/release/`). No Rust toolchain is needed to
  run it, unlike a `.py` file which always needs a Python interpreter present.
- **Crate** — Rust's unit of compilation and distribution — roughly
  equivalent to a Python "package" (a `pip`-installable thing with a name and
  version), but a crate can produce either a library (`lib.rs`) or an
  executable (`main.rs`).
- **Cargo** — Rust's build tool + package manager, playing the role of
  `pip` + `venv` + `setuptools` + a task runner, all in one CLI.
- **Workspace** — a group of crates that share one dependency lock file and
  one build output directory. This whole repo is one workspace.
- **Ownership** — Rust's core memory-management rule: every value has exactly
  one owner responsible for cleaning it up. No garbage collector, no manual
  `free()` — the compiler enforces this at compile time.
- **Move** — when a value's ownership transfers to a new variable, the old
  variable becomes invalid. Unlike Python, where `b = a` just adds another
  name pointing at the same object.
- **Borrow / reference (`&`, `&mut`)** — temporary, checked access to a value
  you don't own. You can have many read-only borrows, or exactly one mutable
  borrow, never both at once ("aliasing XOR mutability").
- **Lifetime** — the compiler's bookkeeping for *how long* a reference stays
  valid, written as `'a`. Almost always inferred; you only write it
  explicitly when the compiler can't figure out the relationship itself.
- **`Option<T>`** — Rust's answer to "this might not have a value" — instead
  of `None`/`null` sneaking in anywhere, absence is an explicit type you must
  handle before you can use the value.
- **`Result<T, E>`** — Rust's answer to recoverable errors — a function that
  can fail returns `Result`, forcing the caller to handle both the success
  (`Ok`) and failure (`Err`) case, instead of relying on exceptions.
- **Trait** — Rust's version of an interface/protocol: a set of methods a
  type promises to implement. Similar in spirit to a Python Protocol or ABC,
  but resolved at compile time by default.
- **Generic** — code written once, parameterized over a type, e.g.
  `fn largest<T>(list: &[T]) -> &T`. Compiled separately for each concrete
  type used (monomorphization) rather than resolved at runtime like Python's
  duck typing.
- **`unsafe`** — an escape hatch that lets you do a small set of operations
  the compiler can't verify are safe (raw pointer deref, calling C code,
  etc.), with the promise that *you've* verified it by hand. Most Rust code
  never needs it.
- **`async`/`await`, Future** — Rust's model for concurrent I/O-bound work: an
  `async fn` returns a `Future`, a value representing "work that will
  complete later," which does nothing until a runtime (like `tokio`) polls it.
- **Runtime (async)** — the scheduler that actually drives `Future`s to
  completion (spawns tasks, wakes them up when I/O is ready). Rust's standard
  library deliberately ships without one — you choose (almost always
  `tokio` for backend work).
- **`Arc`, `Mutex`** — `Arc` ("atomic reference count") lets multiple threads
  share ownership of a value; `Mutex` ensures only one thread can mutate it at
  a time. The combination (`Arc<Mutex<T>>`) is the most common way to share
  mutable state across threads/tasks.
- **Idempotency** — an operation that produces the same end result no matter
  how many times it's applied (e.g. "set balance to $10" vs. "add $10").
  Critical for retried network requests and job queues.
- **Backpressure** — a system's way of saying "slow down" to whatever is
  sending it work, instead of silently queuing forever or falling over.
