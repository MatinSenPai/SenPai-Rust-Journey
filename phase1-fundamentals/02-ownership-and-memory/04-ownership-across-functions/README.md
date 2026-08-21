# 1.2.4 — Ownership across functions

## At a glance

After this lesson you can:

- Say what happens when you pass a `String` to a function, and why it isn't yours afterwards.
- Write a function that takes ownership, and one that creates ownership and gives it away.
- Say what `fn f(x: String)` and `fn f(x: &str)` each demand of the caller.
- Say why the "take it and give it back" pattern works and why nobody writes it.

**Time:** ~40 minutes · **Prerequisites:** [1.2.3 — `Clone` and `Copy`](../03-clone-and-copy/README.md)

---

## Why this matters

There is no new rule in this lesson.

Passing a value to a function is exactly what `let b = a;` was: a move. Returning it is the same thing in the other direction. That's all. The language has no separate concept for arguments.

So why a whole lesson? Because **this is where ownership stops being an idea and starts being a decision in every line you write.** Every signature you write is a demand on the caller, and until now you have never had to choose how much to demand.

And one more thing: this lesson deliberately walks into a dead end. With the tools you have so far, a function that only wants to *read* something has to take ownership of it and then give it back. It works and it's ugly. **That ugliness is the argument for the next module**, and it's better to feel the problem before you're shown the solution.

---

## The concept

### Passing is a move

```rust
let name = String::from("Matin");
let length = consume(name);
```

```text
length:     5
```

`consume` takes a `String` by value, so this call **moves** `name` into it. After that line `name` is no longer a valid binding.

Nothing new here. Writing `let moved = name;` would have done the same thing. A function's parameter is just another binding for the value to move into.

And the same exception holds:

```rust
let count = 5_i32;
let doubled = double(count);
println!("count:      {count}");
```

```text
count:      5
doubled:    10
```

`count` still works afterwards, because `i32` is `Copy` and owns nothing.

### A function that swallows a value

```rust
fn consume(text: String) -> usize {
    text.len()
} // <- `text` is dropped here
```

That closing brace matters. `text` belongs to this function, the function ends, so `text` is dropped and its heap buffer freed — **inside `consume`**, not in `main`.

```rust
let temporary = String::from("this will not survive the call");
consume(temporary);
```

```text
that String was freed inside `consume`
```

Not a leak, and not an accident. The function became the owner, so the function became responsible for freeing it.

### A function that creates ownership

```rust
fn build() -> String {
    String::from("made inside build()")
}
```

```text
built:      made inside build()
built   @:  0x269ef3ca270
```

There is no ceremony here, and that's the point. The value is made inside the function and its ownership moves out. **Every new value in your program comes from a function shaped like this.**

And you can of course do both: take one in, hand a different one back.

```rust
fn shout(text: String) -> String {
    text.to_uppercase()
}
```

### And now the dead end

Suppose you want a string's length **and** you want to keep the string. With this module's tools, the function has to give it back:

```rust
fn measure_and_return(text: String) -> (usize, String) {
    let length = text.len();
    (length, text)
}
```

```rust
let (length, name) = measure_and_return(name);
```

```text
length:     5
still ours: Matin
```

It works. Now call it twice:

```rust
let (length, name) = measure_and_return(name);
let (bytes, name) = measure_and_return(name);
```

```text
again:      5 5 Matin
```

A fresh tuple, a fresh destructuring and a fresh binding each time, for a function that only wanted to read.

And with two arguments it stops being tolerable:

```rust
fn total_length(first: String, second: String) -> (usize, String, String) {
    let total = first.len() + second.len();
    (total, first, second)
}
```

```text
total:      9
both back:  alpha beta

every one of those returns exists only to give the value back
```

Three values in, three out, for one addition.

> **This code works and no Rust programmer writes it.** Every one of those return values is there purely to hand the value back. The next module deletes exactly this, and you have already seen its answer in the signatures: `&`.

### What a signature says

Three shapes, three different demands on the caller:

```rust
fn by_view(text: &str) -> usize { text.len() }
fn by_value(mut text: String) -> String { text.push('!'); text }
```

```text
by_view:    5
by_view:    9
still ours: hello
by_value:   goodbye!
```

| Signature | What it demands | When it's right |
|---|---|---|
| `&str` | "I only want to look; keep it" | reading — your default |
| `String` | "I need to keep it or change it" | storing, mutating, returning a changed version |
| `&String` | almost never | it's `&str` with an extra restriction |

That last row is worth explaining: a function taking `&String` won't accept a string literal — the caller has to build a `String` first. A function taking `&str` accepts both. **`&String` buys nothing and excludes a whole class of caller.**

