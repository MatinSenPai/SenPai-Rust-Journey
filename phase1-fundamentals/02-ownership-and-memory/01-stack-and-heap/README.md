# 1.2.1 — Stack and heap

## At a glance

After this lesson you can:

- Say where any value in your program lives, and why it's there.
- Explain why a `Vec<i32>` is always 24 bytes, whether it's empty or holds a million items.
- Name the three classic memory bugs and say how different languages deal with them.
- Say why Rust needed a thing called "ownership" — that it's the answer to a specific question, not an arbitrary strictness.

**Time:** ~45 minutes · **Prerequisites:** [1.1.6 — `Vec` and `String` basics](../../01-foundations/06-vec-and-string-basics/README.md)

---

## Why this matters

This lesson teaches no new syntax. It builds a mental model, and without that model the rest of Rust looks like a set of arbitrary rules a bad-tempered compiler put in your way.

With it, those same rules turn into the only possible answers to a very specific question.

The question is: **when does memory you took at run time get given back?**

There are three historical answers, and you've met all three even if you didn't know their names:

| Language | Answer | The price |
|---|---|---|
| C, older C++ | you call `free` yourself | you forget, or call it twice, or use it after freeing |
| Python, Java, Go, JS | a garbage collector works it out later | unpredictable pauses, extra memory, less control |
| Rust | the compiler knows when, and writes it for you | you have to prove to the compiler when |

That third row is the whole of Rust. "Ownership", "borrowing" and "lifetimes" are all tools for that proof. This lesson shows you what the proof is proving.

---

## The concept

### Two regions of memory

Your running program divides its memory into regions. Two of them matter here.

**The stack** is a stack of plates. When a function is called, a **frame** goes on it: room for all of that function's local variables. When the function returns, the frame comes off. That's it.

It's fast, and it's fast because it does almost nothing: it moves one number up and down. But it has one constraint, and everything follows from it — **each frame's size must be known at compile time.** The compiler has to know how far to move that number.

**The heap** is a big warehouse. You can ask for any amount of space at any time and give it back in any order. That's flexible, and for the same reason slower: something has to keep track of the free space, and that something is an allocator doing real work.

### Which values go where

```text
i32:            4 bytes
i64:            8 bytes
bool:           1 bytes
char:           4 bytes
usize:          8 bytes
():             0 bytes

[i32; 5]:       20 bytes
[i32; 100]:     400 bytes
```

Every one of these has a size known at compile time, so every one of them sits on the stack. Arrays included — and now you can see why an array's length was part of its type: **without it, the compiler wouldn't know how much frame to reserve.**

(And `()` being zero bytes is worth a smile: the unit type genuinely takes up no space.)

Now the interesting ones:

```text
Vec<i32>:       24 bytes
Vec<i64>:       24 bytes
String:         24 bytes
&str:           16 bytes
&i32:           8 bytes
```

`Vec<i32>` and `Vec<i64>` are both 24. `String` is 24 as well. And 24 is exactly `3 * size_of::<usize>()`:

```text
3 x usize:      24 bytes
```

**Three machine words:**

| word | holds |
|---|---|
| 1 | a pointer — where the data is on the heap |
| 2 | length — how many items |
| 3 | capacity — how many there's room for |

And the proof:

```text
empty vec:      24 bytes
1000-item vec:  24 bytes
its contents:   4000 bytes
```

A thousand-item `Vec` is still 24 bytes. Those four thousand bytes are somewhere else.

### See that "somewhere else"

You can print a memory address. The numbers differ every run — the operating system puts your program somewhere new each time — but what stays true is which ones are near each other:

```text
stack values
  a:          0x479b76f748
  b:          0x479b76f74c
  c:          0x479b76f750

their headers, also on the stack
  numbers:    0x479b76f7d8
  text:       0x479b76f7f0

what they point at, on the heap
  numbers:    0x2454881a350
  text:       0x24548811e20
```

The first five addresses all start `0x479b76f7` — all in `main`'s frame, within a few dozen bytes of each other. The last two start `0x24548`, somewhere else entirely.

**So a `Vec` is on the stack *and* on the heap.** The variable itself — those three words — is on the stack. What it points at is on the heap.

### Why that creates a problem

A stack frame comes off automatically when the function returns. Nobody has to do anything.

A heap block doesn't. It has to be released one day, and **it has to be released exactly once.** There are three possible mistakes and all three have names:

| Mistake | What happens |
|---|---|
| **memory leak** — never free it | the program grows without bound |
| **double free** — free it twice | the allocator's internal structures are corrupted |
| **use after free** — read it after freeing | you read whatever's there now — or an attacker writes it |

The third is the worst, and it's one of the two or three main sources of security vulnerabilities in systems software. Microsoft and Google have both reported that around **70%** of their serious security vulnerabilities are memory errors, and a large share of that is precisely this.

### Three solutions, and which one Rust took

**C says be careful.** You `malloc`, you `free`, and getting it right is your job. The fastest approach and the most error-prone. We've been paying for it for fifty years.

