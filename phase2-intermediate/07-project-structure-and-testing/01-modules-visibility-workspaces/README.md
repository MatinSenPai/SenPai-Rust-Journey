# 2.7.1 — Modules, visibility, re-exports, workspaces

## At a glance

After this lesson you can:

- Explain how Rust's module tree maps onto the file tree — `mod foo;`
  paired with `foo.rs`, or a nested submodule paired with `foo/bar.rs` —
  and split a file yourself the same way once it grows.
- Choose correctly among no modifier, `pub`, `pub(crate)`, and
  `pub(super)` for a given item, and read and fix a genuine private-access
  error, `E0616`, on your own.
- Use `pub use` to re-export an item at a shallower path than where it's
  actually defined, and point at this repository's own root `Cargo.toml`
  as a real, inspectable Cargo workspace.

**Time:** ~65 minutes · **Prerequisites:**
[1.5.1 — Structs and methods](../../../phase1-fundamentals/05-your-own-types/01-structs-and-methods/README.md)

---

## Why this matters

From Phase 0 until right now, every time you ran `cargo run` or
`cargo test`, exactly one file got compiled. `src/lib.rs`, or one of the
`examples/*.rs` files — always one file, top to bottom. For a short
exercise, that's great. For a real project, it isn't — and that's exactly
where this lesson picks up.

Python solves this for you automatically: every `.py` file is a module
just by existing, and `import foo` works because `foo.py` is on disk —
nothing else to declare. Rust doesn't work that way. A Rust crate is a
**module tree**, and you build that tree yourself, explicitly, with the
`mod` keyword. No file is a module on its own until some `mod` names it.

