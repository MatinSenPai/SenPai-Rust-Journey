# 1.4.1 — `String` versus `&str`

## At a glance

After this lesson you can:

- Say how many words a `String` is made of and how many a `&str` is, and what each word does.
- Choose the right parameter and return types for a function you're writing, and say why the rule is "take `&str`, return `String`".
- Explain why `&String` works where a `&str` was asked for, without saying "Rust handles it".
- Choose between `.to_string()`, `String::from`, `.to_owned()`, `.as_str()` and `&*`, and say which allocate and which are free.

**Time:** ~50 minutes · **Prerequisites:** [1.3.4 — Slices](../../03-borrowing-and-references/04-slices/README.md) and [1.1.6 — `Vec` and `String` basics](../../01-foundations/06-vec-and-string-basics/README.md)

---

## Why this matters

[1.1.6](../../01-foundations/06-vec-and-string-basics/README.md) gave you one paragraph on these two types and one rule: "take `&str`, return `String`". That paragraph was enough to write code with. It isn't enough to read somebody else's code with, or to know why the compiler objected when it objects.

This lesson opens the paragraph up.

Python has one text type and that's the end of it: `str`. You pass it wherever text is wanted, you get it back, and where its bytes live or who's responsible for freeing them is never your question. Rust split that one type into two: **the owner of the buffer** and **a view of the buffer**. That's why Rust code has two names where Python had one.

And the split isn't decoration. A function taking `&str` accepts text from every caller and copies nothing. The same function taking `String` forces the caller either to give up ownership or to make a pointless copy. In a real program the difference between those two signatures is the difference between hundreds of necessary allocations and hundreds of unnecessary ones.

You also have a motive an English-speaking programmer doesn't. `len()` counts bytes, and for Persian a byte is not a letter. Until you know which type owns those bytes and which merely looks at them, you can't reason correctly about cutting them or counting them. This lesson is the ground the next two stand on.

---

## The concept

### Two types, two shapes — and the numbers

You have `size_of` from [1.2.1](../../02-ownership-and-memory/01-stack-and-heap/README.md). Point it at both types:

```rust
println!("size_of::<String>()  = {}", std::mem::size_of::<String>());
println!("size_of::<&str>()    = {}", std::mem::size_of::<&str>());
println!("size_of::<&String>() = {}", std::mem::size_of::<&String>());
```

```text
size_of::<String>()  = 24
size_of::<&str>()    = 16
size_of::<&String>() = 8
```

On a 64-bit machine a word is 8 bytes. So:

| Type | Words | What they are |
|---|---|---|
| `String` | 3 | pointer, length, capacity |
| `&str` | 2 | pointer, length |
| `&String` | 1 | a pointer to those three words |

```senpai-visual
{"kind":"ownership","labels":["String: pointer + length + capacity","the heap buffer","&str: pointer + length"]}
```

That table holds the whole lesson. A `String` is an **owner**: three words on the stack, plus a heap buffer it is responsible for. A `&str` is a **view**: two words pointing at bytes that somebody else owns.

And that one-word `&String` is an extra hop — a pointer to three words that themselves contain a pointer. Hold onto it; it comes back in the `clippy::ptr_arg` section.

### Those two words point into somebody else's bytes

```rust
let owned = String::from("سلام دنیا");
let view: &str = owned.as_str();

println!("owned @ {:p}", owned.as_ptr());
println!("view  @ {:p}", view.as_ptr());
```

```text
owned @ 0x27daef69790
view  @ 0x27daef69790
```

**One address, two names.** The second line allocated nothing and copied no bytes; it took the pointer and the length out of those three words and set them side by side.

Now the term: a `&str` is a **string slice**. It's exactly what you saw in [1.3.4](../../03-borrowing-and-references/04-slices/README.md) — `&[T]` was a pointer and a length too, a view into a `Vec`'s buffer — with one extra promise: the bytes a `&str` points at are guaranteed to be valid UTF-8.

And because it's a view, every borrowing rule applies to it. While `view` is alive, `owned` is lent out. That's [1.3.3](../../03-borrowing-and-references/03-borrow-scopes-and-nll/README.md), applied to text.

### The third word, the one the view hasn't got

```rust
let mut growing = String::from("سلام");
growing.push_str(" دنیا");
```

```text
before push: len/cap = 8/8 @ 0x27daef560f0
after  push: len/cap = 17/17 @ 0x27daef560f0
```

