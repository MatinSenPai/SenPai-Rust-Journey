# 1.6.1 — `Option` and null safety

## At a glance

After this lesson you can:

- Say what happens when you try to use an `Option<T>` as if it were guaranteed to hold a value — and why Rust catches it at compile time, where C, Java and Python would let it through to crash later.
- Choose between `match`, `if let`, `.unwrap()` and `.expect("...")` for a given `Option<T>`, and defend the choice out loud.
- Tell `Option<&T>` and `&Option<T>` apart on sight, and reach for `.as_ref()` to get from the second to the first without a clone.
- Read a `size_of::<Option<Box<T>>>()` next to a `size_of::<Box<T>>()` and say precisely why they match.

**Time:** ~55 minutes · **Prerequisites:** [1.5.5 — `if let`, `while let`, `let else`](../../05-your-own-types/05-if-let-while-let-let-else/README.md)

---

## Why this matters

Since Phase 0, a standard function has been handing you something wrapped in `Some` or `None`, and every single time the lesson has said "the full explanation is later":

- [1.1.2](../../01-foundations/02-scalar-types-and-overflow/README.md) — `checked_add` returned `None` when the answer didn't fit.
- [1.1.3](../../01-foundations/03-compound-types-and-destructuring/README.md) and [1.1.6](../../01-foundations/06-vec-and-string-basics/README.md) — `.get()` on an array and on a `Vec` returned `Option` instead of panicking.
- The same [1.1.6](../../01-foundations/06-vec-and-string-basics/README.md) — `.pop()` and `.chars().nth()` did it too.
- [1.3.4](../../03-borrowing-and-references/04-slices/README.md) — `.first()` on a slice, the same wrapper.
- Even [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md)'s errors section ran into it: cloning `lines.first()` cloned the wrapper, not the string inside it.
- And [1.1.5](../../01-foundations/05-control-flow/README.md) ended with a confession: `index_of_first_negative` returned the array's length to mean "not found" — a plain number a caller could index with, unaware, and panic on — and said this lesson would fix it.

Time to collect. This lesson pays off every one of those debts at once.

The problem `Option` exists to solve has a name, because one specific person is responsible for it: Tony Hoare, who invented the "null reference" in ALGOL W in 1965. Years later, speaking at QCon London in 2009, he put it this way:

> "I call it my billion dollar mistake. It was the invention of the null reference in 1965. ... I couldn't resist the temptation to put in a null reference, simply because it was so easy to implement. This has led to innumerable errors, vulnerabilities, and system crashes, which have probably caused a billion dollars of pain and damage in the last forty years."
>
> — Tony Hoare, QCon London 2009

What that meant in practice: a variable of any type — a `User`, a number, a database connection — could quietly be `null`, and the language drew no distinction between "this is definitely a `User`" and "this might be nothing at all." The compiler couldn't help, because as far as types went, they were the same thing. The result was an entire class of bug — `NullPointerException` in Java, `'NoneType' object has no attribute` in Python, `Cannot read property of null` in JavaScript — findable only by testing, or by luck, never by the compiler up front.

Rust puts the answer in the type itself. If a function might not find something, its signature says exactly that, and nothing else:

```rust
fn find_user(id: u32) -> Option<String>
```

Not `-> String` with a hidden `null` lurking inside it. `Option<String>` is a type that differs from `String` — as much as `i32` differs from `bool` — and that one sentence is the whole lesson: **because the type is different, the compiler will not let you use a `None` as if it were a value, unless you deal with the possibility first.** No exceptions, no forgotten check, no finding out in production six months from now.

---

## The concept

### `Option<T>` — an ordinary enum with two variants

```rust
let recorded: Option<u32> = Some(25);
let missing: Option<u32> = None;

println!("recorded: {recorded:?}");
println!("missing:  {missing:?}");
```

```text
recorded: Some(25)
missing:  None
```

Nothing new in the machinery. `Option<T>` is exactly what [1.5.3](../../05-your-own-types/03-enums-as-data/README.md) revealed: an ordinary enum, with two variants — one carries a value, one carries nothing:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

That's it. The standard library added no magic; it just used the tools [1.5](../../05-your-own-types/README.md) already gave you. And because it's an ordinary enum, you can ask what it is without opening it:

```rust
println!("recorded.is_some(): {}", recorded.is_some());
println!("missing.is_none():  {}", missing.is_none());
```

