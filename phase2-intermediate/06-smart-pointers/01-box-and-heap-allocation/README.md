# 2.6.1 — `Box` and heap allocation

## At a glance

After this lesson you can:

- Explain what `Box<T>` actually is — a smart pointer with a single owner — and say exactly when its heap allocation is freed.
- Say, for any `T`, without running the code, what `size_of::<Box<T>>()` is and why that number never depends on `T`'s size or contents.
- Explain, from memory rather than by repeating it, three facts Phase 1 and Phase 2 only ever handed you as a promise — why `Option<Box<T>>` is free, why `Box<str>` is two words instead of three, why `Box<dyn Trait>` has a fixed size.

**Time:** ~70 minutes · **Prerequisites:**
[1.2.1 — Stack and heap](../../../phase1-fundamentals/02-ownership-and-memory/01-stack-and-heap/README.md),
[1.2.5 — `Drop` and RAII](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.md),
[2.3.7 — Static dispatch versus dynamic, and object safety](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md),
[2.4.3 — `Deref`, `AsRef`, `Borrow`, `ToOwned`](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md)

---

## Why this matters

From Phase 1 all the way to today, `Box<T>` has shown up six times — and every single time, the lesson said exactly the one piece you needed right then, and left an IOU for later:

- [1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) set its size next to `String` and `&str`: "an owner with no capacity, two words instead of three." Never said why exactly two.
- [1.5.3](../../../phase1-fundamentals/05-your-own-types/03-enums-as-data/README.md) showed you `size_of::<Option<Box<i32>>>()` equals `size_of::<Box<i32>>()` — "niche optimisation" — and moved on in one sentence.
- [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md) showed you the same fact from `Option`'s side, and dismissed it in one line: "`Box<T>` is a heap pointer, owning what it points at — that one line is all you need today; its full lesson is in Phase 2."
- [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) reached for `Box<dyn Trait>` to build a heterogeneous `Vec`, and said: "The full story of `Box`... belongs to 2.6.1; today only this one role of it is needed."
- [2.4.3](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md) showed you `Box<T>` carries the `Deref` trait — which is why a `Box<String>` behaves like a `String` — and repeated the same sentence: "The full story of `Box` — heap allocation, ownership — belongs to 2.6.1."
- [2.5.2](../../05-error-handling/02-error-source-chains/README.md) repeated that exact sentence, this time about `Box<dyn Error>`.

Six loans, zero repayments. Today is repayment day.

And the point isn't just to hear a definition. The point is that a single fact — the size of `Box<T>` never depends on `T`'s size or contents — settles every one of those six loans at once. That one sentence explains why `Option<Box<T>>` is free, why `Box<str>` is two words instead of three, and why `Box<dyn Trait>` has one fixed size no matter which concrete type stands behind it. By the end of this lesson, all three go from something you took on faith to something you've proven yourself with `size_of`.

---

## The concept

### What `Box<T>` is: a smart pointer with one owner

A **smart pointer** is a struct that behaves like a pointer — it lets you get at some data — but also carries an extra promise: it owns what it points to. `Box<T>` is the simplest one in that family: a value of type `T`, moved onto the heap, with a plain pointer on the stack pointing at it.

```rust
let boxed: Box<i32> = Box::new(5);
println!("boxed  = {boxed}");
println!("*boxed = {}", *boxed);
```

```text
boxed  = 5
*boxed = 5
```

`Box::new(5)` takes the `5` off the stack, puts it on the heap, and hands back a `Box<i32>` that is just a pointer to it. To read what's inside, you have `*boxed` — the same dereference operator from [1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md), only this time it's unwrapping an owner instead of a borrowed reference.

Most of the time you don't even need that `*`. [2.4.3](../../04-lifetimes-and-conversion/03-deref-asref-borrow/README.md) showed you `Box<T>` implements `Deref`, so the compiler follows the pointer for you at a method call:

```rust
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn distance_from_origin(&self) -> f64 {
        ((self.x * self.x + self.y * self.y) as f64).sqrt()
    }
}

let boxed_point = Box::new(Point { x: 3, y: 4 });
println!("distance = {}", boxed_point.distance_from_origin());
```

```text
distance = 5
```

`Box<Point>` has no method of its own called `distance_from_origin`. The compiler inserts a `*` for you, reaches the `Point` on the heap, and finds the method there. This is called **auto-deref**, and it's the cousin of the **deref coercion** [1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) showed you on `&String` → `&str` — both come from the same general trait, not something special-cased to `Box`.

And ownership is exactly the same rule as always, with one extra layer of indirection: the `Box` owns the value on the heap, and at the closing brace of its scope — exactly what [1.2.5](../../../phase1-fundamentals/02-ownership-and-memory/05-drop-and-raii/README.md) gave RAII — `T`'s destructor runs and the heap block is freed:

```rust
struct Loud(String);

impl Drop for Loud {
    fn drop(&mut self) {
        println!("dropping: {}", self.0);
    }
}
```

```rust
println!("--- entering inner scope ---");
{
    let inner = Box::new(Loud(String::from("inner")));
    println!("inner alive: {}", inner.0);
}
println!("--- inner scope ended ---");
```

```text
--- entering inner scope ---
inner alive: inner
dropping: inner
--- inner scope ended ---
```

`dropping: inner` prints right before `--- inner scope ended ---`, not after. Until now you'd seen RAII on stack values; this is the first time you're watching the exact same rule on something that genuinely lives on the heap — and the behaviour hasn't changed one bit.

```senpai-visual
{"kind":"ownership","labels":["Box<T>: one pointer on the stack","T: on the heap","the Box owns it","freed when the Box's scope ends"]}
```

### Why you'd want the heap at all

If every value fits on the stack right now and the stack is faster — what [1.2.1](../../../phase1-fundamentals/02-ownership-and-memory/01-stack-and-heap/README.md) told you — why reach for this extra layer at all? Two real reasons.

**First: a value that's expensive to move around on the stack.** Moving a large value directly means every one of its bytes gets copied. Moving a `Box` means only that one pointer gets copied — always, no matter how big what it points at is:

```rust
let big = Box::new(BigBuffer { data: [0; 100_000] });
println!("heap address before move: {big:p}");

let moved = big; // only the pointer moved — not 100,000 bytes
println!("heap address after move:  {moved:p}");
```

```text
heap address before move: 0x1e9255c8f40
heap address after move:  0x1e9255c8f40
```

The heap address before and after the move is identical — because not one byte of `data` moved; only an eight-byte pointer went from one variable to another. The exact digits change every run — the OS puts your program somewhere new each time, exactly what [1.2.1](../../../phase1-fundamentals/02-ownership-and-memory/01-stack-and-heap/README.md) showed you — but the "before" and "after" staying equal never changes. Full numbers are in "Hands on".

**Second, and more important for the rest of this lesson: a value whose size isn't known at compile time at all.** "Big" and "unknown" are not the same thing. A `[u8; 100_000]` is big, but its size is completely known and fixed — the compiler knows exactly how much frame to reserve. But some types have no such number at all — [1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) showed you one: bare `str`, without a `&`. For these, putting them on the stack directly isn't even an option — a few sections down you'll see why — and that's exactly where `Box` earns its second job.

### `Box`'s size: always one word, for any `Sized` `T`

Here's today's central claim: for any `T` that is **`Sized`** — meaning the compiler knows exactly how many bytes it is at compile time, which is true of almost every type you've met so far — `Box<T>` is always exactly one machine word. One pointer. That's it, no matter how small or large `T` is.

```rust
struct Tiny;
struct Small {
    id: u32,
}
struct Large {
    buffer: [u8; 4096],
}
```