Capacity went from 8 to 17. **Capacity is the third word**, and it's what makes growing possible: a `String` knows how much room it reserved, so it knows when to go and get a bigger block. (Here the allocator managed to grow it in place and the address survived. That's not guaranteed.)

A `&str` hasn't got that word, and doesn't own the buffer either. So:

```rust
let mut view: &str = "hello";
view.push_str(", world");
```

```text
error[E0599]: no method named `push_str` found for reference `&str` in the current scope
 --> src\lib.rs:3:10
  |
3 |     view.push_str(", world");
  |          ^^^^^^^^ method not found in `&str`
```

The method simply isn't there. And look at that `mut`: it's on the **binding**, not on the text. You can point `view` at different text; you can't change the text it points at.

> **The one-liner:** `String` grows, `&str` looks.

### Why you never write `str` bare

`str` is a real type in its own right: the bytes themselves, however many of them there are. And "however many there are" is exactly the problem — its size isn't known at compile time. A type like that is called an **unsized type**.

To put a value in a variable, pass it to a function, or return it, the compiler has to know how many bytes of room it needs. For `str` it doesn't. So everything you ever do with text goes through a pointer to it:

- `&str` — a view of those bytes. What you write 99% of the time.
- `String` — an owner holding those bytes on the heap.
- `Box<str>` — an owner with no capacity, two words instead of three. You won't need it before [Phase 2](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md).

The exact error is in the errors section, and you can produce it yourself.

### Literals: text baked into the executable

```rust
let literal: &str = "سلام";
println!("literal     = {literal}");
println!("literal len = {} bytes", literal.len());
println!("literal   @ {:p}", literal.as_ptr());
```

```text
literal     = سلام
literal len = 8 bytes
literal   @ 0x7ff6b3349440
```

Look at that address: `0x7ff6…`. The heap addresses a few lines further down are `0x142d…`. These eight bytes aren't on the heap at all — they're part of the executable file itself, loaded into memory when the program started. Nothing allocated them and nothing will free them.

That's why the type of a **string literal** is a `&str` and not a `String`: there's no buffer for anyone to own.

Its full type is `&'static str`. The `'static` says "these bytes are there for as long as the program is". You'll see that notation on and off until [Phase 2](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md) gives lifetimes their own lesson; for now, read it as "a view of text that is always there".

### The conversions, and what each costs

Going up — from a view to an owner — has three common spellings:

```rust
let a: String = literal.to_string();
let b: String = String::from(literal);
let c: String = literal.to_owned();

println!("to_string()   @ {:p}", a.as_ptr());
println!("String::from  @ {:p}", b.as_ptr());
println!("to_owned()    @ {:p}", c.as_ptr());
```

```text
to_string()   @ 0x142de1bef70
String::from  @ 0x142de1bef90
to_owned()    @ 0x142de1befb0
```

**Three different heap addresses.** All three went to the allocator and copied the bytes across. This direction is the expensive one.

Going down — from an owner to a view — has three spellings too, and all three are free:

```rust
let view_one: &str = a.as_str();
let view_two: &str = &*a;

println!("a           @ {:p}", a.as_ptr());
println!("a.as_str()  @ {:p}", view_one.as_ptr());
println!("&*a         @ {:p}", view_two.as_ptr());
```

```text
a           @ 0x142de1bef70
a.as_str()  @ 0x142de1bef70
&*a         @ 0x142de1bef70
```

**One address, three names.** None of them allocated; each put two words on the stack.

`&*a` is worth a second look. `*a` opens the `String` up to reach the `str` it's holding — the unsized type from two subsections ago — and the `&` immediately takes a view of that. You have `*` from [1.3.1](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md). You'll rarely write `&*` yourself; it matters because it's what the compiler writes for you in the next subsection. So rarely, in fact, that clippy objects to it in the example file — `&a` would have done. It's spelled out there on purpose, with the lint silenced on that one line, so you can see the step you're about to stop seeing.

| From → to | How | Cost |
|---|---|---|
| `&str` → `String` | `.to_string()` | allocate + copy |
| `&str` → `String` | `String::from(s)` | allocate + copy |
| `&str` → `String` | `.to_owned()` | allocate + copy |
| `String` → `&str` | `s.as_str()` | free |
| `String` → `&str` | `&*s` | free |
| `String` → `&str` | `&s` wherever a `&str` is wanted | free |