```text
recorded.is_some(): true
missing.is_none():  true
```

### A `None` cannot sneak in behind your back

Write this — a function that might not find anything:

```rust
fn find_user(id: u32) -> Option<String> {
    if id == 1 {
        Some(String::from("Matin"))
    } else {
        None
    }
}
```

Now try using that `Option<String>` as if it were guaranteed to be a `String` — hand it straight to a function that wants one:

```rust
fn greet(name: String) -> String {
    format!("hello, {name}")
}
```

This is exactly the bug the null reference makes possible in most languages: a lookup that might not have found anything, handed straight to code that assumed it did. Java, C# and Python compile this and crash the first time `id` doesn't match. Rust refuses to even build it — the full error is in "Errors you will meet".

**Get the point precisely:** the problem was never that `find_user` might not find something — functions like that always exist. The problem, in most other languages, was that this possibility was never written into the function's signature. Here it is, and the compiler reads it.

### Getting the value out: `match` and `if let`

The only way to reach what's inside an `Option<T>` is to answer both cases — exactly what [1.5.4](../../05-your-own-types/04-match-in-depth/README.md) and [1.5.5](../../05-your-own-types/05-if-let-while-let-let-else/README.md) already gave you, now aimed at `Option`:

```rust
fn describe(rating: Option<u8>) -> String {
    match rating {
        Some(score) => format!("rated {score}/10"),
        None => "not rated yet".to_string(),
    }
}
```

```rust
println!("{}", describe(Some(9)));
println!("{}", describe(None));
```

```text
rated 9/10
not rated yet
```

When only one case has work to do, `if let` is that same `match` with the "do nothing" arm left out:

```rust
let maybe_name: Option<&str> = Some("Matin");
if let Some(name) = maybe_name {
    println!("if let:   hello, {name}");
}
```

```text
if let:   hello, Matin
```

```senpai-visual
{"kind":"concept","labels":["fn call","Some(v)","None","match","use T safely"]}
```

Keep making the trade [1.5.5](../../05-your-own-types/05-if-let-while-let-let-else/README.md) told you to make on purpose: `match` guarantees you've looked at both cases; `if let` gives that guarantee up. For "is there something to show, or not" that's a fine trade — the other arm really is "do nothing".

### `.unwrap()` and `.expect("...")` — and when either is defensible

Both take the value out of a `Some` and **panic** on `None` — the program stops immediately. Neither is wrong, and neither is always right; they're the correct tool exactly when you can see something the type checker can't:

```rust
let mut scores = Vec::new();
scores.push(10);
scores.push(20);
scores.push(30);
let last = scores.last().expect("scores just had three items pushed");
println!("expect on a Vec you just filled: {last}");
```

```text
expect on a Vec you just filled: 30
```

`.last()` returns `Option<&T>` because a `Vec` might be empty — but this one just had three items pushed onto it, on the line directly above. `.expect("...")` writes that visual proof down, for whoever reads the code next — quite possibly you, six months from now.

Get it wrong, though, and the two tell you very different amounts. `.unwrap()` only ever says:

```text
called `Option::unwrap()` on a `None` value
```

While `.expect("user 7 should exist in the seed data")` says:

```text
user 7 should exist in the seed data
```

The first tells you *what* failed. The second tells you *why you thought it couldn't*. Both full panics — with file and line — are in "Errors you will meet".

> **The rule:** `.unwrap()`/`.expect()` are for a `None` the *type* allows but the *surrounding code* rules out. And if you're ruling it out, write down why in the `.expect()` message — the way we just did above.

### `Option<&T>` versus `&Option<T>`, and `.as_ref()`

These are built from the same two words but they are not the same type. The first is "an `Option` that, if it has anything, has a reference"; the second is "a reference to the whole box, `Some` or `None` and all."

```rust
fn describe_ref(nickname: &Option<String>) -> String {
    match nickname {
        Some(name) => format!("&Option<T>:    Some(\"{name}\")"),
        None => "&Option<T>:    None".to_string(),
    }
}
```

```rust
let known = Some(String::from("Matin"));
println!("{}", describe_ref(&known));
```

```text
&Option<T>:    Some("Matin")
```

Now the other spelling. `.as_ref()` turns the `Option<String>` you own into an `Option<&String>` you can look at — without taking it away from you:

```rust
fn describe_inner(nickname: Option<&String>) -> String {
    match nickname {
        Some(name) => format!("Option<&T>:    Some(\"{name}\")"),
        None => "Option<&T>:    None".to_string(),
    }
}
```

```rust
println!("{}", describe_inner(known.as_ref()));
println!("still have `known`: {known:?}");
```

```text
Option<&T>:    Some("Matin")
still have `known`: Some("Matin")
```

`known` is still yours afterward. Write `match known { ... }` directly instead of going through `.as_ref()`, and `known` would **move** — exactly what you'll see in "Errors you will meet".

And there's a way back: `.cloned()` turns an `Option<&T>` into an owned `Option<T>` — the same `.cloned()` [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md) promised you, this time working on what's inside the `Option` rather than on the `Option` itself:

```rust
let words = vec![String::from("hello"), String::from("world")];
let borrowed: Option<&String> = words.first();
let owned: Option<String> = borrowed.cloned();
println!("borrowed: {borrowed:?}");
println!("owned:    {owned:?}");
```

```text
borrowed: Some("hello")
owned:    Some("hello")
```

### `Option` in a struct field means "optional"

When a field might not have a value, its type is `Option` — not an empty string, not a zero standing in for nothing:

```rust
struct Profile {
    nickname: Option<String>,
}

impl Profile {
    fn nickname_len(&self) -> Option<usize> {
        match self.nickname.as_ref() {
            Some(name) => Some(name.len()),
            None => None,
        }
    }
}
```

`self.nickname.as_ref()` dodges the same trap here: without it, `match self.nickname { ... }` would try to move the field out of `self` — and you cannot own something out of a reference (`&self`).

```rust
let matin = Profile {
    nickname: Some(String::from("Matin")),
};
let anon = Profile { nickname: None };

println!("matin.nickname_len(): {:?}", matin.nickname_len());
println!("anon.nickname_len():  {:?}", anon.nickname_len());
```

```text
matin.nickname_len(): Some(5)
anon.nickname_len():  None
```

No sentinel string like `""` for "never set", no hand-rolled convention to remember. The type itself says the field is optional.

### The null-pointer optimisation

`Option<T>` usually needs room for `T` plus a tag — "is this `Some` or `None`?":

```rust
use std::mem::size_of;

println!("size_of::<i32>():           {}", size_of::<i32>());
println!("size_of::<Option<i32>>():   {}", size_of::<Option<i32>>());
```

```text
size_of::<i32>():           4
size_of::<Option<i32>>():   8
```

Every possible byte pattern of an `i32` is a real number, so there's no spare pattern free for "nothing" — `Option` has to take extra room. A real pointer is a different story: a valid `&T` or `Box<T>` is never all-zero-bits, so `None` can borrow that exact bit pattern for free:

```rust
println!("size_of::<Box<i32>>():         {}", size_of::<Box<i32>>());
println!("size_of::<Option<Box<i32>>>(): {}", size_of::<Option<Box<i32>>>());
```

```text
size_of::<Box<i32>>():         8
size_of::<Option<Box<i32>>>(): 8
```

(`Box<T>` is a heap pointer, owning what it points at — that one line is all you need today; its full lesson is in [Phase 2](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md).)

This trick isn't only for pointers — any type with unused bit patterns gets the same discount. `bool` only uses 2 of a byte's 256 patterns:

```rust
println!("size_of::<bool>():          {}", size_of::<bool>());
println!("size_of::<Option<bool>>():  {}", size_of::<Option<bool>>());
```

```text
size_of::<bool>():          1
size_of::<Option<bool>>():  1
```

**`Option<Box<T>>` costs no bytes over `Box<T>`. `Option<i32>` does.** That's a measurement you just made yourself, not a claim from documentation.

---

## Hands on

```sh
cargo run -p p1-06-01-option-and-null-safety --example 01-some-and-none
cargo run -p p1-06-01-option-and-null-safety --example 02-match-and-if-let
cargo run -p p1-06-01-option-and-null-safety --example 03-defensible-unwrap-and-expect
cargo run -p p1-06-01-option-and-null-safety --example 04-option-ref-and-struct-field
cargo run -p p1-06-01-option-and-null-safety --example 05-null-pointer-optimization
```

Then the four broken ones:

```sh
cargo run -p p1-06-01-option-and-null-safety --example 06-cannot-use-option-directly --features broken
cargo run -p p1-06-01-option-and-null-safety --example 07-unwrap-on-none --features broken
cargo run -p p1-06-01-option-and-null-safety --example 08-expect-on-none --features broken
cargo run -p p1-06-01-option-and-null-safety --example 09-matching-by-value-moves-it --features broken
```

Then try:

1. In `01-some-and-none`, build an `Option<bool>` — `Some(false)` and `None`. What does comparing them with `==` give you?
2. In `04-option-ref-and-struct-field`, call `matin.nickname_len()` three times in a row. Does it still compile? Why is the answer "no" for `describe(nickname)` in file `09`?
3. In `05-null-pointer-optimization`, print `size_of::<Option<char>>()` too and compare it with `size_of::<char>()`.

---

## Errors you will meet

### `E0308` — a lookup that might fail, handed to code that assumed it didn't

```text
error[E0308]: mismatched types
  --> phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\06-cannot-use-option-directly.rs:24:26
   |
24 |     println!("{}", greet(name));
   |                    ----- ^^^^ expected `String`, found `Option<String>`
   |                    |
   |                    arguments to this function are incorrect
   |
   = note: expected struct `String`
                found enum `Option<String>`
note: function defined here
  --> phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\06-cannot-use-option-directly.rs:18:4
   |
18 | fn greet(name: String) -> String {
   |    ^^^^^ ------------
help: consider using `Option::expect` to unwrap the `Option<String>` value, panicking if the value is an `Option::None`
   |
24 |     println!("{}", greet(name.expect("REASON")));
   |                              +++++++++++++++++
```

**What the compiler is objecting to:** `find_user` returns an `Option<String>`, not a `String`. `greet` wants a `String`. These are two different types, and the compiler will not blur them together — not even for convenience.

**The fix:** deal with the possibility of `None` first, then answer:

```rust
match find_user(7) {
    Some(name) => println!("{}", greet(name)),
    None => println!("no such user"),
}
```

**Why that's the fix:** the compiler's own suggested fix (`.expect("REASON")`) is a way out too, but it's more honest to call it by its real name: it's a deliberate `panic!`, not a repair. If a missing user really should stop the program, write `.expect()` with a message saying *why* it should always be found. If not, `match` or `if let` is what you should write instead.

### The `.unwrap()` panic — on a `None` value

```text
thread 'main' (34052) panicked at phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\07-unwrap-on-none.rs:16:29:
called `Option::unwrap()` on a `None` value
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is objecting to:** this isn't even a compiler error — the program built and ran, and panicked exactly where `find_user(7)` genuinely returned `None`. `.unwrap()` type-checks on any `Option<T>`; the compiler has no way to know ahead of time whether the answer will be `Some` or `None`.

**The fix:** answer both cases with `match` or `if let`, or, if you genuinely believe this `None` can't happen, write `.expect("...")` and say why.

**Why that's the fix:** the message only says *what* failed — `Option::unwrap()` on a `None`. It doesn't say which value came up empty or why the surrounding code expected it not to. For that, you want `.expect()` — next.

### The `.expect("...")` panic — and what it adds

```text
thread 'main' (15160) panicked at phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\08-expect-on-none.rs:17:29:
user 7 should exist in the seed data
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

**What the compiler is objecting to:** the same thing — a `None` with `.expect()` called on it. But this time the panic message is exactly the text you gave `.expect()`, not a generic sentence.

**The fix:** the same fix as above — a `match` that genuinely answers both cases.

**Why that's the fix:** compare this message with `07-unwrap-on-none`'s. One says "an `unwrap()` failed, somewhere." The other says "user 7 should exist in the seed data" — precisely which assumption broke. **That is the entire reason `.expect()` exists:** wherever you genuinely believe a `None` can't come up, put the panic message where the reader needs it — in the code itself, not in a comment nobody reads.

### `E0382` — matching an owned `Option` moves it

