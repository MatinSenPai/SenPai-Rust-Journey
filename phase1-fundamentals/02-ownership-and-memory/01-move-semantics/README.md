# 02.1 — Move semantics

## The rule

**Every value has exactly one owner.** When that owner goes out of scope,
Rust cleans the value up automatically (next lesson: `Drop`). There's no
garbage collector deciding *when* to free memory, and no manual `free()`
either — ownership is tracked entirely at compile time, and cleanup happens
deterministically, the instant the owner's scope ends.

The consequence that trips up every Python developer first:

```rust
let s1 = String::from("hello");
let s2 = s1;              // ownership of the String MOVES from s1 to s2
println!("{s1}");         // compile error: borrow of moved value: `s1`
```

In Python, `s2 = s1` makes `s2` a second name pointing at the *same* object
— both remain fully usable. In Rust, `let s2 = s1;` **moves** ownership:
`s1` is no longer valid at all after that line. This isn't a copy of the
string's contents (that would be wasteful for a large `String`) — it's a
transfer of responsibility, and the compiler statically forbids using the
old name afterward, specifically so two variables can never both think they
own (and later both try to clean up) the same heap allocation.

Why does this only bite you with `String` and not with, say, `i32`?
`let x = 5; let y = x; println!("{x}");` compiles fine — that's the next
lesson (`Copy` types).

## Moves happen at function boundaries too

```rust
fn takes_ownership(s: String) {
    println!("{s}");
} // s is dropped here, at the end of this function

let s = String::from("hello");
takes_ownership(s);
// s is no longer valid here — it moved into the function
```

Passing `s` into `takes_ownership` moves it, exactly like `let s2 = s1`
does. This is *why* borrowing (`&s`, next module) exists at all: constantly
losing access to a value just because you passed it to a function would be
unworkable. For this lesson, though, work *with* moves rather than around
them — notice the patterns below where a function deliberately takes and
then **gives back** ownership.

## Your task

Implement the three functions in `src/lib.rs`. Also: uncomment the block
marked `// UNCOMMENT ME` at the bottom of the file, run `cargo check -p
p1-02-01-move-semantics`, read the compiler error carefully, then
re-comment it before running `cargo test` (a file that fails to compile
can't be tested).

## Next

`solution/SOLUTION.md` — but only after a real attempt.