### `.to_owned()` versus `.to_string()`

All three ways up produce a `String` and — as the addresses showed — all three allocate exactly once. So which do you write?

- `.to_string()` means **"render this as text"**. It's the machinery `println!("{}")` uses. For a `&str` the standard library short-circuits it straight to `String::from`, so there's no difference in cost.
- `.to_owned()` means **"give me the owned form of this borrowed thing"**. It's the same method that turns a `&[i32]` into a `Vec<i32>`.

Same result here, different sentence. And there's one place they genuinely differ:

```rust
let n = 5;
println!("n.to_string() is text: {}", n.to_string().len());
println!("n.to_owned() is still a number: {}", n.to_owned() + 1);
```

```text
n.to_string() is text: 1
n.to_owned() is still a number: 6
```

`.to_string()` works on anything printable and always produces text. `.to_owned()` only means something where there's a borrowed/owned pair, and gives back the same type. For a `&str` they land in the same place; for a number they don't.

### Why `&String` works where a `&str` is wanted

Back to `03-the-signature-rule`. `byte_length` there is declared `fn byte_length(text: &str) -> usize`, and here it is called three ways:

```rust
println!("literal    -> {}", byte_length(literal));
println!("&String    -> {}", byte_length(&owned));
println!("&str       -> {}", byte_length(borrowed));
```

```text
literal    -> 6
&String    -> 5
&str       -> 5
```

Look at the middle line. `byte_length` is declared as taking a `&str` and we handed it a `&String`. Two different types — and it compiled.

The name for it is **deref coercion**, and what it does is precisely the `&*` from two subsections ago: the compiler sees it has a `&String` and needs a `&str`, so it inserts the `&*` for you. It does the same for `&Vec<T>` → `&[T]` — which is why the functions in [1.3.4](../../03-borrowing-and-references/04-slices/README.md) took `&[T]` and you could still call them with a `&Vec<T>`.

Now the limits of the convenience, because half the confusion lives here:

- It only goes **down**. `&String` becomes `&str`; a `&str` never becomes a `&String` on its own.
- It only works through a reference. A `String` with no `&` in front of it will not compile where a `&str` is wanted, and that's the `E0308` in the next section.
- **The compiler will insert a deref; it will never insert an allocation.** Anywhere an allocation is needed, you write it. That's Rust's standing rule: an allocation should be visible in the code.

Python doesn't have this distinction because its `str` is immutable, and when a thing can never change, "a view of it" and "it" can be treated as one. Rust's `String` is mutable and can move its buffer — which is exactly why the view has to be a separate type, one that can promise it won't grow. That's where the Python analogy stops.

### The signature rule

> **Take `&str`. Return `String`.**

The first half, because every caller can hand you a `&str` for free — out of a literal, out of a `String` with one `&`, out of a view they borrowed themselves. A function taking `String` forces the caller to give up ownership or to make a pointless copy.

The second half, because the text a function makes is new and has to outlive the function. A view has to point at something that lives longer than the function does, and a local variable doesn't. Trying it is `E0515`, and you'll see it in the next section.

```rust
fn shout(text: &str) -> String {
    text.to_uppercase()
}

println!("shout(literal) = {}", shout(literal));
println!("shout(&owned)  = {}", shout(&owned));
```

```text
shout(literal) = SENPAI
shout(&owned)  = MATIN
```

And because it's a rule rather than a superstition, know its two exceptions:

- **Take `String` when you're going to keep it or grow it.** If you need a buffer anyway, let the caller hand you one they've finished with; that's one allocation fewer. The `extended` exercise in this lesson is exactly that.
- **Take `&str` and return `&str` when the answer is a piece of the input rather than new text.** No allocation is needed there. [1.4.4](../04-slicing-text-safely/README.md) does that — and shows why it's dangerous at the wrong offset in Persian text.

---

## Hands on

```sh
cargo run -p p1-04-01-string-vs-str --example 01-two-shapes
cargo run -p p1-04-01-string-vs-str --example 02-literals-and-conversions
cargo run -p p1-04-01-string-vs-str --example 03-the-signature-rule
```

Then the three broken ones:

```sh
cargo run -p p1-04-01-string-vs-str --example 04-string-where-str-wanted --features broken
cargo run -p p1-04-01-string-vs-str --example 05-bare-str --features broken
cargo run -p p1-04-01-string-vs-str --example 06-returning-a-view-of-a-local --features broken
```

