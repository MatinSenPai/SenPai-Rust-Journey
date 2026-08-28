# 2.6.4 — `Weak` and reference cycles

## At a glance

After this lesson you can:

- Build two Rc-holding structures that each hold the other with a strong reference, and explain precisely why neither one's count ever reaches zero.
- State what safe Rust actually guarantees and what it doesn't — and back that distinction with a real memory leak you built by hand.
- Use `Weak<T>` and `.upgrade()`, and read the `Option<Rc<T>>` it returns as an honest answer to a real question, not an obstacle.
- Build a parent/child tree where the parent holds its children strongly and each child holds only a weak reference back, and explain why that direction — not the reverse — is what breaks the cycle.

**Time:** ~50 minutes · **Prerequisites:** [2.6.3 — `Rc` and `Arc`](../03-rc-and-arc/README.md)

---

## Why this matters

Last lesson made you a nice promise: with `Rc`, a value stays alive as long as at least one strong owner holds it, and cleans itself up the moment the last owner lets go. `Rc::clone` is cheap, the count goes up; an `Rc` gets dropped, the count comes down; it hits zero, the value is freed. That promise is true — but it carries an unstated assumption this lesson puts a finger on: it assumes the count *eventually* reaches zero.

Here's the bad news: Rust gives you no guarantee that it will. If two values each hold a strong `Rc` to the other, neither one's count ever reaches zero — not because your code has a bug, not because something panicked, but because the shape of the data itself has made it impossible. That is a genuine memory leak: those values sit on the heap forever, and `Drop::drop` never runs for either of them, for as long as the program keeps running.

This needs to be said plainly, because saying it only halfway is worse than not saying it at all: Rust shuts out data races and use-after-free with the full force of its type system — those two bug classes are genuinely unwritable in safe Rust. But memory leaks were never part of that promise. This lesson shows you exactly where that line falls, and — because the problem is real — the tool built specifically for it: `Weak<T>`.

---

## The concept

### The cycle that never frees itself

Say you have two `Friend`s, and each one wants to hold on to its best friend. The obvious first move: each holds a strong `Rc` to the other.

```rust
struct Friend {
    name: String,
    best_friend: RefCell<Option<Rc<Friend>>>,
}

let alice = Rc::new(Friend { name: "Alice".to_string(), best_friend: RefCell::new(None) });
let bob = Rc::new(Friend { name: "Bob".to_string(), best_friend: RefCell::new(None) });

*alice.best_friend.borrow_mut() = Some(Rc::clone(&bob));
*bob.best_friend.borrow_mut() = Some(Rc::clone(&alice));

println!("alice strong_count: {}", Rc::strong_count(&alice));
println!("bob strong_count:   {}", Rc::strong_count(&bob));
```

```text
alice strong_count: 2
bob strong_count:   2
```

The `best_friend` field is wrapped in a `RefCell` so it can be set *after* `alice` and `bob` already exist behind an `Rc` — without it there is no way for `alice` to point at `bob` while `bob` points at `alice` at the same time. `RefCell` is [2.6.5](../05-refcell-and-interior-mutability/README.md)'s subject; for now this much is enough: `.borrow_mut()` means "temporarily unlock this field for writing," even though you only hold a shared reference.

Each count went from 1 to 2: one for its own local binding (`alice` or `bob`), one for the copy the other one is holding. Now drop both local bindings:

```rust
drop(alice);
drop(bob);
```

```text
both local bindings dropped — no "dropping: ..." line printed above
```

Neither one dropped. Each `Friend` already had an `impl Drop` that printed its own name — and that message never printed, for either Alice or Bob. `drop(alice)` only destroys the local `alice` binding; the count goes from 2 down to 1, because the copy inside `bob.best_friend` is still alive. `drop(bob)` does the exact same thing to `bob`'s count. Both values now sit on the heap, each kept alive by exactly one strong reference — from inside the other — with no named variable anywhere in the program pointing at either of them. A two-node cycle, completely cut off from the rest of the program, that is never cleaned up for as long as the program runs.

