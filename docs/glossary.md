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

## Memory and ownership

- **Destructor** — the code that runs as a value is destroyed. In Rust that is
  `Drop::drop`, and you never call it yourself; the compiler inserts the call
  at the closing brace of the owner's scope.
- **RAII** (resource acquisition is initialisation) — acquire the resource when
  the value is created, release it in the destructor. Scope closes the file,
  not discipline. It is why Rust has no `finally` block and does not need one.
- **Dereference (`*`)** — following a reference to the value at the other end.
  `*count += 1` changes the number; `count += 1` would try to change the arrow.
- **Auto-deref** — the compiler inserting `*` for you on a method call, which
  is why `text.len()` works whether `text` is a `String` or a `&String`.
- **Aliasing rule** — any number of shared borrows, *or* exactly one mutable
  borrow, never both at once. The single rule the borrow checker enforces, and
  the reason a data race cannot be written in safe Rust.
- **Data race** — two threads touching the same memory at the same time with at
  least one of them writing. The aliasing rule makes it unrepresentable.
- **Iterator invalidation** — mutating a collection while walking it, so the
  walk is left pointing at memory that has moved. A run-time crash in C++ and a
  compile error here.
- **Borrow scope** — how long a borrow actually lasts: from where it is taken
  to its **last use**, not to the end of the block.
- **Non-lexical lifetimes (NLL)** — the rule that gives borrows that shorter,
  use-based scope. Before Rust 2018 a borrow lasted to the closing brace, and a
  great deal of correct code was rejected.
- **Two-phase borrow** — the compiler's allowance that makes
  `items.push(items.len())` legal: the arguments are evaluated before the
  mutable borrow becomes active.
- **Slice (`&[T]`, `&str`)** — a borrowed view of a contiguous run of values.
  Two words: where it starts and how many there are.
- **Fat pointer** — a reference carrying a second word alongside the address. A
  slice carries a length; a trait object carries a vtable.

## Text

- **String literal** — text written in the source, e.g. `"hello"`. Baked into
  the executable and typed `&'static str`, so it is a view, never an owner.
- **Unsized type** — a type whose size is not known at compile time, such as
  bare `str` or `[T]`. You can never hold one directly, only behind a
  reference or a `Box`.
- **Deref coercion** — the compiler turning a `&String` into a `&str` (or a
  `&Vec<T>` into a `&[T]`) at a call site. It is why taking `&str` in a
  parameter costs the caller nothing.
- **Unicode scalar value** — one code point, which is what a Rust `char` holds.
  Four bytes in memory, one to four bytes when written as UTF-8.
- **UTF-8** — the encoding Rust strings always use. ASCII takes one byte,
  Persian and Arabic letters two, most other scripts three, emoji four.
- **Continuation byte** — every byte of a multi-byte character after the first,
  recognisable because it starts with the bits `10`. Never a character on its
  own, which is what makes a mis-aimed slice detectable.
- **Char boundary** — a byte offset where a character actually starts. Slicing
  anywhere else panics rather than producing broken text.
- **Combining mark** — a character that modifies the one before it, like a
  Persian fatha. One thing on screen, two Unicode scalars.
- **Grapheme cluster** — what a person means by "a character": one or more
  scalars that display as a single unit. `.chars().count()` does not count
  these, and the standard library deliberately does not offer them.
- **ZWNJ (zero-width non-joiner, `\u{200C}`)** — the Persian half-space that
  keeps letters from joining, as in «می‌روم». Three bytes, one `char`, no
  width — so it silently breaks any layout that counts characters as columns.
- **Normalisation** — rewriting text into a canonical form so that two spellings
  of the same thing compare equal. Persian text needs it: the Arabic ك and the
  Persian ک look alike and are different characters.
- **`Display` / `Debug`** — the two ways of turning a value into text.
  `Display` (`{}`) is for the user; `Debug` (`{:?}`) is for you. They are
  separate traits because they are separate audiences.

## Your own types