```rust
println!("size_of::<Tiny>()   = {}", size_of::<Tiny>());
println!("size_of::<Small>()  = {}", size_of::<Small>());
println!("size_of::<Large>()  = {}", size_of::<Large>());
println!("size_of::<Box<Tiny>>()  = {}", size_of::<Box<Tiny>>());
println!("size_of::<Box<Small>>() = {}", size_of::<Box<Small>>());
println!("size_of::<Box<Large>>() = {}", size_of::<Box<Large>>());
```

```text
size_of::<Tiny>()   = 0
size_of::<Small>()  = 4
size_of::<Large>()  = 4096
size_of::<Box<Tiny>>()  = 8
size_of::<Box<Small>>() = 8
size_of::<Box<Large>>() = 8
```

`Tiny` is zero bytes. `Large` is four thousand ninety-six bytes. And a `Box` around either one is exactly the same eight bytes — exactly what a `&T` would have been too. Even boxing another box keeps the same rule: `size_of::<Box<Box<i32>>>()` is also eight, because `Box<i32>` is itself a `Sized` type — just like any other `T`.

This is exactly what solves the "big value" problem above: moving a `Box<Large>` is always eight bytes, whether `Large` is four kilobytes or forty megabytes.

```senpai-visual
{"kind":"concept","labels":["small T: a few bytes","large T: thousands of bytes","Box<small T>: 8 bytes","Box<large T>: 8 bytes"]}
```

### The niche optimisation, explained in full this time

Now go back to the loan from [1.5.3](../../../phase1-fundamentals/05-your-own-types/03-enums-as-data/README.md) and [1.6.1](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.md). Both showed you this exact number:

```rust
println!("size_of::<Box<i32>>()      = {}", size_of::<Box<i32>>());
println!("size_of::<Option<Box<i32>>>() = {}", size_of::<Option<Box<i32>>>());
```

```text
size_of::<Box<i32>>()      = 8
size_of::<Option<Box<i32>>>() = 8
```

And both only said it comes down to a valid `Box` never being all-zero-bits — one put it as "a `Box` is never a null pointer," the other as "never all-zero-bits" — same fact, two phrasings. Now you have the full version. `Box::new` always performs a real allocation and hands back a real pointer — there is no general way in safe Rust to build a `Box<T>` whose insides are zero. So of the 2⁶⁴ possible bit patterns for an eight-byte pointer, exactly one — all-bits-zero — never occurs. The compiler borrows that impossible pattern for `None`, so `Option<Box<T>>` costs no extra bytes at all. That's the **niche optimisation** [1.5.3](../../../phase1-fundamentals/05-your-own-types/03-enums-as-data/README.md) named.

Compare that to `Option<i32>`: every one of a `i32`'s 2³² bit patterns is a real number — no unused pattern is left over for `None` to borrow, so `Option<i32>` has to buy a separate tag byte and rounds up to eight. The difference isn't about `T` itself; it's about whether `T` — here, `Box<T>` — has an impossible bit pattern to hand `None` for free or not.

### When `T` has no size: the fat pointer

Every `T` you've met so far has been **`Sized`**. But [1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) also showed you an **unsized type**: bare `str`. Its size isn't known at compile time — it could be three bytes or three million — so you can't put it on the stack, hold it in a variable, or pass it to a function by value. [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) gave you a second example: `dyn Trait`, once the concrete type underneath has been erased and the compiler no longer knows how many bytes it is.

Here "always one word" stops being possible — one word only has room for one address, and a raw address isn't enough for an unsized type. `Box<T>` for an unsized `T` becomes a **fat pointer** instead — two words, not one — exactly what [1.3.4](../../../phase1-fundamentals/03-borrowing-and-references/04-slices/README.md) showed you for slices and [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) showed you for trait objects: one pointer to the real data, plus whatever it takes to know "how much" or "which implementation."

```rust
let owned_str: Box<str> = "senpai".into();
println!("size_of::<Box<str>>() = {}", size_of::<Box<str>>());
println!("size_of::<Box<dyn Playable>>() = {}", size_of::<Box<dyn Playable>>());
```