```senpai-visual
{"kind":"ownership","labels":["Alice holds Bob — strong","Bob holds Alice — strong","both counts: 2","drop both bindings","both counts: 1 — never 0"]}
```

### `Weak<T>` — a handle that takes no ownership

What if one side of that pair simply didn't count toward the strong total? That's exactly what `Weak<T>` is: a handle to the same value whose existence does not keep the value alive and has no effect on `Rc::strong_count`. You build one with `Rc::downgrade(&rc)`:

```rust
let strong = Rc::new(String::from("shared"));
let weak: Weak<String> = Rc::downgrade(&strong);

println!("strong_count: {}", Rc::strong_count(&strong));
println!("weak_count:   {}", Rc::weak_count(&strong));
```

```text
strong_count: 1
weak_count:   1
```

The strong count is still 1 — only a separate counter, the weak count, went up. But now you have a problem: how do you actually reach the `String` from a `Weak<String>`? There's no direct access — and that's deliberate. A `Weak` can never promise its target is still there; the last owner might have dropped it a moment ago. The only way to reach the value is `.upgrade()`, which returns an `Option<Rc<T>>` — `Some` if at least one strong owner is still alive, `None` if every one of them is gone:

```rust
match weak.upgrade() {
    Some(value) => println!("upgrade while alive: got {value:?}"),
    None => println!("upgrade while alive: got nothing"),
}
```

```text
upgrade while alive: got "shared"
```

Now drop the strong owner and ask the exact same question again:

```rust
drop(strong);

match weak.upgrade() {
    Some(value) => println!("upgrade after drop:  got {value:?}"),
    None => println!("upgrade after drop:  got nothing"),
}
```

```text
upgrade after drop:  got nothing
```

There is nothing mysterious here — this is the exact same `Option<T>` you already know from
[1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md),
just with `T` being `Rc<T>` this time. And for the exact same reason that `Option` was never
arbitrary there — a lookup that might not find anything shouldn't pretend it always does — an
`Option<Rc<T>>` here is honesty about a certainty `Weak` fundamentally cannot offer. `weak_count`
goes down the same way `strong_count` does, just under a different rule: dropping a `Weak` lowers
the weak count, but leaves the underlying value untouched — as long as the strong count hasn't
already reached zero.

### The standard fix: strong down, weak up

Back to that cycle. Alice and Bob's problem wasn't that each held the other — it was that *both* sides of the relationship were strong. A relationship that genuinely has a natural direction — a parent/child tree — makes this obvious: a parent *owns* its children (if the parent goes, the children should go too); a child does not own its parent (a child only needs to be able to *find* its parent, not keep it alive). So the downward reference (parent to child) stays strong; the reference back up (child to parent) becomes `Weak`.

```rust
struct Folder {
    name: String,
    parent: Weak<Folder>,
    children: Vec<Rc<Folder>>,
}

let root = Rc::new_cyclic(|weak_root| Folder {
    name: "root".to_string(),
    parent: Weak::new(),
    children: vec![
        Rc::new(Folder { name: "docs".to_string(), parent: weak_root.clone(), children: vec![] }),
        Rc::new(Folder { name: "src".to_string(), parent: weak_root.clone(), children: vec![] }),
    ],
});
```

This time, no `RefCell` was needed at all. `Rc::new_cyclic` is built for exactly this shape: before `root` exists as a real `Rc`, it hands its closure a `Weak<Folder>` that will point at that same `root` once construction finishes. Each child, at the moment it's created, gets a `weak_root.clone()` — a working `Weak` back to a parent that, at this very instant, isn't fully built yet.

```rust
for child in &root.children {
    match child.parent.upgrade() {
        Some(parent) => println!("{}'s parent is {}", child.name, parent.name),
        None => println!("{}'s parent is gone", child.name),
    }
}
println!("root strong_count: {}", Rc::strong_count(&root));
```

```text
docs's parent is root
src's parent is root
root strong_count: 1
```

Even with two children pointing at `root`, its strong count is still 1 — because neither of those pointers is strong. Now drop `root`:

```rust
drop(root);
```

```text
dropping: root
dropping: docs
dropping: src
```

Everything cleaned up this time, in the natural order: `root`'s only strong owner was the local `root` binding itself; once it went away, `root`'s count hit zero and `Folder::drop` ran. That `drop` in turn frees its own `children` field — which means the two strong copies of `docs` and `src` go too, each of their counts hits zero, and their `Folder::drop` runs as well. A cascade, exactly what you'd expect from an ordinary tree under ordinary ownership — not because anyone was careful, but because the direction of the references no longer forms a cycle.

```senpai-visual
{"kind":"ownership","labels":["root holds children — strong","children hold root — weak","root count: 1","drop root","children drop too — count 0"]}
```

---

## Hands on

```sh
cargo run -p p2-06-04-weak-and-reference-cycles --example 01-the-cycle-problem
cargo run -p p2-06-04-weak-and-reference-cycles --example 02-weak-and-upgrade
cargo run -p p2-06-04-weak-and-reference-cycles --example 03-the-fix-parent-child-tree
```

Then the three broken ones:

```sh
cargo run -p p2-06-04-weak-and-reference-cycles --example 04-upgrade-is-not-the-value --features broken
cargo run -p p2-06-04-weak-and-reference-cycles --example 05-weak-has-no-direct-methods --features broken
cargo run -p p2-06-04-weak-and-reference-cycles --example 06-cannot-mutate-through-shared-rc --features broken
```

Then try these:

1. In `01-the-cycle-problem`, print `Rc::strong_count(&alice)` right before the two `borrow_mut` lines. What is it, and why does it differ from afterward?
2. In `02-weak-and-upgrade`, add `let _second = Rc::clone(&strong);` before `drop(strong)`. Now what does `weak.upgrade()` return after `drop(strong)`? Why?
3. In `03-the-fix-parent-child-tree`, add a third child named `"tests"` to `root` and run it again. How does the order of the `dropping: ...` lines change?

---

## Errors you will meet

### `E0308` — `.upgrade()` never hands back the value itself

```text
error[E0308]: mismatched types
  --> phase2-intermediate\06-smart-pointers\04-weak-and-reference-cycles\examples\04-upgrade-is-not-the-value.rs:26:16
   |
26 |     print_name(&maybe_folder);
   |     ---------- ^^^^^^^^^^^^^ expected `&Rc<Folder>`, found `&Option<Rc<Folder>>`
   |     |
   |     arguments to this function are incorrect
   |
   = note: expected reference `&Rc<_>`
              found reference `&Option<Rc<_>>`
note: function defined here
  --> phase2-intermediate\06-smart-pointers\04-weak-and-reference-cycles\examples\04-upgrade-is-not-the-value.rs:15:4
   |
15 | fn print_name(folder: &Rc<Folder>) {
   |    ^^^^^^^^^^ -------------------
```

**What the compiler is objecting to:** `weak.upgrade()` returned an `Option<Rc<Folder>>`, not an `Rc<Folder>`. The code above handed that `Option` straight to a function that wants a `&Rc<Folder>` — two different types, and the compiler never blurs the two together on its own.

**The fix:** answer both cases of the `Option` first, then act:

```rust
match weak.upgrade() {
    Some(folder) => print_name(&folder),
    None => println!("(gone)"),
}
```

**Why this is the fix:** `.upgrade()` cannot promise the value is still there — exactly the promise a `Weak` fundamentally cannot make — so its return type says that, and nothing else. The only way to reach the `Rc<Folder>` inside it is to answer both cases.

### `E0599` — `Weak<T>` has none of `T`'s own methods

```text
error[E0599]: no method named `shout` found for struct `std::rc::Weak<T, A>` in the current scope
  --> phase2-intermediate\06-smart-pointers\04-weak-and-reference-cycles\examples\05-weak-has-no-direct-methods.rs:27:25
   |
27 |     println!("{}", weak.shout());
   |                         ^^^^^ method not found in `std::rc::Weak<Folder>`
```