And the moment a project splits into several files, a question shows up
that had no meaning before today: out of everything you just wrote —
functions, fields, the modules themselves — which of them is the rest of
your code (or somebody else's code, if this crate is used as a library)
actually allowed to see? Rust doesn't guess: **everything is private by
default.** There's no unwritten convention like Python's leading
underscore, which only *suggests* "internal" without stopping anyone from
importing it anyway — here, privacy is a compile error, not a naming hint.

This is exactly the same boundary
[2.3.6](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.md)
showed you once already, through a completely different door: the orphan
rule said `impl Trait for Type` is only legal when the trait or the type
is defined in your own crate — a crate boundary, so that two unrelated
crates can never collide. Today you meet that same crate boundary again,
this time for a much more ordinary question: is this code even allowed to
read this field?

This lesson gives you four things: how to spread a module across files,
the four visibility levels Rust actually has, `pub use` for presenting a
clean API on top of a messy internal layout, and workspaces — which,
right now, from this lesson's very first line, you've been living inside
one.

---

## The concept

### `mod`: building a tree

Every real Rust crate — not just this repository, any crate — has its
code living inside a module tree. `mod` builds that tree, and you can
nest as many layers deep as you like:

```rust
mod catalog {
    pub struct Anime {
        pub title: String,
    }

    pub mod series {
        pub struct Volume {
            pub number: u32,
        }
    }
}

let a = catalog::Anime { title: "Frieren".to_string() };
let v = catalog::series::Volume { number: 3 };
println!("{} — volume {}", a.title, v.number);
```

```text
Frieren — volume 3
```

`mod catalog { ... }` declares a module named `catalog`, right where
it's written — this is called the **inline** form. Reaching something
inside it goes through a double colon (`::`): `catalog::Anime`, or even
two levels down, `catalog::series::Volume`. Every new `mod` adds one more
segment to that path — exactly the way nested folders add one more
segment to a file path.

The inline form is fine for a short lesson — everything stays in one file
you can read top to bottom. It isn't fine for a real project; a few
paragraphs down you'll see why, and how you break that same tree apart.

### Visibility: private by default, `pub`, and `pub(crate)`

This is the part that surprises Python developers most: **every item —
struct, field, function, even the module itself — is visible, by
default, only to the module that defines it and that module's
descendants, unless you say otherwise.**

- **No modifier** — private. Only inside that same module, and any
  module nested inside it.
- **`pub`** — visible to anyone who can see the module itself, including
  code entirely outside this crate (as long as the path itself is
  reachable too).
- **`pub(crate)`** — visible from anywhere *inside this same crate*, but
  **not** to an external crate depending on this one as a library. This
  level is extremely common in real code: it's exactly how you say "the
  rest of my own codebase can use this, but it's an implementation
  detail, not part of my public API."

```rust
mod catalog {
    pub struct Anime {
        pub title: String,
        pub(crate) internal_rating: u8,
    }
    // ...
}

mod front_desk {
    use crate::catalog::Anime;

    pub fn describe(a: &Anime) -> String {
        format!("{} (internal rating {}/10)", a.title, a.internal_rating)
    }
}
```

`catalog` and `front_desk` are siblings — neither is nested inside the
other. `front_desk` can read `title` because it's `pub`. But it can also
read `internal_rating`, even though `front_desk` isn't `catalog`'s child
at all! This is exactly what separates `pub(crate)` from ordinary
private: "anywhere in this crate," not "this module and its children."
Run it:

```sh
cargo run -p p2-07-01-modules-visibility-workspaces --example 01-inline-modules-and-visibility
```

```text
Frieren: Beyond Journey's End (internal rating 10/10)
```

And if `internal_rating` had no modifier at all — fully private — that
same `front_desk` line wouldn't compile; you'll see the full error in
"Errors you will meet". For an *external* crate depending on this one as
a library, the story is different: it could still read `title`, but it
couldn't even name `internal_rating` — the external crate's own compiler
never generates any code referencing that field, because from its point
of view the field doesn't exist at all. There's no run-time check to
bypass; everything is settled before the program ever runs, at compile
time.

### `pub(super)`: exactly one level up

There's a fourth level, for when you want something exactly *one step*
more open than ordinary private — not as open as the whole crate:

```rust
mod catalog {
    pub struct Anime {
        pub title: String,
    }

    pub mod series {
        use super::Anime;
        pub(super) fn shelf_code(a: &Anime) -> String {
            format!("SR-{}", a.title.len())
        }
    }
    pub fn shelf_label(a: &Anime) -> String {
        format!("{}: {}", a.title, series::shelf_code(a))
    }
}
```

`series` is nested inside `catalog`. `shelf_code` inside it is
`pub(super)` — meaning "visible to my immediate parent module,
`catalog`, and anywhere `catalog` itself is visible from; not one step
further." `catalog::shelf_label` is that exact parent, so it can call
`series::shelf_code`:

```sh
cargo run -p p2-07-01-modules-visibility-workspaces --example 02-pub-super-nested-visibility
```

```text
Mushishi: SR-8
```

But crate-root code — one level further out than `catalog` — still
cannot call `catalog::series::shelf_code`, even though `series` itself is
`pub` and its path is perfectly nameable. The path resolves; the function
itself doesn't. `examples/02-pub-super-nested-visibility.rs` has exactly
this line, commented out, at the bottom of `main` — uncomment it and see
for yourself which item the compiler calls private.

```senpai-visual
{"kind":"concept","labels":["private: this module + children","pub(super): one level up","pub(crate): the whole crate","pub: outside the crate too"]}
```

Four levels, each one stacked on the last: every one sees exactly what
the level before it saw, plus a little more.

### Splitting into real files: `mod foo;` beside `foo.rs`

Now back to this section's opening promise. `mod foo { ... }` writes
everything right there. But you can drop the body entirely — just
`mod foo;` — and Rust goes looking for its contents itself:

```text
src/
├── lib.rs        # mod catalog;
├── catalog.rs    # pub mod series;
└── catalog/
    └── series.rs
```

The rule is simple: when the root file (`lib.rs`, or `main.rs`, or
whatever file the crate — or its example — actually starts from) writes
`mod catalog;`, Rust looks for `catalog.rs` right beside it. And if
`catalog.rs` itself writes `mod series;`, Rust this time looks for
`catalog/series.rs`: a subdirectory named after the parent module, next
to that same file.

```senpai-visual
{"kind":"concept","labels":["mod catalog; in lib.rs","looks for catalog.rs","mod series; in catalog.rs","looks for catalog/series.rs"]}
```

`examples/03-splitting-into-files/` builds the exact same
`catalog`/`series` tree you just saw, but this time across three real
files — `main.rs`, `catalog.rs`, and `catalog/series.rs`. Open them and
read them together; then run it:

```sh
cargo run -p p2-07-01-modules-visibility-workspaces --example 03-splitting-into-files
```

```text
Mushishi: SR-8
```

The exact same output as `02`. That's not a coincidence — it's the whole
point. The inline form and the multi-file form build exactly the same
tree; only the layout on disk changed, not the meaning of the code. The
only time you actually need a separate file is once a module has grown
too big to fit comfortably on one screen — and that day, this is exactly
what you do about it.

### `use` and re-exporting with `pub use`

`use` brings a path into the current scope, so you don't have to write
it out in full every time:

```rust
mod catalog {
    pub mod series {
        pub struct Anime {
            pub title: String,
        }
    }
}

mod front_desk {
    use crate::catalog::series::Anime;

    pub fn label(a: &Anime) -> String {
        format!("now showing: {}", a.title)
    }
}
```

Here `Anime` really lives three layers down:
`catalog::series::Anime`. The `use` that `front_desk` wrote is just a
shortcut — it exports nothing, it only saves typing inside this one
module.

`pub use` does one extra thing: it **re-exports** that item at a new,
shallower path:

```rust
pub use catalog::series::Anime;
```

Now anyone — from inside this same crate, or from an external crate that
depends on it — can write `Anime` directly (or, from outside,
`this_crate::Anime`), with no idea it's actually defined three layers
down. Run it:

```sh
cargo run -p p2-07-01-modules-visibility-workspaces --example 04-use-and-pub-use-reexport
```

```text
now showing: Frieren
```

The important part: those two paths — the re-exported `Anime` and the
original `catalog::series::Anime` — build one `Anime`, not two.
`pub use` never copies anything; it just opens a second door into the
same room. `examples/04` proves exactly this: one `Anime` is built
through the re-exported path, then handed to a function that imported
the internal path instead — and it compiles, because both name the exact
same type.

This is precisely the pattern real crates use to present one clean, flat
API on top of a deep, nested internal module tree: if you split
`catalog::series` into two further submodules tomorrow, you change that
one `pub use` line — no caller who reached `Anime` through the short path
has to change a thing.

### Workspaces: this repository, a real example

A **Cargo workspace** is a group of crates that share one `Cargo.lock`
and one `target/` build output directory — a shared dependency compiles
once between them, not once per crate.

You don't have to build one to see how it works — you're inside one of
the largest examples you'll find, right now. This repository's root has
a `Cargo.toml` that declares itself a workspace:

```toml
[workspace]
resolver = "2"
members = [
    "web-ui",
    "tools/lesson-lint",
    "phase0-setup/03-hello-rust",
    # ...
    "phase1-fundamentals/*/*",
    "phase2-intermediate/*/*",
    # ...
]
```

Every entry in `members` is its own separate crate — this very lesson,
`p2-07-01-modules-visibility-workspaces`, is one of them, matched
through the `"phase2-intermediate/*/*"` pattern. Dozens of small crates,
one `Cargo.lock`, one `target/`. That's why compiling a lesson that
needs `tokio` doesn't rebuild `tokio` from scratch — it reuses the
compile from whichever lesson needed it first.

One genuinely strange, entirely real fact straight from that same file:
the comment above that list says a member glob matching zero directories
is a hard error from Cargo itself, not a silent no-op — meaning the
`"phase2-intermediate/*/*"` pattern couldn't be added until the day a
real Phase 2 lesson actually existed on disk.

One last note this very lesson leans on: every lesson's `solution/` is
also a crate, but deliberately *not* a member of this workspace — it
carries its own empty `[workspace]` table (you can see it in this
lesson's own `solution/Cargo.toml`) so Cargo treats it as a fully
separate root, not part of this one.

### A reminder: the same crate boundary as 2.3.6

The crate boundary is showing up for the second time today.
[2.3.6](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.md)
taught you the orphan rule: you may implement a trait for a type only if
the trait or the type is defined in your own crate — a crate boundary,
on *implementing traits*. `pub(crate)` is that same boundary, on *seeing
items*: not a new rule, the same one line — "my crate versus everyone
else" — put to work on a completely different question.

---

## Hands on

```sh
cargo run -p p2-07-01-modules-visibility-workspaces --example 01-inline-modules-and-visibility
cargo run -p p2-07-01-modules-visibility-workspaces --example 02-pub-super-nested-visibility
cargo run -p p2-07-01-modules-visibility-workspaces --example 03-splitting-into-files
cargo run -p p2-07-01-modules-visibility-workspaces --example 04-use-and-pub-use-reexport
```

Then the broken one:

```sh
cargo run -p p2-07-01-modules-visibility-workspaces --example 05-private-field-is-inaccessible-broken --features broken
```

Then try these:

1. In `examples/02-pub-super-nested-visibility.rs`, uncomment the last
   commented line in `main`. Which item does the compiler call private —
   `series` itself, or `shelf_code`?
2. In `examples/03-splitting-into-files/`, open all three files and
   compare the tree to `examples/02-pub-super-nested-visibility.rs` — is
   it the same tree?
3. In `examples/04-use-and-pub-use-reexport.rs`, add a
   `println!("{}", a.title);` directly in `main`, before the call to
   `front_desk::label`. Does it compile? Why?

---

## Errors you will meet

### `E0616` — a private field, reached from outside its module

```text
error[E0616]: field `internal_rating` of struct `Anime` is private
  --> phase2-intermediate\07-project-structure-and-testing\01-modules-visibility-workspaces\examples\05-private-field-is-inaccessible-broken.rs:28:22
   |
28 |     println!("{}", a.internal_rating);
   |                      ^^^^^^^^^^^^^^^ private field

For more information about this error, try `rustc --explain E0616`.
```

**What the compiler is actually complaining about:** `internal_rating` in
this version has no modifier at all — fully private. Private visibility
means "only the module that defines it, and that module's descendants."
`main`, in this same file, is written outside `catalog` — not `catalog`
itself, not its child. So even though `main` is inside this same crate,
it can't name `internal_rating` at all.

**The fix:** one of two ways — either turn it back into `pub(crate)`,
exactly what `01-inline-modules-and-visibility.rs` already has:

```rust
pub(crate) internal_rating: u8,
```

or, if you don't want any other code anywhere in this crate reading the
raw number directly either, add a public read-only method instead:

```rust
pub fn internal_rating(&self) -> u8 {
    self.internal_rating
}
```

**Why this is the fix:** the first way answers exactly this lesson's
question — `main` is inside this same crate, so `pub(crate)` is enough
for it to see the field, without opening the field to external crates
too. The second way is one step more conservative: the field stays fully
private, so no code — not even inside this same crate — can ever write
something like `a.internal_rating = 99` directly anymore; it can only
read, through the method. Which one is right depends on how much you
want to protect that number from direct tampering; either one moves past
the full-private state that produced this exact error.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
mod a {
    fn secret() -> i32 {
        7
    }
}

mod b {
    fn try_it() {
        println!("{}", crate::a::secret());
    }
}
```

</details>

<details>
<summary>Answer</summary>

No — `E0603`, "function `secret` is private". `secret` has no modifier,
so it's only visible inside `a` (and `a`'s descendants). `b` is a sibling
of `a`, not its child.

</details>

<details>
<summary>With the same two modules, if you make <code>secret</code> <code>pub(crate)</code>, can <code>b</code> call it then?</summary>

</details>

<details>
<summary>Answer</summary>

Yes. `pub(crate)` means "anywhere in this crate" — and `b`, just like
`a`, is inside this same crate. The sibling relationship between `a` and
`b` stops mattering.

</details>

<details>
<summary>If <code>catalog</code> itself has no <code>pub</code> in front of <code>mod catalog</code>, can <code>main</code> — written in the same file, at the crate root — still call <code>catalog::hello()</code> (assuming <code>hello</code> itself is <code>pub</code>)?</summary>

</details>

<details>
<summary>Answer</summary>

Yes. `catalog` being private means "visible only to the module that
defines it, and that module's descendants" — and that module is the
crate root itself, the same place `main` is written. `catalog` really is
hidden from outside the crate; from inside this same crate, it isn't.

</details>

<details>
<summary>After <code>pub use catalog::series::Anime;</code>, does <code>catalog::series::Anime</code> still work, or only the short path?</summary>

</details>

<details>
<summary>Answer</summary>

Both work. `pub use` *adds* a new path; it doesn't remove the original
one. One definition, now reachable through two doors.

</details>

### Repair

Fix `examples/05-private-field-is-inaccessible-broken.rs` **two** ways:

1. By turning `internal_rating` back into `pub(crate)`.
2. Without changing the field's visibility — instead add a public method
   `internal_rating(&self) -> u8`, and change `main` to call that method
   instead of reading the field directly.

Then write one sentence: which one do you see more often in a real
library, and why?

### Implement

Three functions in `src/lib.rs`:

```sh
cargo test -p p2-07-01-modules-visibility-workspaces
```

`Anime::public_rating_band` does what you saw above — a coarse band from
the hidden `internal_rating`. `pricing::discount_percent` is a
crate-internal discount policy, `pub(super)` rather than `pub`, because
it's `catalog`'s business, not the whole crate's. `Anime::rental_price_cents`
connects the two — the only function that actually calls the
`pub(super)` one from outside `pricing`. Implement all three exactly to
the format stated in each one's own doc comment above it.

### Build

Add a second submodule of your own to `catalog` — any name, any idea
(reviews, tags, studio, whatever you like). Give it at least one `pub`
item and one item that's deliberately not `pub`. Then, above each one, in
a doc comment, write one sentence explaining why you picked that
visibility level.

### Challenge (optional)

**Part one.** Open this repository's own root
[`Cargo.toml`](../../../Cargo.toml). Find the `"phase2-intermediate/*/*"`
line inside `members`, and the comment a few lines below it explaining
why that pattern couldn't have been added from the very start, before
the first real Phase 2 lesson actually existed on disk. State that rule
in one sentence.

**Part two.** (This one looks ahead, and knows it.) Build a scratch
experiment, just for yourself: a `tests/` directory next to this lesson's
own `src/`, with one `#[test]` function that does
`use p2_07_01_modules_visibility_workspaces::Anime;` and builds one. It
compiles. Now change that same function to reach for the internal
`catalog` path instead of `Anime`. It doesn't compile — even though you
just watched, a few minutes ago, this same crate's own `main` reach that
exact path directly! (Delete the file afterward; nothing here needs to
be committed.) That exact, strange contrast is the entire subject of the
next lesson:
[2.7.2](../02-unit-integration-doc-tests/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Module (`mod`) | the unit of code organization; inline (`mod foo { ... }`) or file-based (`mod foo;`) | any project that outgrows one file |
| Private by default | visible only to the defining module and its children | the starting state for every item |
| `pub` | visible everywhere the module itself is visible, including outside the crate | a public API |
| `pub(crate)` | visible anywhere in this crate, not outside it | a shared implementation detail |
| `pub(super)` | visible only to the immediate parent module | something only the parent module needs to know |
| `use` | shortcutting a path, without exporting anything | shorter typing inside the same module |
| Re-export (`pub use`) | presenting an item at a shallower path, without moving it | a clean public API over a messy internal layout |
| Workspace | several crates, one `Cargo.lock`, one `target/` | the crate you're inside right now |
| `E0616` | a private field, read from outside its module | make it `pub(crate)`, or write a reader method |

### What you now know

- `mod` builds a module tree; the inline form and the file form
  (`mod foo;` beside `foo.rs`) build exactly the same tree.
- Visibility has four steps: private by default (this module + children),
  `pub(super)` (+ one level up), `pub(crate)` (+ the whole crate), `pub`
  (+ outside the crate too).
- A private item defined at the crate root is visible from anywhere in
  *that same crate* — privacy closes off the crate boundary, not every
  place inside it.
- `pub use` exports an item at a new path without breaking the original
  path or building a second type.
- A workspace keeps several crates on one shared `Cargo.lock` and one
  `target/`; this repository, from the very first line of its own root,
  is one of them.

### What comes back later

- **Unit tests versus integration tests — exactly why they rely on this
  same visibility boundary** —
  [2.7.2 — Unit, integration, and doc tests](../02-unit-integration-doc-tests/README.md)

### Can you explain?

- Why does `pub(crate)` separate itself from ordinary private, when both
  items live in that same one crate?
- Exactly how much visibility does `pub(super)` add — and why is "one
  level" more precise than "everywhere further up"?
- Explain `mod foo;`, without looking at this lesson, to someone who
  doesn't yet know where Rust goes looking for its file.
- What does `pub use` move, and what does it not move?
- Why is this repository's root a workspace, and what would be slower if
  it weren't?
- What one idea do 2.3.6's orphan rule and today's `pub(crate)` both lean
  on?

---

## Going further

- [The Rust Book — Managing Growing Projects with Packages, Crates, and
  Modules](https://doc.rust-lang.org/book/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html) —
  the same ground, official, with more detail on the file-resolution
  rules.
- [The Rust Reference — Visibility and
  privacy](https://doc.rust-lang.org/reference/visibility-and-privacy.html) —
  the precise, official definition of all four visibility levels.
- [Cargo's documentation —
  Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html) —
  the same concept this repository's root is built on.
- [This repository's own root `Cargo.toml`](../../../Cargo.toml) — a
  real workspace, dozens of members, inspectable right now.