Then try:

1. In `01-two-shapes`, put your own name in Persian in place of "سلام دنیا". What are the length and the capacity? What's the ratio to the number of letters?
2. In `02-literals-and-conversions`, write the literal out twice and print both addresses. Are they the same? What explains that?
3. In `03-the-signature-rule`, change `byte_length`'s parameter to `&String`. Which calls break? Then run `cargo clippy`.

---

## Errors you will meet

### `E0308` — an owner where a view was wanted, and the reverse

```text
error[E0308]: mismatched types
  --> examples\04-string-where-str-wanted.rs:15:32
   |
15 |     println!("{}", byte_length(owned));
   |                    ----------- ^^^^^ expected `&str`, found `String`
   |                    |
   |                    arguments to this function are incorrect
   |
note: function defined here
  --> examples\04-string-where-str-wanted.rs:7:4
   |
 7 | fn byte_length(text: &str) -> usize {
   |    ^^^^^^^^^^^ ----------
help: consider borrowing here
   |
15 |     println!("{}", byte_length(&owned));
   |                                +

error[E0308]: mismatched types
  --> examples\04-string-where-str-wanted.rs:18:24
   |
18 |     let copy: String = "سلام";
   |               ------   ^^^^^^ expected `String`, found `&str`
   |               |
   |               expected due to this
   |
help: try using a conversion method
   |
18 |     let copy: String = "سلام".to_string();
   |                              ++++++++++++
```

**What the compiler is objecting to:** one error in each direction, and the pair of them is the lesson.

The first: the function wanted a two-word view and you handed it an owner's three words. Deref coercion only goes from `&String` to `&str`, and there was no `&` there for it to start from.

The second: the variable wanted an owner and got a literal. Here the compiler can't do anything for you, because what's needed is an allocation — and Rust doesn't allocate behind an assignment on your behalf.

**The fix:** exactly the two things it suggested.

```rust
println!("{}", byte_length(&owned));
let copy: String = "سلام".to_string();
```

**Why that's the fix:** notice how asymmetric the two `help` lines are. The first says "consider borrowing here" and wants a single `+`; the second says "try using a conversion method" and adds a whole method call. That difference in tone isn't accidental: going down is a borrow and it's free, going up is a conversion and it costs. The compiler is handing you back the same price table you read above.

### `E0277` — `str` without the `&`

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
 --> examples\05-bare-str.rs:7:22
  |