```text
size_of::<Box<str>>() = 16
size_of::<Box<dyn Playable>>() = 16
```

Two words — sixteen bytes — and **still fixed**, regardless of how many characters the string has or which concrete type sits behind `dyn Playable`. Full numbers are in "Hands on".

```senpai-visual
{"kind":"concept","labels":["&str: pointer + length","Box<str>: pointer + length","Box<dyn Trait>: pointer + vtable","always two words, never more"]}
```

### Resolving what 1.4.1 showed you: `Box<str>`

[1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) wrote this exact sentence: "`Box<str>` — an owner with no capacity, two words instead of three." Now you can prove it yourself:

```rust
println!("size_of::<String>()   = {}", size_of::<String>());
println!("size_of::<Box<str>>() = {}", size_of::<Box<str>>());
println!("size_of::<&str>()     = {}", size_of::<&str>());
```

```text
size_of::<String>()   = 24
size_of::<Box<str>>() = 16
size_of::<&str>()     = 16
```

`String` is three words because it grows — its third word, capacity, is exactly what tracks the extra room it has reserved. `Box<str>` never grows: its size is whatever it was allocated with, forever. So it carries the same two words `&str` has — a pointer, a length — with one difference: `Box<str>` **owns** those bytes, `&str` only **views** them. The same owner/view split all of [1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) was built on works exactly the same way here — just with the owner sitting on the heap instead of a three-word struct.

### Resolving what 2.3.7 and 2.5.2 showed you: `Box<dyn Trait>`

[2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) put a `Box<dyn Summarize>` around an `AnimeSeries` and another `Box<dyn Summarize>` around a `MangaVolume` — two genuinely different concrete types, different sizes — so both could sit in one `Vec`. [2.5.2](../../05-error-handling/02-error-source-chains/README.md) did the same thing to `Box<dyn Error>`, to gather `ConfigError` and `ParseIntError` behind one return type. Both said the full story of `Box` belonged to this lesson; here's the full story:

```rust
struct Song {
    title: String,
}
struct Podcast {
    title: String,
    duration_minutes: u32,
}
```

```rust
println!("size_of::<Song>()    = {}", size_of::<Song>());
println!("size_of::<Podcast>() = {}", size_of::<Podcast>());
println!("size_of::<Box<dyn Playable>>() = {}", size_of::<Box<dyn Playable>>());
```

```text
size_of::<Song>()    = 24
size_of::<Podcast>() = 32
size_of::<Box<dyn Playable>>() = 16
```

`Song` and `Podcast` are different sizes — twenty-four versus thirty-two bytes. But `Box<dyn Playable>` is exactly sixteen bytes for either one. The first word is the same pointer to the real data — that `Song` or that `Podcast` on the heap. The second word, exactly what [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) explained, is a pointer to that concrete type's **vtable** — not the real data, just the address of where to find the right implementation of `play`. `Box` does nothing new here; it just puts that same two-word fat pointer `&dyn Playable` already was onto the heap and owns it, exactly as it does for any `Sized` `T`.

That's exactly the promise [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) made: "`Box<dyn Summarize>` is a fixed size for either one." Now you know why — and you know exactly where that size comes from.

### One sentence further: recursive, and shared ownership

Two doors this lesson deliberately leaves closed: when `T` itself holds a `Box<T>` inside it — a recursive type — and boxed trait objects belong to [2.6.2](../02-recursive-types-and-trait-objects/README.md); and when a single owner genuinely isn't enough because more than one part of the program truly needs to hold a value together, that's where `Rc` and `Arc` come in — belonging to [2.6.3](../03-rc-and-arc/README.md).

---

## Hands on

```sh
cargo run -p p2-06-01-box-and-heap-allocation --example 01-box-basics
cargo run -p p2-06-01-box-and-heap-allocation --example 02-moving-a-box-is-cheap
cargo run -p p2-06-01-box-and-heap-allocation --example 03-size-of-sized-types
cargo run -p p2-06-01-box-and-heap-allocation --example 04-unsized-fat-pointers
```