**What the compiler is objecting to:** `Folder` has a `shout` method, but `weak` has type `Weak<Folder>` — not `Folder`, and not even `Rc<Folder>`. Unlike `Rc<T>`, `Weak<T>` does not implement `Deref` — there is no auto-deref down to `T`, because the compiler cannot prove ahead of time that `T` is even still there.

**The fix:** `.upgrade()` first, then call the method on the result:

```rust
if let Some(folder) = weak.upgrade() {
    println!("{}", folder.shout());
}
```

**Why this is the fix:** `.upgrade()` is exactly the one step that turns a "maybe" into a real `Rc<Folder>` — the only thing `Deref` is defined on. Until you take that step, none of `Folder`'s methods are reachable.

### `E0594` — you cannot write through a shared `Rc`

```text
error[E0594]: cannot assign to data in an `Rc`
  --> phase2-intermediate\06-smart-pointers\04-weak-and-reference-cycles\examples\06-cannot-mutate-through-shared-rc.rs:26:5
   |
26 |     docs.parent = Rc::downgrade(&root);
   |     ^^^^^^^^^^^ cannot assign
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Folder>`
```

**What the compiler is objecting to:** `docs` has type `Rc<Folder>`. `Rc<T>` implements `Deref` (which is why `docs.parent` works for reading) but not `DerefMut` — you never get a `&mut T` back out of an `Rc`, no matter how many owners happen to be alive right now.

**The fix:** either put a field that needs writing later inside a `RefCell` from the start — exactly what `01-the-cycle-problem.rs` did — or, when you can, build everything that depends on each other in one shot with `Rc::new_cyclic`, like `03-the-fix-parent-child-tree.rs`, so there's nothing left to change afterward.

**Why this is the fix:** this is the exact wall `Weak` and `Rc::new_cyclic` each go around, in their own way — one by allowing writes through a shared reference (`RefCell`), the other by building everything before you'd ever hit this wall. The compiler is right here: there is no way for two simultaneous owners to both hold a `&mut`, without coordination — the same aliasing rule you already know from
[1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md).

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
let a = Rc::new(5);
let b = Rc::clone(&a);
let weak = Rc::downgrade(&a);
drop(a);
drop(b);
println!("{:?}", weak.upgrade());
```

</details>

<details>
<summary>Answer</summary>

```text
None
```

`weak` was never counted in the strong total. `a` and `b` were both strong owners; once both were dropped, the strong count hit zero and the value was freed — so there's nothing left for `.upgrade()` to reach.

</details>

<details>
<summary>And this one?</summary>

```rust
let rc = Rc::new(10);
let weak = Rc::downgrade(&rc);
println!("{:?}", weak.upgrade());
```

</details>

<details>
<summary>Answer</summary>

```text
Some(10)
```

`rc` is still alive — nothing has been dropped. `.upgrade()` returns a fresh (and cheap) `Rc<i32>` to the same value; the strong count is briefly 2, until that fresh `Rc` itself gets dropped.

</details>

<details>
<summary>Why does creating a <code>Weak&lt;T&gt;</code> never change the strong count?</summary>

Because a `Weak` takes no ownership — it makes no promise to keep the value alive. The strong count is exactly what decides when the value gets freed; if a `Weak` counted toward it, you could never use one to break a cycle, because it would keep the value alive just as much as an `Rc` would.

</details>

<details>
<summary>In the Alice/Bob cycle, after both local bindings are dropped, does <code>Drop::drop</code> run for either of them?</summary>

Neither. Each still holds one strong reference — from inside the other; each count goes from 2 to 1, not to zero. For as long as the program keeps running, both sit on the heap, with no named variable anywhere reaching either of them.

</details>

### Repair

Fix all three broken examples:

1. Fix `examples/04-upgrade-is-not-the-value.rs` so it compiles — with a `match` or `if let` that actually answers both cases of `.upgrade()`'s `Option`, not an unjustified `.unwrap()`.
2. Fix `examples/05-weak-has-no-direct-methods.rs` so `shout()` actually gets called, while `Folder` is still alive.
3. Fix `examples/06-cannot-mutate-through-shared-rc.rs` **two** ways: once by putting `parent` inside a `RefCell` (like `01-the-cycle-problem.rs`), once by building the whole tree in one shot with `Rc::new_cyclic` (like `03-the-fix-parent-child-tree.rs`) instead of changing something afterward.

### Implement

Four functions in `src/lib.rs`:

```sh
cargo test -p p2-06-04-weak-and-reference-cycles
```

None of them needs `RefCell` or building a fresh tree — the sample tree already exists; your job is only to read it.

### Build

Write a `pub fn full_path(folder: &Rc<Folder>) -> String` that returns the path from the root down to `folder`, in a format you choose (something like `"root/docs"` is reasonable) — and write that exact format in your own doc comment.

### Challenge (optional)

**Part one.** Build a three-level tree — a `root` with a `docs`, and `docs` with its own `guide.md` — by nesting `Rc::new_cyclic` calls (any level with children of its own needs to be built with `Rc::new_cyclic` too, so it can hand out a working `Weak` to its own children). Then call `depth()` on `guide.md` — what does it return?

**Part two.** (This one looks ahead.) This lesson's tree stops growing once it's built — you can't add a new child to `root` afterward, because `children` is a plain `Vec<Rc<Folder>>`, not something you can write through a shared reference. Guess: in
[2.6.5](../05-refcell-and-interior-mutability/README.md), which field or fields of `Folder` would need to change type to make it possible to add a child after the tree is built?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Weak<T>` | A non-owning handle to a value managed by `Rc`/`Arc` | The back-reference in any two-way structure |
| `Rc::downgrade(&rc)` | Builds a `Weak<T>` from an `Rc<T>` | Anywhere you only need to *find* something, not keep it alive |
| `.upgrade()` | The only way to reach the value from a `Weak`; gives `Option<Rc<T>>` | Every time you actually need the value |
| Reference cycle | Values holding *strong* references to each other in a loop | Exactly what this lesson showed you how to build, and how not to |
| `Rc::new_cyclic` | A constructor that hands its closure a `Weak<T>` to the value before construction finishes | Building a tree/graph in one shot, with no `RefCell` needed |

### What you now know

- Safe Rust shuts out data races and use-after-free, but not every memory leak — two `Rc`s that strongly point at each other never have a count that reaches zero.
- `Weak<T>` takes no ownership and doesn't count toward `Rc::strong_count`; the only way to reach the value, `.upgrade()`, gives an `Option<Rc<T>>`, because it can never promise the value is still there.
- The standard fix for a parent/child tree: the downward reference (parent to child) stays strong; the reference back up (child to parent) becomes `Weak` — because a tree's natural ownership already runs in that same direction.
- `Rc::new_cyclic` lets you build a tree in one shot, so every child has a working `Weak` back to its parent from the moment it's created — no `RefCell` needed.
- `Rc<T>` implements `Deref` but not `DerefMut`; writing through a shared `Rc` is always refused, no matter how many owners happen to be alive right now.

### What comes back later

- **`RefCell` and interior mutability — how you actually write through a shared reference** — [2.6.5](../05-refcell-and-interior-mutability/README.md)

### Can you explain?

- Why do two `Rc`s that mutually hold each other strongly never get dropped?
- Why doesn't holding a `Weak<T>` change the strong count?
- Why does `.upgrade()` return `Option<Rc<T>>`, not a bare `Rc<T>`?
- In a parent/child tree, why does the strong reference point down and the weak one point up, not the other way around?
- What problem does `Rc::new_cyclic` solve that an ordinary `Rc::new` can't?

---

## Going further

- [The Rust Book — Reference Cycles Can Leak Memory](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html) — this exact lesson, from the official source.
- [`std::rc::Weak`](https://doc.rust-lang.org/std/rc/struct.Weak.html) — its full method list.
- [`Rc::new_cyclic`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.new_cyclic) — the official docs, with another example of the same pattern.
