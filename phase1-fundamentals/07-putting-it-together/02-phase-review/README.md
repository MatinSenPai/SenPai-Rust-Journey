# 1.7.2 — Phase review

## At a glance

After this lesson you can:

- Retell the whole argument of Phase 1 as one chain of "so"s — from heap memory to `&str` — without recalling thirty-one lessons as separate facts.
- Look at a function signature and, in a few seconds, say whether its parameter should be owned or borrowed, and why.
- Find any major Phase 1 error code in the diagnostic table and say exactly which lesson covers it — without searching for it.
- List what you still cannot do — `HashMap`, closures and iterators, traits you define, generics, lifetimes, `Box`/`Rc`/`Arc`, threads and async — and say exactly where in Phase 2 each one is waiting.

**Time:** ~120 minutes · **Prerequisites:** [1.7.1 — A guided mini-project](../01-guided-mini-project/README.md) and all thirty-one lessons of [Phase 1](../../README.md)

---

## Why this matters

You've read thirty-one lessons. You built a program with a guide holding your hand. The question isn't "did you learn something new" — of course you did. The question is: of everything you read, **how much is actually yours, and how much is only familiar?**

The difference is large. Something "familiar" you recognize when you see it again — like a face in a crowd. Something that's "yours" you reach for on a blank page, without anyone saying its name. Phase 2 assumes ownership, borrowing, `Option` and `Result` are already yours — because everything from here on builds on top of them: generics sit on top of ownership, `Rc`/`Arc` sit on the exact same question of "who is responsible for freeing this," `async` sits on the same borrowing Phase 1 already taught, just with a scheduling layer on top.

This lesson doesn't have the usual shape, and that's on purpose. Nothing in it is new. Its job is to say honestly which ideas landed, to build a reference for the day you see an error code and have forgotten its name, and to say plainly — before you walk into Phase 2 — what you don't yet know. Hearing that list stated plainly is more useful than the illusion that you're ready.

---

## The concept

This section doesn't teach anything new. It's the map of what the previous thirty-one lessons built, step by step — so you can see they were one argument, not thirty-one scattered facts.

```senpai-visual
{"kind":"roadmap","labels":["one owner","move","borrowing","the aliasing rule","slices","absence over exceptions"]}
```

### Why no byte is ever freed twice

Rust runs with no garbage collector. That means no background process sweeps through periodically deciding what's no longer needed. Memory on the heap has to be freed **exactly once**: less than that, a memory leak; more than that, a classic bug called a "double free" that crashes the program or opens a security hole in C and C++. [1.2.1](../../02-ownership-and-memory/01-stack-and-heap/README.md) showed this with a real `String`: pointer, length, capacity — and a buffer on the heap that someone has to free.

### So every value has exactly one owner

If freeing has to happen exactly once, exactly one party has to be responsible for it. Rust named that party **ownership**: every value on the heap has exactly one variable responsible for freeing it, and that responsibility fires the moment it goes out of scope. That single rule — no guard, no runtime checker — makes the entire "double free" class of bug unwritable in Rust.

### So assignment means move

If two variables could both own one buffer, both would try to free it — the exact same problem, in one line of code. Rust's answer: `let b = a;` **moves** ownership from `a` to `b`, and `a` is no longer valid. [1.2.2](../../02-ownership-and-memory/02-move-semantics/README.md) showed this; `E0382` is the compiler's voice when you touch `a` after the move.

### So a function that takes a value swallows it

Ownership transfer reaches function boundaries too. `fn f(x: String)` means the caller hands `x` over for good — not a loan, a handover. [1.2.4](../../02-ownership-and-memory/04-ownership-across-functions/README.md) drew that line clearly: `fn f(x: String)` versus `fn f(x: &str)`, two completely different contracts with the caller.

And when taking ownership is expensive, or you genuinely need two independent copies, `.clone()` is the escape hatch — but a *visible* one. [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md) put a number on its cost: cloning a `Vec<String>` of `n` elements is `n + 1` allocations. [1.2.5](../../02-ownership-and-memory/05-drop-and-raii/README.md) showed exactly when this "automatic cleanup" runs — with `Drop` and RAII, in the exact reverse order values were built.

### So you need a way to use without taking

If every time you only wanted to *read* a value you had to own it, any function that computes a string's length would take that string from its caller and never give it back. That's unusable. The answer: **borrowing** — `&T` and `&mut T` — temporary, checked access to something you don't own. [1.3.1](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md) opened this with `&`, `*`, and the choice between `T`, `&T` and `&mut T`.