Run `02-moving-a-box-is-cheap` a few times. The address itself changes every run — exactly like [1.2.1](../../../phase1-fundamentals/02-ownership-and-memory/01-stack-and-heap/README.md) — but the "before" and "after" staying equal never changes.

Then the two broken ones:

```sh
cargo run -p p2-06-01-box-and-heap-allocation --example 05-box-str-from-literal-broken --features broken
cargo run -p p2-06-01-box-and-heap-allocation --example 06-move-out-through-shared-ref-broken --features broken
```

Then try:

1. In `01-box-basics`, build a second `Loud` inside the same inner block (say, with text `"second"`) and print it before the block ends. Which one prints `dropping` first?
2. In `02-moving-a-box-is-cheap`, grow `BigBuffer` to a million bytes. Does `size_of::<Box<BigBuffer>>()` change?
3. In `03-size-of-sized-types`, add a struct of your own with five or six fields. Before running, guess its size and the size of a `Box` around it, then run and check.

---

## Errors you will meet

### `E0308` — a `Box<&str>` where `Box<str>` was wanted

```text
error[E0308]: mismatched types
 --> phase2-intermediate\06-smart-pointers\01-box-and-heap-allocation\examples\05-box-str-from-literal-broken.rs:6:26
  |
6 |     let text: Box<str> = Box::new("senpai");
  |               --------   ^^^^^^^^^^^^^^^^^^ expected `Box<str>`, found `Box<&str>`
  |               |
  |               expected due to this
  |
  = note: expected struct `Box<_>`
             found struct `Box<&_>`

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is objecting to:** `"senpai"` is a literal, whose type is `&str` — not `str`. `Box::new` takes a **`Sized`** value and puts that exact value on the heap; what went onto the heap here was a `&str`, so the result is `Box<&str>` — a plain pointer to another fat pointer, not a `Box<str>`.

**The fix:** `str` can't be boxed with `Box::new`, because `Box::new` never has a bare `str` value in hand to put on the heap in the first place — exactly the "unsized type" problem [1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md) showed you. The right way is a conversion:

```rust
let text: Box<str> = "senpai".into();
```

**Why that's the fix:** `.into()` here uses an `impl From<&str> for Box<str>` the standard library wrote — that conversion itself, behind the scenes, both allocates and copies the bytes, and the result is a genuine `Box<str>`, not a `Box` around another pointer. `Box::new` is for boxing a value that already exists and is `Sized`; building a `Box` around something that could never sit on the stack by itself always goes through a conversion instead.

### `E0507` — moving out from behind a shared reference

```text
error[E0507]: cannot move out of `**boxed` which is behind a shared reference
  --> phase2-intermediate\06-smart-pointers\01-box-and-heap-allocation\examples\06-move-out-through-shared-ref-broken.rs:10:5
   |
10 |     **boxed
   |     ^^^^^^^ move occurs because `**boxed` has type `Profile`, which does not implement the `Copy` trait
   |