**Python, Java and Go add a garbage collector.** A piece of the runtime periodically sweeps and frees whatever is no longer reachable. It nearly always works, and the price is: the runtime must always be present, memory use is higher than necessary, and the program sometimes stops for pauses you don't control. For a web service that's usually fine. For an OS kernel or an audio driver it isn't.

**Rust took a third route: the compiler proves when.**

The rule is one sentence:

> **Every value on the heap has exactly one owner. When the owner goes out of scope, the value is freed.**

No garbage collector, no runtime, no pauses. The compiler writes the free call at the right place while it's building the code. **It costs nothing at run time** — because all the work happened at compile time.

### Scope, precisely

"Going out of scope" means reaching the closing brace of the block the value was declared in:

```rust
let outer = String::from("I last until the end of main");

{
    let inner = String::from("I last until the next brace");
    println!("inside:     {inner}");
} // <- inner ends here
```

```text
start:      I last until the end of main
inside:     I last until the next brace
also here:  I last until the end of main
after:      I last until the end of main
```

`inner` exists, then it doesn't. Its heap buffer is released at exactly that brace — not later, not on some subsequent collection cycle. **Right then.**

Scopes nest, and things are released in reverse order of declaration: the last thing built is the first thing to go. Exactly the stack-of-plates discipline.

### Capacity, and why it doubles

You saw this in [1.1.6](../../01-foundations/06-vec-and-string-basics/README.md):

```text
pushes:    0  1  2  3  4  5  6  7  8  9 ... 16 17
capacity:  0  4  4  4  4  8  8  8  8 16 ... 16 32
```

Now you can see why. Every time the capacity runs out, the `Vec` has to ask the allocator for a bigger block and move everything across. That's expensive, so it has to happen rarely.

If it grew by one each time: `n` pushes would be `n` allocations and `n²/2` copies. Doubling makes it about `log₂(n)` allocations and fewer than `2n` copies. That's why `push` is called "amortised O(1)" — any individual one may be expensive, but the average over any run is constant.

And if you know how many are coming, you can skip all of it:

```rust
let sized: Vec<i32> = Vec::with_capacity(100);
```

```text
reserved:  0 / 100
```

### Reference table

| Value | Where it is | Where its data is |
|---|---|---|
| `i32`, `bool`, `char` | stack | right there |
| `[i64; 10]` | stack (80 bytes) | right there |
| `Vec<T>` | stack (24 bytes) | heap |
| `String` | stack (24 bytes) | heap |
| `&str` to a literal | stack (16 bytes) | inside the executable |
| `&T` | stack (8 bytes) | wherever the `T` is |

---

## Hands on

```sh
cargo run -p p1-02-01-stack-and-heap --example 01-sizes
cargo run -p p1-02-01-stack-and-heap --example 02-where-things-live
cargo run -p p1-02-01-stack-and-heap --example 03-scope
```

Run `02-where-things-live` **several times**. The addresses change every run; the gap between the two groups stays.

Then the broken one:

```sh
cargo run -p p1-02-01-stack-and-heap --example 04-out-of-scope --features broken
```

Then try:

1. In `01-sizes`, print the size of `[u8; 1_000_000]`. Then `Vec<u8>`. Explain the difference.
2. In `01-sizes`, print `size_of::<Option<i32>>()` and then `size_of::<Option<Box<i32>>>()`. Why is the second smaller than you'd expect? (The answer is in Phase 2 — for now just look at the number.)
3. In `02-where-things-live`, make another `Vec` and print its heap address. Is it near the first one or far away?

---

## Errors you will meet

### `E0425` — a name that no longer exists

```text
error[E0425]: cannot find value `inner` in this scope
  --> examples\04-out-of-scope.rs:13:25
   |
13 |     println!("outside: {inner}");
   |                         ^^^^^
   |
help: the binding `inner` is available in a different scope in the same function
  --> examples\04-out-of-scope.rs:7:13
   |
 7 |         let inner = String::from("I only exist inside these braces");
   |             ^^^^^
```

**What the compiler is objecting to:** `inner` was declared inside that block and the block has ended. Not just its value — the *name* doesn't exist out here either.

**The fix:** either move the declaration to the outer scope, or do whatever you needed with it inside the block.

**Why that's the fix:** look at that `help`, because it's more useful than it looks. The compiler isn't saying "no such thing"; it's saying "it exists, but elsewhere", and pointing at exactly where. Which means you haven't mistyped the name — **this is a scope problem, not a spelling one.**

And this error shares a root with the whole of module 1.2: `inner` died at that brace because its **owner** ended there. Rust won't keep a name around for you that refers to freed memory — so the name goes with the value.

---

## Exercises

### Warm up

<details>
<summary>Why does an array live on the stack when a <code>Vec</code> doesn't?</summary>

Because an array's length is part of its type, so the compiler knows the frame size. A `Vec`'s length isn't known until run time, so its contents have to live somewhere that can be sized at run time.

</details>

<details>
<summary>What is <code>size_of::&lt;Vec&lt;i32&gt;&gt;()</code>, and does it change with the contents?</summary>

24 on a 64-bit machine, and no. It's three words: pointer, length, capacity. The contents are elsewhere.

