# 06.1 — Modules, visibility, workspaces

## `mod`: organizing code into a tree

In Python, every `.py` file is automatically a module — `import foo` works
because `foo.py` exists on disk, full stop. Rust has no such automatic
mapping. A Rust crate is a **tree of modules**, and you build that tree
explicitly with the `mod` keyword:

```rust
mod catalog {
    pub struct Anime {
        pub title: String,
    }
}

let a = catalog::Anime { title: "Frieren".to_string() };
```

`mod catalog { ... }` declares a module named `catalog` right here, inline.
The more common style for anything non-trivial is `mod catalog;` (no body)
in `src/lib.rs`, paired with a separate file `src/catalog.rs` (or
`src/catalog/mod.rs`) that holds the actual contents — Rust looks for the
file by the module's name. Both forms produce the exact same tree; this
lesson uses the inline form so everything stays in one file you can read
top to bottom, but know that splitting into files is what you'll do the
moment a module gets big.

## Visibility: private by default

This is the part that surprises Python developers most: **every item
(struct, field, function, module) is private to the module that defines it,
plus that module's descendants, unless you say otherwise.** Python leans on
convention (a leading underscore *suggests* "internal," but nothing stops
you importing it anyway); Rust enforces privacy as a compile error.

- No modifier — private. Visible only inside the defining module and its
  children.
- `pub` — visible to anyone who can see the module itself, including code
  outside this crate entirely (if the module is reachable, e.g. re-exported
  from the crate root).
- `pub(crate)` — visible anywhere *inside this same crate*, but **not** to
  external crates that depend on this one as a library. This is extremely
  common in real code: it's how you say "other parts of my own codebase can
  use this, but it's an implementation detail, not part of my public API."

```rust
mod catalog {
    pub struct Anime {
        pub title: String,           // public field
        pub(crate) internal_rating: u8, // crate-visible, not public API
    }
}
```

A caller in a *different* crate that depends on this one can construct an
`Anime` and read `title`, but cannot even name `internal_rating` — the
compiler reports it as a private field, exactly as if it had no `pub` at
all. Only code inside *this* crate (including this crate's own test code,
which compiles as part of the crate for unit tests — more on that
distinction in the next lesson) can see it.

## `use` and re-exporting with `pub use`

`use` brings a path into scope so you don't have to write it out in full
every time:

```rust
use catalog::Anime;
let a = Anime { title: "Frieren".to_string() };
```

`pub use` does something extra: it **re-exports** an item at a new,
shallower path. Say `Anime` really lives at `catalog::series::Anime` three
modules deep — that's an awkward path for callers to type. At the crate
root you can write:

```rust
pub use catalog::series::Anime;
```

Now external callers write `my_crate::Anime` directly, even though nothing
about where `Anime` is *actually defined* changed. This is how real crates
present a clean, flat public API on top of a deeply-nested internal module
structure — you'll see this pattern constantly in libraries you depend on.

## Workspaces (conceptual — no code here)

A **Cargo workspace** is a group of crates that share one `Cargo.lock` and
one `target/` build output directory, so a shared dependency compiles once
instead of once per crate. This entire repo is one workspace (see the root
`Cargo.toml`): every lesson crate you build is a workspace member, so
running `cargo build` anywhere in the repo doesn't recompile `tokio` from
scratch for the 40th lesson that uses it. You won't create a nested
workspace in this lesson — just recognize the term, since you're already
living inside one.

## Your task

`src/lib.rs` defines a small `catalog` module simulating an anime catalog,
with a public `Anime` type re-exported at the crate root, and a crate-only
"internal rating" hidden behind `pub(crate)`. Fill in the two `todo!()`s.

## Next

`solution/SOLUTION.md` — but only after a real attempt.