```text
error[E0382]: use of moved value: `nickname`
  --> phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\09-matching-by-value-moves-it.rs:15:29
   |
13 |     let nickname = Some(String::from("Matin"));
   |         -------- move occurs because `nickname` has type `Option<String>`, which does not implement the `Copy` trait
14 |     println!("{}", describe(nickname));
   |                             -------- value moved here
15 |     println!("{}", describe(nickname));
   |                             ^^^^^^^^ value used here after move
   |
note: consider changing this parameter type in function `describe` to borrow instead if owning the value isn't necessary
  --> phase1-fundamentals\06-absence-and-failure\01-option-and-null-safety\examples\09-matching-by-value-moves-it.rs:5:23
   |
 5 | fn describe(nickname: Option<String>) -> String {
   |    --------           ^^^^^^^^^^^^^^ this parameter takes ownership of the value
   |    |
   |    in this function
help: consider cloning the value if the performance cost is acceptable
   |
14 |     println!("{}", describe(nickname.clone()));
   |                                     ++++++++
```

**What the compiler is objecting to:** `describe` takes ownership of an `Option<String>` (`nickname: Option<String>`, not `&Option<String>`). Calling it the first time moves `nickname`. `Option<String>`, just like `String` itself, isn't `Copy` — so the second call has nothing left to call with.

**The fix:** change `describe`'s signature to take a reference:

```rust
fn describe(nickname: &Option<String>) -> String {
    match nickname {
        Some(name) => format!("hello, {name}"),
        None => "hello, stranger".to_string(),
    }
}
```

**Why that's the fix:** `describe` never wanted to own `nickname` — it only wanted to look at it. This is exactly the `Option<&T>` versus `&Option<T>` distinction from the concept section. The compiler's second suggestion — `.clone()` — also works, but it allocates a fresh buffer only to free it immediately; [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md) taught you to spot that: when a borrow is enough, a clone is the wrong tool.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
let x: Option<i32> = Some(0);
match x {
    Some(0) => println!("zero"),
    Some(_) => println!("something else"),
    None => println!("nothing"),
}
```

</details>

<details>
<summary>Answer</summary>

```text
zero
```

`Some(0)` matches the first arm. `0` is a perfectly valid value inside a `Some` — a completely different thing from `None`.

</details>

<details>
<summary>Why can't an <code>Option&lt;u32&gt;</code> be handed straight to a function that wants a <code>u32</code>?</summary>

Because `Option<u32>` and `u32` are two different types — as different as `bool` and `i32`. The compiler does no automatic conversion between them; you have to deal with `None` first.

</details>

<details>
<summary>What does <code>.unwrap()</code>'s panic message NOT tell you that <code>.expect("...")</code>'s does?</summary>

`.unwrap()` only says a `None` got opened. `.expect("...")` says *why* the author thought it couldn't be `None` — the exact text you wrote.

</details>

<details>
<summary>Is <code>Option&lt;&amp;T&gt;</code> the same as <code>&amp;Option&lt;T&gt;</code>?</summary>

No. The first is an `Option` that, if it has anything, has a reference. The second is a reference to the whole `Option`, whatever variant it is. `.as_ref()` is how you get from the second to the first.

</details>

<details>
<summary>Why does <code>size_of::&lt;Option&lt;Box&lt;i32&gt;&gt;&gt;()</code> equal <code>size_of::&lt;Box&lt;i32&gt;&gt;()</code>, but <code>size_of::&lt;Option&lt;i32&gt;&gt;()</code> is bigger than <code>size_of::&lt;i32&gt;()</code>?</summary>

A valid pointer is never all-zero-bits, so `None` can borrow that pattern for free. Every bit pattern of an `i32` is used by a real number, so there's nothing spare for `None`, and `Option` has to take extra room.

</details>

### Repair

Fix all four broken examples:

1. Fix `examples/06-cannot-use-option-directly.rs` so it compiles — with a `match` or `if let` that genuinely answers both cases, not with `.expect()` on the call to `find_user`.
2. Fix `examples/07-unwrap-on-none.rs` and `examples/08-expect-on-none.rs` so neither panics — print something sensible for both cases.
3. Fix `examples/09-matching-by-value-moves-it.rs` **two** ways: once by changing `describe`'s signature to `&Option<String>`, once by calling `.clone()` on `nickname` before the second call. Which is better in a real program, and why?

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-06-01-option-and-null-safety
```

No `.map()`, `.and_then()`, `.unwrap_or()` or `.ok_or()` — those are [1.6.2](../02-option-combinators/README.md). `match` and `if let` are enough for all five.

Read `index_of_first_negative` carefully. It's [1.1.5](../../01-foundations/05-control-flow/README.md)'s function, the one that answered `readings.len()` for "not found". Its signature is now `-> Option<usize>` — see what a caller can no longer get wrong.

### Build