### So there's one rule for borrows: aliased or mutable, never both

Borrowing on its own doesn't buy safety — if a read-only borrow and a write borrow of the same thing were alive at once, the exact bug class ownership just ruled out would sneak back in through the side door: reading somewhere while something else is changing it. **The aliasing rule**: any number of read-only borrows, or exactly one write borrow — never both, at the same time. [1.3.2](../../03-borrowing-and-references/02-borrow-checker-rules/README.md) stated the rule; `E0502` and `E0499` are the compiler's voice when it's broken. [1.3.3](../../03-borrowing-and-references/03-borrow-scopes-and-nll/README.md) showed exactly where a borrow ends — at its **last use**, not at the closing brace — which is why some seemingly suspicious code compiles fine.

### So a slice is only a look, not ownership

If you only need part of a `Vec` or a string, you don't want to own the whole thing — and copying it would be wasteful. A **slice**, `&[T]`, gives exactly that borrow for *part* of memory: two words, start and length, with nothing new allocated. [1.3.4](../../03-borrowing-and-references/04-slices/README.md) showed this with `&v[1..4]`, and why `&[i32]` instead of `&Vec<i32>` in a signature accepts both an array and a `Vec`.

### So `&str` is a look at text — and text means UTF-8 bytes

`&str` isn't anything new — it's the same slice, over the bytes of some text — with one condition: it has to start and end exactly on a character boundary, or you'd produce invalid text. [1.4.1](../../04-text-and-strings/01-string-vs-str/README.md) opened the relationship between `String` and `&str`; [1.4.2](../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md) showed why `"سلام".len()` is 8, not 4 (every Persian letter is two bytes), and [1.4.4](../../04-text-and-strings/04-slicing-text-safely/README.md) showed that a bad cut panics right there — exactly what you'll see again in "Errors you will meet."

### And the same question, on your own types

Ownership and borrowing were never only about `String` and `Vec` — they were about *any* value. **Building your own type** added nothing new; it just carried the exact same question — "who is responsible for this memory, and who is only looking at it?" — onto data you designed yourself. [1.5.1](../../05-your-own-types/01-structs-and-methods/README.md) showed this with `&self`/`&mut self`/`self` on a `struct` — three method signatures, three answers to the same recurring question. [1.5.3](../../05-your-own-types/03-enums-as-data/README.md) and [1.5.4](../../05-your-own-types/04-match-in-depth/README.md) added one tool that hadn't been needed until then: when a value can genuinely take several different shapes, `enum` models each one separately, and `match` forces the compiler to make you answer for all of them — not just the ones you remembered.

### And, separately: absence and failure are values, not exceptions

This second half of the phase comes from a different family — not from ownership, from **type honesty**. In most languages, "this value doesn't exist" either happens through a silent `null` that can hide anywhere, or through an exception that breaks the program's normal path. Rust has neither. Instead: **`Option<T>`** puts "there might be nothing here" directly into the type, and the compiler forces you to decide before you can use the value — [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md) and [1.6.2](../../06-absence-and-failure/02-option-combinators/README.md). **`Result<T, E>`** does the same for failure, with a reason attached — [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md), where `?` plugs in too. [1.6.4](../../06-absence-and-failure/04-panic-vs-result/README.md) drew the line: `panic!` for a broken contract (a bug), `Result` for a failure the caller can decide about. And [1.6.5](../../06-absence-and-failure/05-from-and-error-conversion/README.md) showed what `?` actually does: it calls `From::from` on the error before it returns early.

### One idea, not thirty-one

Read that chain in one breath and one thing becomes clear: every "so" follows from the one before it. None of it was Rust's designers picking arbitrarily; each step was the forced answer to the last one. If one of these links feels vague, the problem isn't that link — it's that the link before it hasn't settled yet. Go back there, not here.

### The decisions Phase 1 kept asking for

Seven questions Phase 1 asked over and over, and the answer it gave each time:

| Decision | Rule | Reference lesson |
|---|---|---|
| Parameter: owned or borrowed? | Only reading → `&T`. Consuming or keeping → `T`. | 1.2.4 · 1.3.1 |
| `String` or `&str`? | A parameter is almost always `&str`. Something you build and own is `String`. | 1.4.1 |
| `match` or `if let`? | Several arms matter → `match`. Only one arm matters → `if let`. | 1.5.4 · 1.5.5 |
| `Option` or a sentinel? | Absence is possible → always `Option`. Never let `-1`, `""` or `0` mean "not there." | 1.6.1 |
| `Result` or `panic!`? | A failure the caller can/should handle → `Result`. A broken internal contract → `panic!`. | 1.6.4 |
| `Vec` or an array? | The length is fixed at compile time → an array. The length changes at runtime → `Vec`. | 1.1.3 · 1.1.6 |
| Clone or borrow? | Only looking → borrow. Two genuinely independent copies needed → clone, once, deliberately. | 1.2.3 · 1.3.1 |