And the cost of choosing wrong is real:

```rust
println!("wasteful:   {}", by_value(owned.clone()));
```

```text
wasteful:   hello!
still ours: hello
```

Because `by_value` demanded ownership and the caller wanted to keep its value, the caller had to clone. An allocation, purely because a signature asked for more than it needed.

> **The rule to take away:** ask for the smallest thing that does the job. If you're only reading, take `&str`.

---

## Hands on

```sh
cargo run -p p1-02-04-ownership-across-functions --example 01-passing-moves
cargo run -p p1-02-04-ownership-across-functions --example 02-giving-it-back
cargo run -p p1-02-04-ownership-across-functions --example 03-what-the-signature-says
```

Then the two broken ones:

```sh
cargo run -p p1-02-04-ownership-across-functions --example 04-used-after-passing --features broken
cargo run -p p1-02-04-ownership-across-functions --example 05-returning-a-local-reference --features broken
```

Then try:

1. In `01-passing-moves`, uncomment the `println!("{name}")` line. Read the whole error — it has a `note` pointing straight at the next module.
2. In `03-what-the-signature-says`, change `by_view` to take `&String`. Which call breaks now?
3. In `02-giving-it-back`, write a three-argument function that returns all three. How far do you get before it becomes intolerable?

---

## Errors you will meet

### `E0382` — used after passing

```text
error[E0382]: borrow of moved value: `name`
  --> examples\04-used-after-passing.rs:12:25
   |
 6 |     let name = String::from("Matin");
   |         ---- move occurs because `name` has type `String`, which does not implement the `Copy` trait
 7 |
 8 |     println!("length:  {}", consume(name));
   |                                     ---- value moved here
...
12 |     println!("name:    {name}");
   |                         ^^^^ value borrowed here after move
   |
note: consider changing this parameter type in function `consume` to borrow instead if owning the value isn't necessary
  --> examples\04-used-after-passing.rs:15:18
   |
15 | fn consume(text: String) -> usize {
   |    -------       ^^^^^^ this parameter takes ownership of the value
   |    |
   |    in this function
```

**What the compiler is objecting to:** the familiar `E0382`, except this time the move was a function call.

**The fix:** look at that `note`, because the compiler is telling you the right answer: *"consider changing this parameter type in function `consume` to borrow instead if owning the value isn't necessary"*.

**Why that's the fix:** rustc is separating the culprit from the victim. The problem isn't where the error is shown; it's **in `consume`'s signature.** That function demanded ownership and then only called `.len()`. Note that it didn't even suggest `.clone()` here — because that isn't the right answer.

That `note` is effectively module 1.3's introduction, written by the compiler.

### `E0106` — returning a reference to a local

This is the error you get when you try to invent the way out yourself.

```text
error[E0106]: missing lifetime specifier
  --> examples\05-returning-a-local-reference.rs:11:23
   |
11 | fn make_greeting() -> &String {
   |                       ^ expected named lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
help: consider using the `'static` lifetime, but this is uncommon unless you're returning a borrowed value from a `const` or a `static`
   |
11 | fn make_greeting() -> &'static String {
   |                        +++++++
help: instead, you are more likely to want to return an owned value
   |