Write a `pub fn record_summary(records: &[Option<u32>]) -> String` producing a one-line summary of an optional score list — how many are known (`Some`), how many aren't (`None`) — in a format you choose and state in the function's doc comment.

Then write a second version that also reports the highest known value — using `match` and a loop, not a combinator (those come later).

### Challenge (optional)

**Part one.** Guess, then check with `size_of`: of these three, `size_of::<Option<Option<i32>>>()`, `size_of::<Option<Option<bool>>>()` and `size_of::<Option<Option<&i32>>>()`, which match their single-layer size exactly, and which one is bigger? (Hint: one is bigger. That one had already spent its only spare bit pattern on the first layer.)

**Part two.** (This one reaches forward.) Look up `Option::flatten` in the standard docs. Guess what it returns for `Some(Some(5))` and for `Some(None::<i32>)` — then check your answer in [1.6.2](../02-option-combinators/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Option<T>` | a two-variant enum: `Some(T)` or `None` | anywhere a value might be absent |
| `Some(x)` / `None` | `Option`'s two constructors | building and matching |
| `.is_some()` / `.is_none()` | ask which variant, without opening it | quick checks |
| `.unwrap()` | takes the value out, or panics generically | when you built the `Some` yourself |
| `.expect("...")` | like `.unwrap()`, with your own panic message | when you believe an invariant |
| `.as_ref()` | `Option<T>` → `Option<&T>`, no move | looking without taking ownership |
| `.cloned()` | `Option<&T>` → `Option<T>`, via a clone | the way back from `.as_ref()` |
| null-pointer optimisation | `Option<Box<T>>` costs nothing over `Box<T>` | whenever `T` is itself a pointer |

### What you now know

- `Option<T>` is an ordinary enum with two variants — `Some(T)` and `None` — not an exception and not a null reference.
- Because `Option<T>`'s type differs from `T`, the compiler won't let you use one in place of the other.
- `match` forces you to answer both cases; `if let` gives up that guarantee, on purpose.
- `.unwrap()` and `.expect("...")` both panic on `None`; they differ in what the panic message tells you.
- `Option<&T>` (a maybe-look) and `&Option<T>` (a reference to the whole box) are not the same type; `.as_ref()` gets you from the second to the first.
- `Option<T>` as a field means "this might not be there" — no sentinel, no hand-rolled convention.
- `Option<Box<T>>`, `Option<&T>` and `Option<bool>` cost no extra bytes over their bare versions; `Option<i32>` does.

### What comes back later

- **`Option` combinators — `.map()`, `.and_then()`, `.unwrap_or()`, `.filter()`, `.ok_or()`** — [1.6.2](../02-option-combinators/README.md)
- **`Result`, `Option`'s cousin for failure that says why** — [1.6.3](../03-result-and-question-mark/README.md)
- **When panicking is the right call, and when it isn't** — [1.6.4](../04-panic-vs-result/README.md)
- **`Box<T>`, and why a heap pointer is never null** — [Phase 2 — `Box` and heap allocation](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md)
- **`Deref`, `AsRef` and the rest of the `.as_ref()` family** — [Phase 2](../../../phase2-intermediate/README.md)

### Can you explain?

- Why won't the compiler let you use an `Option<u32>` in place of a `u32`?
- When is `.unwrap()` defensible? Give a real example you've already seen (not from this lesson).
- What's the difference between what `.unwrap()`'s panic message tells you and what `.expect("...")`'s does?
- Tell `Option<&T>` and `&Option<T>` apart with an example.
- What exactly does `.as_ref()` change that a plain `match` would have changed anyway?
- Why is `Option<Box<T>>` the same size as `Box<T>`, but `Option<i32>` isn't the same size as `i32`?

---

## Going further

- [The Rust Book — Enums and `Option`](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values) — the same ground, officially.
- [Tony Hoare's QCon London 2009 talk](https://www.infoq.com/presentations/Null-References-The-Billion-Dollar-Mistake-Tony-Hoare/) — his own account of "the billion dollar mistake".
- [`std::option::Option`](https://doc.rust-lang.org/std/option/enum.Option.html) — the full list of its methods; worth a look today, even at the ones you haven't been taught yet.
- [The Rustonomicon — layout optimizations](https://doc.rust-lang.org/nomicon/repr-rust.html) — the deeper technical detail behind the null-pointer optimisation, for when you're curious how far it goes.