---

## Hands on

```sh
cargo run -p p1-07-02-phase-review --example 01-the-whole-toolkit
```

```text
there is still an open ticket
last 2 tickets: [Closed { resolution: "duplicate" }, Open]
closed: resolved in review
closed: duplicate
closed: resolved in review
parsed: 4
could not parse: not a ticket count: "۴"
```

Four functions, four Phase 1 decisions, in one file: `close_all` takes ownership of the `Vec` because it hands back the final version ([1.2.4](../../02-ownership-and-memory/04-ownership-across-functions/README.md)); `most_recent` only returns a slice, no allocation ([1.3.4](../../03-borrowing-and-references/04-slices/README.md)); `first_open` returns `None` instead of a made-up ticket ([1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md)); and `parse_ticket_count` reports failure with `Result`, not a panic ([1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md)). The last line of output is also a small reminder: Persian-Eastern digits (`۴`) are not valid input to `.parse::<u32>()` — only ASCII digits are.

Now the broken version — behind a feature, because it genuinely panics:

```sh
cargo run -p p1-07-02-phase-review --example 02-broken-review --features broken
```

We open this one up fully in "Repair." For now just run it and see where it stops.

Then try:

1. In `01-the-whole-toolkit`, raise the number of tickets to ten and call `most_recent` with `count` equal to 5. Guess the output, then check.
2. Call `parse_ticket_count("")` (a completely empty string). What does the `Err` message say?
3. In `02-broken-review`, before fixing anything, comment out just the last line of `main` and rerun it. Which of the five mistakes still show a trace of themselves, and which stay completely silent?

---

## Errors you will meet

Unlike the rest of the phase, this section doesn't open with a single error — it closes with a **diagnostic table**: every major error code you genuinely met across this phase, on one page.

First, the window `02-broken-review.rs` opens when it runs:

```text
thread 'main' (15816) panicked at phase1-fundamentals\07-putting-it-together\02-phase-review\examples\02-broken-review.rs:23:10:
end byte index 3 is not a char boundary; it is inside 'ل' (bytes 2..4 of string)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses next to `'main'` is a thread id and changes every run; the rest of the message is stable.)

**What the compiler is objecting to:** the code that produced this wrote `&word[0..3]` — "from byte zero to byte three." On `"سلام"`, every letter is two bytes, so byte 3 lands right in the middle of the second letter («ل»). If Rust allowed it, the result would be a `&str` that isn't valid UTF-8 — and then it wouldn't really be a `&str`.

**The fix:** instead of guessing at bytes, ask `.char_indices()` where the boundaries actually are — exactly what `shorten` in `src/lib.rs` has to do.

**Why that's the fix:** the error itself names which byte, and *inside which letter* — the two things you need to fix it. The whole story is [1.4.2](../../04-text-and-strings/02-utf8-bytes-chars-graphemes/README.md), and it showed up again here because the most important trap in working with Persian text is still worth relearning.

### Diagnostic table

Every row here was actually run, and its message copied verbatim. Each row links to the lesson that first showed you that error.

| Code | The compiler's actual message | What it really means | Lesson |
|---|---|---|---|
| `E0384` | "cannot assign twice to immutable variable" | A variable without `mut` got assigned a second time. | [1.1.1](../../01-foundations/01-variables-mutability-shadowing/README.md) |
| `E0308` | "mismatched types" | The type of what you gave doesn't match what was expected. The most common code of the whole phase. | [1.1.1](../../01-foundations/01-variables-mutability-shadowing/README.md) |
| `E0382` | "borrow of moved value" | You touched a variable after its value moved; it's no longer valid. | [1.2.2](../../02-ownership-and-memory/02-move-semantics/README.md) |
| `E0507` | "cannot move out of index of `Vec<...>`" | You tried to take ownership of an element out of a `Vec` while the rest of it is still needed. | [1.2.2](../../02-ownership-and-memory/02-move-semantics/README.md) |
| `E0499` | "cannot borrow ... as mutable more than once at a time" | Two mutable borrows of the same thing, alive at once. | [1.3.2](../../03-borrowing-and-references/02-borrow-checker-rules/README.md) |
| `E0502` | "cannot borrow ... as mutable because it is also borrowed as immutable" | The aliasing rule broke: a read borrow was still alive when you tried to take a write borrow too. | [1.3.2](../../03-borrowing-and-references/02-borrow-checker-rules/README.md) |
| `E0596` | "cannot borrow ... as mutable, as it is not declared as mutable" | The owner itself wasn't `mut`, so no `mut` borrow can come from it either. | [1.3.1](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md) |
| `E0106` | "missing lifetime specifier" | A function wants to return a reference and the compiler can't guess which input it's tied to. | [1.2.4](../../02-ownership-and-memory/04-ownership-across-functions/README.md) |
| `E0515` | "cannot return reference to local variable" | A function wants to return a reference to a local value that dies when the function ends. | [1.4.1](../../04-text-and-strings/01-string-vs-str/README.md) |
| `E0004` | "non-exhaustive patterns" | A `match` doesn't cover every shape of the enum. | [1.5.4](../../05-your-own-types/04-match-in-depth/README.md) |
| `E0005` | "refutable pattern in local binding" | A plain `let` used a pattern that might not match (like `Some(x)`). | [1.5.5](../../05-your-own-types/05-if-let-while-let-let-else/README.md) |
| `E0277` | "the type `str` cannot be indexed by `{integer}`" (and its relatives) | The type you're using doesn't have the behavior this operation needs. The phase's second most common code. | [1.1.6](../../01-foundations/06-vec-and-string-basics/README.md) |
| `E0616` | "field ... is private" | Something outside the struct reached for a private field. | [1.5.1](../../05-your-own-types/01-structs-and-methods/README.md) |
| `E0063` | "missing fields ... in initializer" | A struct literal was built without all of its required fields. | [1.5.1](../../05-your-own-types/01-structs-and-methods/README.md) |
| `E0204` | "the trait `Copy` cannot be implemented for this type" | A type with a non-`Copy` field (like `String`) tried to be `Copy`. | [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md) |
| `E0184` | "... the type has a destructor" | A type with `Drop` can't also be `Copy`. | [1.2.5](../../02-ownership-and-memory/05-drop-and-raii/README.md) |
| `E0040` | "explicit use of destructor method" | Something called `.drop()` directly; the free function `drop(x)` is the one to call. | [1.2.5](../../02-ownership-and-memory/05-drop-and-raii/README.md) |

If an error isn't here, that doesn't mean it's unimportant — it means we chose to actually run and precisely record everything that is here, instead of a long, half-verified list.

---

## Exercises

### Warm up

<details>
<summary>Why doesn't <code>let x = 5; x = 6;</code> compile?</summary>

Because `x` was declared without `mut`, and Rust takes immutable assignment seriously. The error code is `E0384`.

</details>

<details>
<summary>What does <code>let b = a;</code> do to <code>a</code> when <code>a</code> is a <code>String</code>? Why does the same line do nothing to <code>a</code> when it's an <code>i32</code>?</summary>

For a `String`, ownership moves from `a` to `b` and `a` is no longer valid. For an `i32`, because it's `Copy`, the assignment makes a full byte copy and there's nothing left to clean up — so both stay alive.

</details>

<details>
<summary>What's the difference between <code>fn f(x: String)</code> and <code>fn f(x: &amp;str)</code> for the caller?</summary>

The first says "hand it over, it's not yours anymore." The second says "just show it to me, it's still yours." The first forces the caller to give up ownership or clone; the second wants neither.

</details>

<details>
<summary>Can a <code>&amp;mut T</code> exist at the same time as another <code>&amp;T</code> of the same value? Why?</summary>

No. The aliasing rule says any number of read borrows, or exactly one write borrow — never both at once. If both were allowed, you could read from one place while something else is changing it.

</details>

<details>
<summary>How many heap allocations does <code>&amp;v[1..3]</code> make?</summary>

None. A slice is just two words — a starting address and a length — looking at `v`'s own buffer. Nothing new is built.

</details>

<details>
<summary>Why is <code>"سلام".len()</code> 8, not 4?</summary>

Because `.len()` counts bytes, not letters. Every Persian letter is two bytes in UTF-8, so four letters comes to eight bytes. `.chars().count()` gives you the letter count.

</details>

<details>
<summary>If an enum has three shapes and a <code>match</code> only has two arms, what does the compiler do?</summary>

It refuses to compile — `E0004`, "non-exhaustive patterns" — and names exactly which shape is missing.

</details>

<details>
<summary>Why is <code>Option&lt;T&gt;</code> better than returning <code>-1</code> to mean "not found"?</summary>

Because the compiler forces you to think about both cases — `Some` and `None` — before you can use the value. `-1` is just an ordinary number; nothing stops it from being used carelessly, and one day it gets summed into a real age or a real balance somewhere.

</details>

<details>
<summary>What does <code>?</code> on a <code>Result&lt;T, E&gt;</code> actually do "instead of" you?</summary>

If the value is `Ok(x)`, it pulls out `x` and keeps going. If it's `Err(e)`, it calls `From::from(e)` and returns from the function right there with `Err` — a whole `match`, in one character.

</details>

<details>
<summary>Why must every <code>Copy</code> type also be <code>Clone</code>, but not the other way around?</summary>

Because `Copy` is a stricter promise: "a byte copy is enough." Any type that can make that promise can naturally also be cloned with that same byte copy. But plenty of types (like `String`) can be cloned — by allocating a new buffer — without being able to make that stricter promise.

</details>

<details>
<summary>Why does Rust have this much machinery around ownership and borrowing instead of a garbage collector? What exact problem do these rules solve?</summary>

Because a garbage collector solves "when do I free this memory" at runtime cost, and with unpredictable timing. Rust wanted the same safety without that cost — so it moved the question to compile time: ownership says exactly when each value gets freed (the moment it leaves its scope), and the aliasing rule prevents simultaneous reading and writing. Neither decision costs anything at runtime.

</details>

<details>
<summary>Why is <code>&amp;str</code> an "unsized type," and why can't you build a plain variable of type <code>str</code> (with no <code>&amp;</code>) directly?</summary>

Because the length of a `str` isn't known at compile time — any piece of text can be any length. The compiler has to know every variable's size without looking at its value. `&str` solves this because a reference always has a fixed size (address plus length), no matter how long the actual text is.

</details>

### Repair

Five classic Phase 1 mistakes sit side by side in `examples/02-broken-review.rs`. The file compiles — only one of the five panics when it runs; the rest are silent and never give themselves away except to a careful reading:

- A parameter that should be the most borrowed shape possible, and isn't.
- A `.clone()` that was never actually needed.
- A number playing the role of `Option`.
- A text slice thinking in bytes, not letters — the one that panicked above.
- A `match` where only one arm actually matters.

Find all five. For each one, write one sentence saying why it's wrong and which Phase 1 lesson covers it. Then fix the code — with `--features broken` it no longer has to run.

### Implement

Six functions in `src/lib.rs`, one per module of Phase 1 (excluding this one):

```sh
cargo test -p p1-07-02-phase-review
```

| Function | Module | Stuck? Go back to |
|---|---|---|
| `grade_letter` | 1.1 Foundations | 1.1.5 — Control flow |
| `merge_unique` | 1.2 Ownership and memory | 1.2.3 and 1.2.4 |
| `interior` | 1.3 Borrowing | 1.3.4 — Slices |
| `shorten` | 1.4 Text | 1.4.2 — UTF-8 |
| `describe_status` | 1.5 Your own types | 1.5.3 and 1.5.4 |
| `safe_average` | 1.6 Absence and failure | 1.6.1 — `Option` |

This table is deliberate: each function needs exactly one idea. If one of them won't come right, this table is exactly what to reread — not "Rust" in general.

### Build

Build a small standalone program — somewhere separate from `src/lib.rs` — that manages a small playlist: a handful of tracks, each with a title and a length. It needs:

- A function that builds the list and owns it.
- A function that only reads the list and searches it — borrowed, not owning.
- An `enum` for each track's status (say, "unplayed" / "played"), and a `match` or `if let` that works on it.
- A function that returns `None` when a track isn't found — not a panic, not a sentinel number.
- One place where a `.clone()` is genuinely needed — with a comment saying why.

Then write one sentence: which decision was hardest, and which Phase 1 lesson answered it?

### Challenge (optional)

This one genuinely steps outside Phase 1 — let's look right now at why Phase 2 exists.

You wrote each of the six "Implement" functions with a hand-rolled loop. Now run this:

```rust
let evens: Vec<u32> = (0..=20).filter(|value| value % 2 == 0).collect();
println!("{evens:?}");
```

```text
[0, 2, 4, 6, 8, 10, 12, 14, 16, 18, 20]
```

One line. No `let mut`. No loop. `filter` and `collect` both come from Phase 2 — module 2.2, iterators and closures — and you haven't formally learned them yet. If this exact code showed up in a Phase 1 lesson, `cargo run -p lesson-lint` should flag it as an early use of a concept; here, in this lesson, it was shown to you on purpose and out loud.

Now try it yourself: rewrite `merge_unique` using a `std::collections::HashSet`, so that instead of `.contains()` (which scans the whole of `merged` for every item) it uses a lookup that's roughly instant. If it doesn't come together, don't worry — that's exactly what module 2.1 (collections) is there to teach you.

---

## Wrapping up

| Term | What it means | Where it was introduced |
|---|---|---|
| ownership | every value has exactly one party responsible for freeing it | 1.2.1 |
| move | ownership transfer; the old name is no longer valid | 1.2.2 |
| borrow | using a value without taking its ownership | 1.3.1 |
| the aliasing rule | any number of read borrows, or exactly one write borrow — never both | 1.3.2 |
| slice | a borrowed look at part of memory someone else owns | 1.3.4 |
| sentinel | an ordinary value pretending to mean "absent" | 1.6.1 |
| diagnostic table | a reference that ties an error code to its meaning and its lesson | this lesson |

### What you now know

- One argument, not thirty-one separate facts: from heap memory to `&str`, each step was the forced answer to the one before it.
- Seven decisions Phase 1 kept asking for, and the rule behind each one.
- Seventeen major error codes from this phase, and exactly what each one is saying.
- That being "familiar" with an idea is different from it being "yours" — and which ones are still only familiar.

### What comes back later

Phase 1 never touched any of these. Each one is a Phase-1-shaped hole in what you know, and it's more useful to know that honestly now than to assume you're ready:

| Concept | What it's for | Where in Phase 2 |
|---|---|---|
| `HashMap` | key-based lookup or counting in roughly constant time, instead of scanning a `Vec` linearly | [Phase 2 — `Vec` and `HashMap` types](../../../phase2-intermediate/01-collections/01-vec-and-hashmap/README.md) |
| Closures and iterators | writing a data transform/filter instead of a hand-rolled loop, and functions that take another function as a parameter | [Phase 2 — closures and the `Fn` traits](../../../phase2-intermediate/02-iterators-and-closures/01-closures-and-fn-traits/README.md) and [iterator adapters](../../../phase2-intermediate/02-iterators-and-closures/02-iterator-adapters/README.md) |
| Traits you define | a shared behavior across several different types, without copying code | [Phase 2 — defining and implementing traits](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md) |
| Generics | a function or struct written once for every type, instead of once per type | [Phase 2 — generic functions and structs](../../../phase2-intermediate/03-generics-and-traits/01-generic-functions-and-structs/README.md) |
| Explicit lifetimes | when the compiler can't itself guess how long a reference stays valid | [Phase 2 — lifetime basics](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md) |
| `Box`, `Rc`, `Arc` | heap ownership through a pointer, and shared ownership when you genuinely need more than one owner | [Phase 2 — `Box` and heap allocation](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md) and [`Rc` and `Arc`](../../../phase2-intermediate/05-smart-pointers/02-rc-and-arc/README.md) |
| Threads and async | running several things at once — one for genuinely using multiple CPU cores, the other for waiting cheaply on I/O | [Phase 2 — threads, `Mutex` and `Arc`](../../../phase2-intermediate/07-concurrency-and-async/01-threads-mutex-arc/README.md) and [futures and runtimes](../../../phase2-intermediate/07-concurrency-and-async/03-futures-and-runtimes/README.md) |

### Can you explain?

- Retell the chain of "so"s from heap memory to `&str`, without looking at this page?
- For a made-up function, say whether its parameter should be `T`, `&T` or `&mut T`, and why?
- Say three error codes from the diagnostic table from memory — code, meaning, and lesson?
- Say why `Option` is better than a sentinel value, without using the word "cleaner"?
- Retell the "what comes back later" list without looking — all seven?

---

## Going further

- [The Rust Book — Chapter 4 (ownership)](https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html) — the same argument, from the Rust team itself.
- [The Rust Book — Chapter 6 (enums and match)](https://doc.rust-lang.org/book/ch06-00-enums.html) — for `Option`, `Result`, and forced exhaustiveness.
- [Rust By Example — Error handling](https://doc.rust-lang.org/rust-by-example/error.html) — the same ground, in shorter examples.
- [The Phase 2 roadmap](../../../phase2-intermediate/README.md) — where you go once this lesson is done.