11 - fn make_greeting() -> &String {
11 + fn make_greeting() -> String {
   |
```

**What the compiler is objecting to:** that `String` was made inside the function and is dropped at the end of it. A reference to it would point at memory that has just been freed — the use-after-free from [1.2.1](../01-stack-and-heap/README.md).

**The fix:** write `-> String` and give the ownership away, exactly as the compiler's second suggestion says.

**Why that's the fix:** look at the key phrase in the `help`: *"there is no value for it to be borrowed from"*. The problem isn't that you wanted to lend something out; it's that **there is nothing to borrow it from**. The value was born here and dies here.

That `'static` suggestion is the first lifetime you've seen, and the compiler itself says it's almost always the wrong answer. Lifetimes are [Phase 2](../../../phase2-intermediate/04-error-handling-and-lifetimes/README.md). For now the practical lesson is: **if you made the value, give the ownership away.**

---

## Exercises

### Warm up

<details>
<summary>What does passing a <code>String</code> to a function do?</summary>

Moves it, exactly like an assignment. After the call the caller's binding is no longer valid.

</details>

<details>
<summary>When is a <code>String</code> passed to a function and not returned freed?</summary>

At that function's closing brace. The function became the owner, so it's responsible for freeing it.

</details>

<details>
<summary>What do <code>fn f(x: String)</code> and <code>fn f(x: &amp;str)</code> demand of the caller?</summary>

The first wants them to give their value up. The second only wants to look, and the caller keeps it — and it accepts a string literal with no conversion at all.

</details>

<details>
<summary>Why is <code>&amp;String</code> in a parameter almost always wrong?</summary>

Because it's `&str` with an extra restriction: it won't take a literal, so the caller has to build a `String` first. It buys nothing and excludes callers.

</details>

<details>
<summary>Why can't you return a reference to a <code>String</code> a function made?</summary>

Because that `String` is dropped at the end of the function, so the reference would point at freed memory. Give the ownership away instead.

</details>

<details>
<summary>When an <code>E0382</code> comes from a function call, where do you look first?</summary>

That function's signature. If it took ownership and is only reading, the signature is at fault rather than the call site — and the compiler says so in a `note`.

</details>

### Repair

Fix `examples/04-used-after-passing.rs` **two** ways:

1. By cloning at the call site.
2. By changing `consume`'s signature, as the `note` suggests.

Which is better, and how many allocations does each save?

Then fix `examples/05-returning-a-local-reference.rs`. The compiler offers two routes — take the one it says is more likely, and say why the other is wrong here.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-02-04-ownership-across-functions
```

Four take ownership and one only borrows. That one's test does something the others cannot — **find it.** That difference is the whole argument for module 1.3.

### Build

Write a `pub fn shortest_of(values: Vec<String>) -> String` returning the shortest string (by bytes; first one wins a tie). `values` is never empty.

Then answer: what happened to the other strings? At what point in your function were they freed?

Then write a version that returns `values` as well, and say why that version is so much more unpleasant.

### Challenge (optional)

**Part one.** Does this compile? If not, which line?

```rust
fn main() {
    let text = String::from("hello");
    let length = measure(&text);
    let owned = consume(text);
    println!("{length} {owned}");
}

fn measure(text: &String) -> usize { text.len() }
fn consume(text: String) -> usize { text.len() }
```

Then change `&String` to `&str` and see what changes at the call site — or doesn't.

**Part two.** Write this function and explain why it's allowed:

```rust
fn hand_it_back(text: String) -> String {
    text
}
```

How many bytes were copied? How many allocations happened? Now call it and print `as_ptr()` before and after.

**Part three.** Run this and explain the order of the output. (Hint: when are values given to a function dropped?)

```rust
fn main() {
    let a = String::from("a");
    let b = String::from("b");
    println!("before");
    eat(a);
    println!("middle");
    eat(b);
    println!("after");
}

fn eat(text: String) {
    println!("  eating {text}");
}
```

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| passing by value | a move into the function | `fn f(x: String)` |
| returning ownership | a move out of the function | every constructor |
| consuming | the function owns it and drops it | `fn f(x: String) -> usize` |
| the "give it back" pattern | the value returns in a tuple | what you must write without borrowing |
| `&str` parameter | "I'll only look" | the default for reading text |
| `String` parameter | "I need to keep it" | storing or mutating |
| `&String` parameter | almost never | `&str` with an extra restriction |
| `E0106` | a reference to something that won't last | returning `&` to a local |

### What you now know

- Passing a value to a function is the same move as an assignment, with no new rules.
- A value consumed and not returned is freed at the end of that function.
- Returning gives ownership away, and that's where new values come from.
- Every signature is a demand; ask for the smallest thing that does the job.
- A `&String` parameter should almost always be `&str`.
- You cannot return a reference to a value the function just made.

### What comes back later

- **Borrowing, which deletes the whole "give it back" pattern** — [1.3.1 — References](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)
- **The rules that keep borrowing safe** — [1.3.2](../../03-borrowing-and-references/02-borrow-checker-rules/README.md)
- **Code that runs when a value is dropped** — [1.2.5 — `Drop`](../05-drop-and-raii/README.md)
- **`'static` and real lifetimes** — [Phase 2](../../../phase2-intermediate/04-error-handling-and-lifetimes/README.md)

### Can you explain?

- What does passing a `String` to a function do?
- When is a consumed, unreturned value freed?
- What do `fn f(x: String)` and `fn f(x: &str)` demand of the caller?
- Why is `&String` in a parameter almost always wrong?
- Why can't you return a reference to a local?
- What is the "give it back" pattern and why does nobody write it?

---

## Going further

- [The Rust Book — Ownership and functions](https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html#ownership-and-functions) — the same ground, officially.
- [Rust API Guidelines — Flexibility](https://rust-lang.github.io/api-guidelines/flexibility.html) — this same parameter-type choice, from a library-design point of view.
- [`rustc --explain E0106`](https://doc.rust-lang.org/error_codes/E0106.html) — short, and comprehensible now.