</details>

<details>
<summary>Name the three memory mistakes.</summary>

Leak (never freeing), double free (freeing twice), and use after free (reading after freeing). The third is the dangerous one.

</details>

<details>
<summary>How does Python solve this, and what does it cost?</summary>

With a garbage collector. It costs a runtime that must always be present, more memory than strictly needed, and pauses you don't control.

</details>

<details>
<summary>What is Rust's answer, in one sentence?</summary>

Every heap value has exactly one owner, and when the owner goes out of scope the value is freed — and the compiler writes that in while it builds the code.

</details>

<details>
<summary>What does "going out of scope" mean?</summary>

Reaching the closing brace of the block the value was declared in. It's freed there and then, not later.

</details>

### Repair

Fix `examples/04-out-of-scope.rs` two different ways:

1. Move `inner` to the outer scope.
2. Leave `inner` where it is and move the `println!` inside the block instead.

Both compile. Say which is better and why — and your answer should contain the word "scope".

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-02-01-stack-and-heap
```

None of them computes anything interesting. All five are measurements, and the point is that you see the numbers yourself instead of trusting a diagram.

Watch the last test: `total_bytes` must count **capacity**, not length. If you wrote `len()` and it failed, that failure is the lesson.

### Build

Write a `pub fn footprint_of_lines(lines: Vec<String>) -> usize` returning every byte that collection is responsible for: the `Vec` itself, plus each `String`'s three words, plus each one's text buffer.

Then run it on `vec!["a".to_string(); 1000]` and compare the answer with `1000`.

Then write a sentence on why a thousand one-letter strings take up so much more than a thousand bytes. If you understand the answer, you understand why a `Vec<String>` is an expensive data structure for short pieces of text.

### Challenge (optional)

**Part one.** Run this and explain it:

```rust
let mut v: Vec<u8> = Vec::with_capacity(1_000_000);
println!("{} / {}", v.len(), v.capacity());
v.push(1);
v.clear();
println!("{} / {}", v.len(), v.capacity());
v.shrink_to_fit();
println!("{} / {}", v.len(), v.capacity());
```

What did `clear()` release, and what did it not?

**Part two.** Write a function that declares a very large array on the stack — say `[u8; 100_000_000]` — and call it. What happens? Now take the same amount in a `Vec`. Why does one work and the other not?

**Part three.** Look at the addresses in `02-where-things-live`. The stack addresses differ every run. Search for **ASLR**. Why does the operating system do that on purpose, and which of the three memory mistakes does it relate to?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| stack | function frames, sizes known at compile time | every local variable |
| heap | run-time allocation of any size | `Vec`, `String`, anything growable |
| frame | room for one function call's variables | removed on return |
| a `Vec`'s three words | pointer, length, capacity | why `size_of` is always 24 |
| capacity | reserved heap room | `with_capacity`, `shrink_to_fit` |
| amortised O(1) | occasionally expensive, cheap on average | why `push` is dependable |
| memory leak | never freed | unbounded growth |
| double free | freed twice | allocator corruption |
| use after free | read after freeing | security vulnerability |
| scope | up to the closing brace | where a value dies |
| owner | the one binding responsible for freeing | the whole of module 1.2 |

### What you now know

- The stack is fast and fixed-size; the heap is flexible and costlier.
- `Vec` and `String` are both three words on the stack with their data on the heap.
- A type's size is known at compile time — which is why an array's length is part of its type.
- There are three memory mistakes and they all come from one question: when do we free it?
- Languages have given three different answers, and Rust gave the third: the compiler proves it.
- A value dies at the closing brace of its scope, right then.

### What comes back later

- **What it actually means for a value to "move"** — [1.2.2 — Move semantics](../02-move-semantics/README.md)
- **When it's copied and when it isn't** — [1.2.3 — `Clone` and `Copy`](../03-clone-and-copy/README.md)
- **Code that runs when a value is freed** — [1.2.5 — `Drop` and RAII](../05-drop-and-raii/README.md)
- **Looking without taking ownership** — [1.3.1 — References](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
- **Putting something on the heap deliberately** — [Phase 2 — `Box` and heap allocation](../../../phase2-intermediate/05-smart-pointers/01-box-and-heap-allocation/README.md)
- **When one owner isn't enough** — [Phase 2 — `Rc` and `Arc`](../../../phase2-intermediate/05-smart-pointers/02-rc-and-arc/README.md)

### Can you explain?

- What's the difference between the stack and the heap, and why is the stack faster?
- Why doesn't `size_of::<Vec<i32>>()` change with the contents?
- What are those three words?
- Name the three memory mistakes, and say which is worst.
- What does a garbage collector solve and what does it cost?
- State Rust's ownership rule in one sentence.

---

## Going further

- [The Rust Book — What Is Ownership?](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html) — the same ground, officially, and it goes straight on into the next lesson.
- [`std::mem::size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html) — this lesson's measuring tool.
- [Memory safety in Chrome](https://www.chromium.org/Home/chromium-security/memory-safety/) — Google's report that around 70% of their serious security bugs are memory errors. Short, and the best practical justification for this whole module.
