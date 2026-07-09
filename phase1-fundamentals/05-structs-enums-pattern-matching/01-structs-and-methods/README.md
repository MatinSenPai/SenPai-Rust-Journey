# 05.1 — Structs and methods

## Defining a struct

```rust
struct Anime {
    title: String,
    episodes: u32,
    is_completed: bool,
}

let one_piece = Anime {
    title: String::from("One Piece"),
    episodes: 1100,
    is_completed: false,
};
```

A struct groups named, typed fields together — the closest Python analogue
is a `@dataclass`, except every field's type is fixed and checked at
compile time, and (as you already know) `one_piece` is immutable by default
unless declared `let mut`.

## Methods: `impl` blocks

```rust
impl Anime {
    // Associated function (no `self`) — called as Anime::new(...), acts
    // like a constructor. Not special syntax, just a convention.
    fn new(title: &str, episodes: u32) -> Self {
        Anime { title: title.to_string(), episodes, is_completed: false }
    }

    // Method (`&self`) — borrows the instance, reads it.
    fn describe(&self) -> String {
        format!("{} ({} episodes)", self.title, self.episodes)
    }

    // Method (`&mut self`) — borrows the instance mutably, can modify it.
    fn mark_completed(&mut self) {
        self.is_completed = true;
    }
}

let mut op = Anime::new("One Piece", 1100);
println!("{}", op.describe());
op.mark_completed();
```

`Self` (capital S) inside an `impl` block means "this same type" —
equivalent to writing `Anime` again, but it stays correct if you ever rename
the type. `title, episodes` in the struct-literal shorthand above is
"field init shorthand": when a variable's name matches the field name
exactly, you can skip writing `title: title`.

`&self` vs `&mut self` vs `self` (by value, consuming) is the same
shared/mutable/owned distinction from the borrowing module, just applied to
"the instance a method was called on." You'll use `self` (by value) in
Phase 2 for consuming builder-style methods.

## Your task

`src/lib.rs` defines a `Book` struct (episodes/manga-chapter tracking, in
the spirit of this repo's interests). Implement its methods.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