7 | fn byte_length(text: str) -> usize {
  |                      ^^^ doesn't have a size known at compile-time
  |
  = help: the trait `Sized` is not implemented for `str`
help: function arguments must have a statically known size, borrowed types always have a known size
  |
7 | fn byte_length(text: &str) -> usize {
  |                      +

error[E0277]: the size for values of type `str` cannot be known at compilation time
  --> examples\05-bare-str.rs:12:32
   |
12 |     println!("{}", byte_length(*"سلام"));
   |                                ^^^^^^^ doesn't have a size known at compile-time
   |
   = help: the trait `Sized` is not implemented for `str`
   = note: all function arguments must have a statically known size
```

**What the compiler is objecting to:** an argument has to have a definite size so the compiler knows how many bytes of stack to set aside for it. `str` means "the bytes of some text, however many there are", and that number isn't known at compile time.

That line `borrowed types always have a known size` is the heart of it: a reference is always the same size — two words here — however big the thing at the end of the pointer is.

**The fix:** the `&` it showed you with a single `+`.

```rust
fn byte_length(text: &str) -> usize {
    text.len()
}
```

**Why that's the fix:** you aren't moving the `str` or copying it; you're only looking at it. And two words is all looking takes. This is why you almost never see a bare `str` in real Rust code.

### `E0515` — returning a view of a local variable

```text
error[E0515]: cannot return reference to local variable `loud`
  --> examples\06-returning-a-view-of-a-local.rs:11:5
   |
11 |     &loud
   |     ^^^^^ returns a reference to data owned by the current function
```

**What the compiler is objecting to:** `loud` is a `String` built inside the function. At the closing `}` its owner goes and its buffer is released. The view you returned pointed at bytes that no longer exist — a dangling pointer in C, and here it simply doesn't compile.

**The fix:** hand over ownership rather than a view.

```rust
fn shout(text: &str) -> String {
    text.to_uppercase()
}
```

**Why that's the fix:** this is the second half of the signature rule, and now you've seen its reason as a real diagnostic. Text a function has just made needs an owner that outlives the function — and the only such owner available is the caller.

Notice the phrase `owned by the current function`, too. The compiler isn't talking about types; it's talking about ownership. The whole of [module 1.2](../../02-ownership-and-memory/README.md) is being examined in those three words.

### `clippy::ptr_arg` — a `&String` parameter

This one isn't an error, it's a clippy warning — so there's no example file for it, because it compiles and runs and would only leave a standing warning in the repository. Produce it yourself in a scratch file:

```rust
pub fn byte_length(text: &String) -> usize {
    text.len()
}
```

```text
warning: writing `&String` instead of `&str` involves a new object where a slice will do
 --> src\lib.rs:1:26
  |
1 | pub fn byte_length(text: &String) -> usize {
  |                          ^^^^^^^
  |
  = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.97.0/index.html#ptr_arg
  = note: `#[warn(clippy::ptr_arg)]` on by default
help: change this to
  |
1 - pub fn byte_length(text: &String) -> usize {
1 + pub fn byte_length(text: &str) -> usize {
  |
```

**What clippy is objecting to:** a `&String` gives you nothing a `&str` doesn't — it's one extra hop, and it narrows the signature. With `&String` you can no longer pass a literal; the caller has to build a `String` first. That's an allocation made purely to get past the signature.

**The fix:** `&str`, exactly as it wrote.

**Why that's the fix:** `&str` accepts the larger set of callers and does less work. `ptr_arg` is on by default, which means the clippy team doesn't consider this a matter of taste either.

---

## Exercises

### Warm up

<details>
<summary>How many words is a <code>String</code>, and what are they?</summary>

Three: pointer, length, capacity. On a 64-bit machine that's 24 bytes.

</details>

<details>
<summary>How many words is a <code>&amp;str</code>, and which word is missing?</summary>

Two: pointer and length. Capacity is the missing one — which is why it can never grow.

</details>

<details>
<summary>Why do you never write <code>str</code> without the <code>&amp;</code>?</summary>

Because its size isn't known at compile time. The compiler doesn't know how many bytes to set aside, so it won't fit in a variable, go into a function, or come back out of one. You always reach it through a pointer.

</details>

<details>
<summary>Where does the <code>"سلام"</code> you write in your code live?</summary>

Inside the executable itself. It's never allocated at run time and never freed; its type is `&'static str`.

</details>

<details>
<summary>Why does <code>byte_length(&amp;my_string)</code> compile when the function wants a <code>&amp;str</code>?</summary>

Deref coercion. The compiler sees it has a `&String` and needs a `&str`, so it inserts the `&*` for you. It only works in that direction and it never adds an allocation.

</details>

<details>
<summary>Which of these allocate: <code>.to_string()</code>, <code>.as_str()</code>, <code>.to_owned()</code>, <code>&amp;*s</code>?</summary>

`.to_string()` and `.to_owned()` do — those go up, from a view to an owner. `.as_str()` and `&*s` don't; they just put two words on the stack.

</details>

<details>
<summary>What's the signature rule, and what are its two exceptions?</summary>

Take `&str`, return `String`. The exceptions: take `String` when you're going to keep it or grow it, and return `&str` when the answer is a piece of the input rather than new text.

</details>

### Repair

Fix both errors in `examples/04-string-where-str-wanted.rs` using the compiler's own suggestions. Then fix the second one a different way — with no allocation at all. (Hint: change the type you wrote for `copy`.)

Fix `examples/05-bare-str.rs`. Then drop the `*` from the call line and see what changes; say why that `*` was there in the first place.

Fix `examples/06-returning-a-view-of-a-local.rs` **two** ways:

1. By returning a `String`.
2. By writing a function that makes no new text at all, and so really can return a view of its input.

Then say which of them is the right answer to "upper-case this" and why the other one can't be.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-04-01-string-vs-str
```

Read the signatures before anything else. Three of them take a view and hand back an owner. One deliberately takes an owner, and its test proves the caller's buffer got reused. One returns only a number and allocates nothing at all.

`joined`'s test checks the **capacity** as well as the contents, and `extended`'s checks the **buffer's address**. Both have Persian data, because the byte/letter difference is invisible in English.

### Build

Write a pair of functions that do the same job with two different signatures: `label_owned(prefix: String, name: String) -> String` and `label_borrowed(prefix: &str, name: &str) -> String`.

Both put `prefix` and then `name` one after the other. Now call each of them twice: once with two literals, once with two `String`s you already have and still need afterwards.

Then count: across those four calls, how many times was the caller forced to write `.to_string()`? And say which signature you'd publish in a real library.

Finally, find a situation where `label_owned` genuinely is the better choice. (Hint: what does the caller do with those `String`s afterwards?)

### Challenge (optional)

**Part one.** Run this and explain all four addresses:

```rust
let s = String::from("سلام");
println!("{:p}", s.as_ptr());
println!("{:p}", s.as_str().as_ptr());
println!("{:p}", (&*s).as_ptr());
println!("{:p}", (&s[..]).as_ptr());
```

That `&s[..]` is a text slice and belongs to [1.4.4](../04-slicing-text-safely/README.md) — with the full range it's always safe, but with any other range on Persian text it isn't. You'll find out why there.

**Part two.** Without running it, say what these print, then run them:

```rust
println!("{}", std::mem::size_of::<&str>());
println!("{}", std::mem::size_of::<&&str>());
println!("{}", std::mem::size_of::<Box<str>>());
println!("{}", std::mem::size_of::<Box<String>>());
```

The last two reach into [Phase 2](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md), deliberately. Compare `Box<str>` with `String`: what has it given up, and what did it get for it?

**Part three.** Run `cargo clippy` over the whole lesson. Then change `byte_length`'s signature in `src/lib.rs` to `&String` and run it again. Which tests still compile and which don't?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `String` | a three-word owner: pointer, length, capacity | text you build yourself |
| `&str` | a two-word view: pointer, length | the parameter of any function that only reads |
| `str` | the unsized type; the bytes themselves | almost never written bare |
| string literal | text inside the executable, typed `&'static str` | every `"..."` you write |
| deref coercion | the compiler writing `&*` for you | `&String` where a `&str` is wanted |
| `.as_str()` / `&*s` | a view of an owner | free, no allocation |
| `.to_string()` | "render this as text" | one allocation + copy |
| `.to_owned()` | "give me the owned form" | one allocation + copy |
| the signature rule | take `&str`, return `String` | nearly every function taking text |
| `E0515` | a view of a local variable | the sign that you should return `String` |

### What you now know

- `String` is three words and `&str` is two, and the missing third word is capacity.
- A view and its owner share one address; taking a view allocates nothing.
- `str` is unsized, so you always reach it through a pointer.
- Every string literal lives in the executable and is typed `&'static str`.
- Going up from a view to an owner always allocates; going down is always free.
- Deref coercion only unwraps a `&`, and never adds an allocation.
- Take `&str` and return `String` — unless you're keeping the buffer, or the answer is a piece of the input.

### What comes back later

- **What those bytes actually are, and why Persian takes twice the room English does** — [1.4.2 — UTF-8](../02-utf8-bytes-chars-graphemes/README.md)
- **Building and transforming text, and the `format!` macro** — [1.4.3](../03-building-and-transforming-strings/README.md)
- **Slicing text without cutting a letter in half** — [1.4.4](../04-slicing-text-safely/README.md)
- **`'static` and lifetimes done properly** — [Phase 2 — Lifetime basics](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)
- **`Deref`, `Box<str>` and the rest of the pointer story** — [Phase 2 — `Box` and heap allocation](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md)
- **`Cow`, for when you sometimes need an owner and sometimes a view** — [Phase 2 — Error handling and lifetimes](../../../phase2-intermediate/04-error-handling-and-lifetimes/README.md)

### Can you explain?

- What words is a `String` made of, and what words is a `&str` made of?
- Why do you never write `str` bare?
- Where does a string literal live, and who frees it?
- Why does `&String` work where a `&str` is wanted, but not the other way round?
- Which conversions allocate and which don't?
- State the signature rule, and say when you shouldn't follow it.

---

## Going further

- [The Rust Book — Strings](https://doc.rust-lang.org/book/ch08-02-strings.html) — the same ground, officially.
- [`std::string::String`](https://doc.rust-lang.org/std/string/struct.String.html) and [`std::primitive::str`](https://doc.rust-lang.org/std/primitive.str.html) — the two reference pages. Scroll each method list once so you know what lives where.
- [`clippy::ptr_arg`](https://rust-lang.github.io/rust-clippy/master/#ptr_arg) — the lint you met above, with more examples.