note: if `Profile` implemented `Clone`, you could clone the value
  --> phase2-intermediate\06-smart-pointers\01-box-and-heap-allocation\examples\06-move-out-through-shared-ref-broken.rs:5:1
   |
 5 | struct Profile {
   | ^^^^^^^^^^^^^^ consider implementing `Clone` for this type
...
10 |     **boxed
   |     ------- you could clone this value

For more information about this error, try `rustc --explain E0507`.
```

**What the compiler is objecting to:** `boxed` has type `&Box<Profile>` — a shared reference to a `Box`. `**boxed` unwraps twice: once to reach the `Box` itself, once more to reach the `Profile` on the heap. The function wants to return that `Profile` **by value** — pull it out of its spot — but all it actually holds is a borrow, not ownership.

**The fix:** one of two things — either change the signature to actually take ownership:

```rust
fn take_it(boxed: Box<Profile>) -> Profile {
    *boxed
}
```

or, if you genuinely only have a borrow, return a reference instead of a value.

**Why that's the fix:** `Box<T>` makes no exception to the ownership rule at all — exactly what [1.2.4](../../../phase1-fundamentals/02-ownership-and-memory/04-ownership-across-functions/README.md) taught you about ordinary values applies here without change. Once `take_it` genuinely takes `boxed: Box<Profile>` (not `&Box<Profile>`), the function owns that `Box`, and `*boxed` — this time with no reference in front of it — is allowed, because Rust lets you pull the value out of a `Box` you actually own outright. The point isn't that `Box` bends the rule; the point is actually having ownership, not just a borrow of it.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
let b: Box<i32> = Box::new(5);
let n: i32 = b;
```

</details>

<details>
<summary>Answer</summary>

No — `E0308`, "expected `i32`, found `Box<i32>`". `Box<i32>` and `i32` are different types; the compiler itself suggests `*b`.

</details>

<details>
<summary>Do <code>size_of::&lt;Box&lt;[u8; 4]&gt;&gt;()</code> and <code>size_of::&lt;Box&lt;[u8; 4_000_000]&gt;&gt;()</code> differ?</summary>

No, both are eight bytes. `Box<T>`'s size is always one word for any `Sized` `T`, no matter how many bytes `T` itself is.

</details>

<details>
<summary>Why does <code>size_of::&lt;Option&lt;Box&lt;i32&gt;&gt;&gt;()</code> equal <code>size_of::&lt;Box&lt;i32&gt;&gt;()</code>?</summary>

Because a valid `Box<i32>` is never all-bits-zero — `Box::new` always performs a real allocation. The compiler borrows that impossible pattern for `None` and pays no extra bytes for it. The niche optimisation.

</details>

<details>
<summary>What is <code>size_of::&lt;Box&lt;str&gt;&gt;()</code>?</summary>

Sixteen — a fat pointer (pointer + length), because `str` is unsized. Less than `size_of::<String>()`, which is twenty-four, because `Box<str>` has no third, capacity word.

</details>

<details>
<summary>A <code>Song</code> is twenty-four bytes and a <code>Podcast</code> is thirty-two. What is <code>size_of::&lt;Box&lt;dyn Playable&gt;&gt;()</code> for each?</summary>

The same for both — sixteen bytes. `Box<dyn Trait>`'s size never depends on the concrete type behind it; it's always a two-word fat pointer: data + vtable.

</details>

### Repair

Fix `examples/05-box-str-from-literal-broken.rs` — not by changing the declared type, but by changing how you build the `Box<str>`.

Fix `examples/06-move-out-through-shared-ref-broken.rs` **two** ways:

1. By changing `take_it`'s signature so it genuinely takes ownership of `Box<Profile>`.
2. Without changing the signature — keep `boxed: &Box<Profile>` — but have the function return only a reference, not a `Profile` by value.

Then say which one you'd actually reach for in real code, and why.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p2-06-01-box-and-heap-allocation
```

The first three are plain mechanics: build, pull out, mutate through a reference. The fourth measures today's central claim in numbers. The fifth builds a `Vec<Box<dyn Playable>>` from the `Playable` trait you saw above — already written, not `todo!()`.

### Build

Design a `struct` that means something in the real world and has at least one field clearly larger than the rest — a text buffer, say, or a fixed-size array. Measure it with `size_of`. Then build a second version with that same large field boxed, and measure that one too.

Then write a paragraph: when does that difference actually matter in real code? (Hint: think back to [1.5.3](../../../phase1-fundamentals/05-your-own-types/03-enums-as-data/README.md) — the lesson where one oversized variant dragged up an entire `enum`'s size.)

### Challenge (optional)

**Part one.** Build a `Vec<i32>`, then turn it into a `Box<[i32]>` with `.into_boxed_slice()`. Put `size_of::<Vec<i32>>()` next to `size_of::<Box<[i32]>>()` — `[i32]` is also unsized (no length in its own type), so the same fat-pointer rule should hold. Build a `Box<[i32]>` with three elements and one with five hundred, and compare their sizes.

**Part two.** This one reaches backward, not forward. In [1.5.3](../../../phase1-fundamentals/05-your-own-types/03-enums-as-data/README.md) — Challenge, part two — you wrote an `enum Chain { End, Link(u32, Chain) }`, hit `E0072`, and the lesson said the compiler names its own fix. Go build that same type again (or find it if you already solved it there), put a `Box` around the recursive field, and this time — instead of just watching it compile — use `size_of` to say exactly how many bytes `Chain` comes out to, and why it's that number and not another one.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Smart pointer | a struct that behaves like a pointer and also owns what it points at | `Box<T>`, and later `Rc`/`Arc` |
| `Box<T>` | the simplest smart pointer: a value on the heap, with a single owner | anywhere a value is large, or its size isn't known at compile time |
| `Sized` | a type the compiler knows the exact byte size of at compile time | almost every `T` you've met so far |
| Unsized type | a type without such a number — `str`, `dyn Trait` | why `Box<T>` sometimes becomes a fat pointer |
| Fat pointer | a two-word pointer: data + length or vtable | `Box<str>`, `Box<dyn Trait>` |
| Niche optimisation | using an impossible bit pattern instead of an extra byte for `None` | `Option<Box<T>>` is the same size as `Box<T>` |

### What you now know

- `Box<T>` puts a value on the heap, owns it, and follows the same RAII rule as always — freed at the closing brace of its scope.
- There are two real reasons to want the heap: a value that's expensive to move on the stack, and a value whose size isn't known at compile time at all.
- For any `Sized` `T`, `Box<T>` is always exactly one word — no matter how small or large `T` is.
- `Option<Box<T>>` costs no extra bytes, because a valid `Box` is never all-bits-zero and the compiler borrows that pattern for `None`.
- When `T` is unsized (`str`, `dyn Trait`), `Box<T>` becomes a two-word fat pointer — still fixed, just two words instead of one.
- `Box<str>` is two words because it lacks `String`'s third, capacity word; `Box<dyn Trait>` is the same size no matter which concrete type sits behind it.

### What comes back later

- **Recursive types, when `T` itself holds a `Box<T>` inside it** — [2.6.2 — Recursive types and boxed trait objects](../02-recursive-types-and-trait-objects/README.md)
- **When a single owner isn't enough** — [2.6.3 — `Rc` and `Arc`](../03-rc-and-arc/README.md)
- **Mutating something you were only handed a shared reference to** — [2.6.5 — `RefCell` and interior mutability](../05-refcell-and-interior-mutability/README.md)

### Can you explain?

- What is `Box<T>`, and exactly when and where does its memory get freed?
- Name the two real reasons for wanting something on the heap.
- Why does `size_of::<Box<T>>()` never depend on `T`'s size or contents? Say one sentence for `Sized` `T` and one for unsized `T`.
- Why is `Option<Box<T>>` free but `Option<i32>` isn't?
- Why is `Box<str>` two words and `String` three?
- Why does `Box<dyn Trait>` have one size no matter which concrete type is behind it?

---

## Going further

- [The Rust Book — 15.1: `Box<T>`](https://doc.rust-lang.org/book/ch15-01-box.html) — the same ground, officially, with the classic cons-list example the next chapter builds on.
- [`Box<T>` documentation](https://doc.rust-lang.org/std/boxed/struct.Box.html) — the same type you saw from every angle today.
- [`std::mem::size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html) — this whole lesson's measuring tool.
- [The Rustonomicon — Exotically Sized Types](https://doc.rust-lang.org/nomicon/exotic-sizes.html) — the more technical detail behind unsized types and fat pointers, for when you're curious how deep it goes.