- **Tuple struct** — a struct whose fields have positions instead of names:
  `struct Meters(f64);`. Reached with `.0`.
- **Unit struct** — a struct with no fields at all: `struct Marker;`. Zero
  bytes, and useful purely as a type.
- **Newtype pattern** — wrapping a primitive in a tuple struct so the type
  system can tell two things apart that are both, underneath, a `u64`. Free at
  run time, and it turns "I passed the arguments in the wrong order" from a
  production incident into a compile error.
- **Refutable / irrefutable pattern** — a pattern that might not match
  (`Some(x)`) versus one that always does (`(a, b)`). `let` needs an
  irrefutable one, which is why `let Some(x) = ...` alone is an error.
- **Diverging** — an expression that never produces a value because control
  never comes back: `return`, `break`, `panic!`, `todo!`. Its type is `!`, so
  it fits wherever a value is wanted.
- **Panic** — an unrecoverable failure. It unwinds the stack, running every
  destructor on the way, and is for bugs — a broken invariant — not for
  failures a caller could reasonably handle.
- **Struct literal** — the expression that builds a struct,
  `Series { title, episodes }`. **Field init shorthand** lets you write
  `title` instead of `title: title` when the variable already has the name.
- **Struct update syntax** — `Series { watched: 0, ..other }`: take these
  fields from `other`. It *moves* out of `other` unless every field is `Copy`.
- **Partial move** — moving one field out of a struct, which leaves the struct
  itself unusable while the remaining fields are still fine.
- **Associated function** — a function in an `impl` block with no `self`, called
  as `Series::new(...)`. `new` is a convention, not a keyword.
- **Invariant** — something a type promises is always true of its values, kept
  true by making fields private and only changing them through methods.
- **Enum** — a type that is exactly one of several shapes. In Rust each shape
  may carry its own data, which is what makes it a **sum type** rather than the
  named-integer enum of C or Java.
- **Variant** — one of those shapes. **Discriminant** is the hidden tag saying
  which one a given value is.
- **Niche optimisation**, and the **null-pointer optimisation** as its most
  famous case — the compiler using an impossible value as the discriminant,
  which is why `Option<Box<T>>` is the same size as `Box<T>`: null is not a
  valid `Box`, so it can mean `None` for free. `Option<bool>` gets the same
  discount and is one byte, because a `bool` has 254 spare bit patterns.
- **Pattern / arm** — a pattern is a shape the compiler matches a value
  against; an **arm** is one `pattern => expression` line of a `match`.
- **Exhaustiveness** — the compiler's proof that a `match` covers every
  possible value. It is why adding an enum variant turns every place that
  needs updating into a compile error instead of a run-time surprise.
- **Guard** — an `if` condition on a match arm. Guards do *not* count towards
  exhaustiveness, because the compiler cannot evaluate them.
- **Range pattern** (`1..=9`), **alternative** (`a | b`), **wildcard** (`_`),
  **rest** (`..`) — the pattern forms for "in this span", "either of these",
  "anything, unnamed", and "the fields I have not listed".
- **`@` binding** — `n @ 1..=9`: match the pattern *and* keep the value under a
  name.
- **Unreachable arm** — an arm no value can reach because an earlier arm already
  covers it. A warning, not an error, and almost always a bug in arm order.
- **Unwinding** — what a panic does by default: walk back up the stack running
  every destructor on the way, so files close and locks release even as the
  program fails. `panic = "abort"` in a release profile skips all of it.
- **Closure** — a function written inline and passed as a value, `|x| x + 1`.
  It can capture variables from around it, which is what separates it from a
  plain `fn`.
- **Combinator** — a method that transforms a wrapped value without unwrapping
  it: `.map()`, `.and_then()`, `.filter()`, `.unwrap_or_else()`. **Eager**
  versions (`.unwrap_or(x)`) evaluate their argument every time; **lazy** ones
  (`.unwrap_or_else(|| x)`) only when it is needed.
