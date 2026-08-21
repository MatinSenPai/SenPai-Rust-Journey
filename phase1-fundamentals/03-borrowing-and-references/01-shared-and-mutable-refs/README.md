# 1.3.1 — Shared and mutable references

## At a glance

After this lesson you can:

- Write a function that reads a value without taking it, and say why the caller still owns it afterwards.
- Choose between `T`, `&T` and `&mut T` in a signature, and say what each one demands of the caller.
- Read and fix `E0596`, and explain why `&mut` has to ask the owner first.
- Say where `*` is required and where the method dot does the work for you.

**Time:** ~45 minutes · **Prerequisites:** [1.2.4 — Ownership across function boundaries](../../02-ownership-and-memory/04-ownership-across-functions/README.md)

---

## Why this matters

The last lesson ended on an ugly pattern. You wanted the length of a `Vec<String>` *and* you wanted to keep the Vec, and the only tool available was to make the function hand it back:

```rust
fn total_length(lines: Vec<String>) -> (usize, Vec<String>) {
    let mut total = 0;
    for line in &lines {
        total += line.len();
    }
    (total, lines)
}

let (total, lines) = total_length(lines);
let (again, lines) = total_length(lines);
println!("total:       {total}");
println!("again:       {again}");
println!("still ours:  {}", lines.len());
```

```text
total:       14
again:       14
still ours:  3
```

The answer is right and nobody writes this. Every `Vec` that was only going to be read adds a return value to the signature, and with two arguments the signature falls apart.

If you're coming from Python this looks strange from the other direction: over there, handing a list to a function never takes it away from you. A reference is always what gets passed. Rust has the same thing — it just makes you write it in the signature, and it puts rules around it.

**Where the Python analogy stops:** in Python any reference can mutate the object and nothing stops you, and a reference count decides when the object dies. In Rust a reference owns nothing, it is checked at compile time, and whether it may write is part of its *type*. That far the analogy holds and no further.

---

## The concept

Borrowing means using a value without taking it. The tool is one character: `&`.

```senpai-visual
{"kind":"borrowing","labels":["owner","shared borrow","mutable borrow","borrow ends"]}
```

### Lending instead of handing over

```rust
fn total_length(lines: &Vec<String>) -> usize {
    let mut total = 0;
    for line in lines {
        total += line.len();
    }
    total
}

let lines = vec![String::from("alpha"), String::from("beta"), String::from("gamma")];
println!("total:       {}", total_length(&lines));
println!("still ours:  {}", lines.len());
```

```text
total:       14
still ours:  3
```

`&lines` doesn't move anything. It makes a **reference**: an arrow to `lines`, not `lines` itself. The function reads through the arrow, the arrow ends, and ownership never moved at all.

Look at the signature: one return value, not two. The `Vec` the previous version handed back had never been taken in the first place.

> A reference owns nothing. When it ends, nothing is freed — dropping is the one owner's job, exactly as [1.2.5](../../02-ownership-and-memory/05-drop-and-raii/README.md) described.

### The arrow points at the caller's value

```rust
let view = &lines;
println!("lines   @:   {:p}", lines.as_ptr());
println!("view    @:   {:p}", view.as_ptr());
```

```text
lines   @:   0x1b720b98d30
view    @:   0x1b720b98d30
```

One address, not two. Borrowing allocates nothing and copies no bytes; it makes an arrow, which is one machine word. That's why passing a `&` to a function is effectively free no matter how much data is behind it.

### `*` — following the arrow by hand

```rust
let answer = 42;
let view = &answer;
println!("through *:     {}", *view + 1);
```

```text
through *:     43
```

`*` is a **dereference**: "the value at the end of this arrow". `view` is a `&i32`; `*view` is an `i32`.

For arithmetic Rust has a second route — `view + 1` compiles too, because the standard library implements `+` for references as well. But for **assignment** there is no way around it: `*counter = 0` is the only spelling that works, and the wrong one is the second error in the next section.

### The dot follows the arrow for you

```rust
let text = String::from("hello");
let look = &text;
println!("look.len():    {}", look.len());
println!("(*look).len(): {}", (*look).len());
```

```text
look.len():    5
(*look).len(): 5
```

Two identical lines. When you call a method, the compiler inserts as many `*` as it takes; the name for that is **auto-deref**. It's why you see so few stars in real Rust: most of what you do with a reference is call a method on it.

`{}` in `println!` follows arrows too, however many are stacked up:

```rust
let arrow_to_arrow = &look;
println!("two arrows:    {arrow_to_arrow}");
println!("still counts:  {}", arrow_to_arrow.len());
```

```text
two arrows:    hello
still counts:  5
```

### A shared reference is `Copy`

```rust
let first = &text;
let second = first;
println!("both arrows:   {first} / {second}");
println!("one buffer:    {}", first.as_ptr() == second.as_ptr());
```

```text
both arrows:   hello / hello
one buffer:    true
```

You met this in [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md) and now it pays off: `&T` is `Copy`, so assigning one isn't a move. The arrow was copied, not the thing at the end of it — and that `true` says both arrows still point at one buffer.

`&mut T` is the one kind of arrow that is **not** `Copy`. Why is [1.3.2](../02-borrow-checker-rules/README.md)'s subject.

### `&mut T` — an arrow you may write through

```rust
fn add_exclamation(text: &mut String) {
    text.push('!');
}

let mut greeting = String::from("hello");
add_exclamation(&mut greeting);
println!("after one:     {greeting}");
add_exclamation(&mut greeting);
add_exclamation(&mut greeting);
println!("after three:   {greeting}");
```

```text
after one:     hello!
after three:   hello!!!
```

`add_exclamation` returns nothing, because it has nothing to give back: the `String` that changed belonged to `main` the whole time. A **mutable reference** deletes the give-it-back pattern outright.

### `&mut` needs the owner's permission first

```rust
fn bump(counter: &mut i32) {
    *counter += 1;
}

let mut count = 10;
bump(&mut count);
bump(&mut count);
println!("count:         {count}");
```

```text
count:         12
```

The `mut` in `let mut count` is not optional. To make a `&mut count`, the owner must itself be mutable — you can't lend a permission you don't hold. Take it away and you get `E0596`, which is `04-owner-not-mut` and the next section.

And note it's `*counter += 1`, not `counter += 1`. That's an assignment, so the star is compulsory.

### Two `&mut` at once, to two different values

```rust
fn move_one(from: &mut i32, to: &mut i32) {
    *from -= 1;
    *to += 1;
}

let mut here = 5;
let mut there = 0;
move_one(&mut here, &mut there);
move_one(&mut here, &mut there);
println!("moved:         {here} {there}");
```

```text
moved:         3 2
```

Two mutable references alive at the same instant, and the compiler is quiet. The rule is about **one value**, not about counting arrows: `here` and `there` are two separate things.

### Borrowing a collection, element by element

```rust
fn double_all(values: &mut Vec<i32>) {
    for value in values {
        *value *= 2;
    }
}

let mut values = vec![1, 2, 3];
let buffer = values.as_ptr();
double_all(&mut values);
println!("doubled:       {values:?}");
println!("same buffer:   {}", buffer == values.as_ptr());
```

```text
doubled:       [2, 4, 6]
same buffer:   true
```

`for value in values` over a `&mut Vec<i32>` hands you a `&mut i32` each turn — so `value` is itself an arrow, and writing through it needs `*value`. The same loop over a `&Vec<i32>` gives you a `&i32` each turn, which is read-only.

And that `true` matters: it was the caller's own buffer that changed. Nothing was copied out and nothing was copied back.

### Many readers, or one writer

```rust
let a = &lines;
let b = &lines;
let c = &lines;
println!("three readers: {} {} {}", a.len(), b.len(), c.len());
```

```text
three readers: 3 3 3
```

Any number of `&T` may be alive at once, because none of them can change anything. Many readers is always safe.

The full rule is one sentence, and it is the whole of the next lesson:

> **At any moment a value may have any number of `&T` pointing at it, or exactly one `&mut T` — never both.**

That's enough of it for today. How the compiler actually measures this, which errors it produces, and why the rule makes a data race impossible, is [1.3.2](../02-borrow-checker-rules/README.md).

### Which one belongs in the signature

Three choices, three different demands on the caller:

| In the signature | It means | The caller afterwards |
|---|---|---|
| `&T` | I only want to look | owns it, everything unchanged |
| `&mut T` | I want to change yours | owns it, with a changed value |
| `T` | I need to own it | doesn't have it any more |

`&T` is the default. `&mut T` when you genuinely have to change something. `T` only when the function has to keep the value, store it, or return a transformed version of it — which is exactly what [1.2.4](../../02-ownership-and-memory/04-ownership-across-functions/README.md) said.

And that `-> (usize, Vec<String>)` pattern? Retired. From here on, every time you see a function returning something purely in order to give it back, a `&` is missing.

---

## Hands on

```sh
cargo run -p p1-03-01-shared-and-mutable-refs --example 01-look-dont-take
cargo run -p p1-03-01-shared-and-mutable-refs --example 02-through-a-mutable-ref
cargo run -p p1-03-01-shared-and-mutable-refs --example 03-following-the-arrow
```

Then the three broken ones:

```sh
cargo run -p p1-03-01-shared-and-mutable-refs --example 04-owner-not-mut --features broken
cargo run -p p1-03-01-shared-and-mutable-refs --example 05-forgot-the-star --features broken
cargo run -p p1-03-01-shared-and-mutable-refs --example 06-taking-through-a-shared-ref --features broken
```

Then try:

1. In `01-look-dont-take`, turn `&lines` into `lines` — drop the `&`. What error do you get, and which lesson do you know it from?
2. In `02-through-a-mutable-ref`, remove the star inside `double_all` and write `value *= 2`. Compare the message with the one from `05-forgot-the-star`.
3. In `03-following-the-arrow`, add `println!("{}", *look);`. It compiles. Now add `let owned = *look;`. It doesn't. What is the difference between those two lines?

---

## Errors you will meet

### `E0596` — cannot borrow as mutable

```text
error[E0596]: cannot borrow `greeting` as mutable, as it is not declared as mutable
  --> examples\04-owner-not-mut.rs:10:21
   |
10 |     add_exclamation(&mut greeting);
   |                     ^^^^^^^^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
 8 |     let mut greeting = String::from("hello");
   |         +++
```

**What the compiler is objecting to:** `&mut greeting` says "lend me permission to write to `greeting`". But `greeting` was bound with a plain `let`, so even its owner has no permission to write. A permission you don't hold can't be lent.

**The fix:**

```rust
let mut greeting = String::from("hello");
```

**Why that's the fix:** notice the error points at line 10 but the help points at line 8 — the compiler walked back and found the *original binding*. That's the distinction [1.1.1](../../01-foundations/01-variables-mutability-shadowing/README.md) set up: `mut` is a property of the binding, not of the value. And it hands you something good: every function signature now says outright whether it will change your value.

### `E0308` — assigning to the arrow instead of its target

```text
error[E0308]: mismatched types
  --> examples\05-forgot-the-star.rs:16:15
   |
15 | fn reset(counter: &mut i32) {
   |                   -------- expected due to this parameter type
16 |     counter = 0;
   |               ^ expected `&mut i32`, found integer
   |
help: consider dereferencing here to assign to the mutably borrowed value
   |
16 |     *counter = 0;
   |     +
```

**What the compiler is objecting to:** `counter` is a `&mut i32`, not an `i32`. Writing `counter = 0` says "replace this arrow with the number zero", which is nonsense. What you meant was "set the thing at the end of the arrow to zero".

**The fix:**

```rust
*counter = 0;
```

**Why that's the fix:** assignment is exactly where auto-deref won't help you. A method dot can guess that you meant the arrow's target; `=` cannot, because "replace the arrow" is also a perfectly valid thing to want. So you have to say which.

Working rule: whenever a reference is on the left of an `=`, you want a `*`.

### `E0507` — cannot move out of what is behind a shared reference

```text
error[E0507]: cannot move out of `*text` which is behind a shared reference
  --> examples\06-taking-through-a-shared-ref.rs:14:16
   |
14 |     let mine = *text;
   |                ^^^^^ move occurs because `*text` has type `String`, which does not implement the `Copy` trait
   |
help: consider removing the dereference here
   |
14 -     let mine = *text;
14 +     let mine = text;
   |
help: consider cloning the value if the performance cost is acceptable
   |
14 -     let mine = *text;
14 +     let mine = text.clone();
   |
```

**What the compiler is objecting to:** `*text` is a `String`, and `String` isn't `Copy`, so `let mine = *text` is a move. But `text` is only a borrow — the owner of that `String` is somewhere else and still wants it. If the move were allowed, two things would be responsible for one buffer.

**The fix:** it depends what you actually wanted.

```rust
let mine = text.clone();
```

**Why that's the fix:** this error usually means you asked the wrong question. If you only meant to read it, you never needed the `*` and plain `text` would have done. If you genuinely want an independent copy, `.clone()` is the honest answer and [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md) already told you its price.

Note how this differs from the `E0507` in [1.2.2](../../02-ownership-and-memory/02-move-semantics/README.md): there the value was being pulled out of an owner, here out of a borrow. The message says so in as many words — `which is behind a shared reference`.

---

## Exercises

### Warm up

<details>
<summary>Does a reference own anything? What gets freed when one ends?</summary>

No, and nothing. Dropping is the owner's job. The end of a borrow only means that permission is no longer valid.

</details>

<details>
<summary>How many bytes does <code>&lines</code> copy when <code>lines</code> is a <code>Vec</code> of a thousand strings?</summary>

One arrow's worth — a single machine word. The size of the thing at the far end has nothing to do with it.

</details>

<details>
<summary>Why does <code>let second = first;</code> leave both alive when <code>first</code> is a <code>&amp;String</code>?</summary>

Because `&T` is `Copy`. The arrow was copied, not the `String` at the end of it. You knew this from [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md).

</details>

<details>
<summary>Does this compile? <code>let text = String::from("hi"); let r = &mut text;</code></summary>

No. `E0596`. `text` was bound with a plain `let`, so the owner has no write permission either and there is nothing to lend. It needs `let mut text`.

</details>

<details>
<summary>In <code>fn bump(counter: &amp;mut i32)</code>, why does <code>counter += 1</code> fail where <code>*counter += 1</code> works?</summary>

`counter` is itself an arrow. `counter += 1` says "add one to the arrow", and the types don't line up — `E0308`. `*counter` is the number at the end of it.

</details>

<details>
<summary>Two <code>&amp;mut i32</code> alive at the same time — always an error?</summary>

No. Perfectly legal if they point at two different values. The rule is about one value, not about counting.

</details>

<details>
<summary>What does <code>for value in values</code> give you each turn when <code>values</code> is a <code>&amp;mut Vec&lt;i32&gt;</code>?</summary>

A `&mut i32`. Writing through it needs `*value`. Over a `&Vec<i32>` the same loop gives a `&i32`, which is read-only.

</details>

### Repair

Fix all three broken examples:

1. `examples/04-owner-not-mut.rs` — one word. Once it's fixed, remove the `&mut` from the call and see what it says then.
2. `examples/05-forgot-the-star.rs` — one character. Then try `counter += 1` as well and compare the two messages.
3. `examples/06-taking-through-a-shared-ref.rs` — fix it **two** ways: once so that you get an independent `String`, and once so that no new `String` is built at all (change the signature so it only returns the length).

Run `cargo clippy` on the second version. It has something to say about `&String`. It's right, and the full answer is [1.4.1](../../04-text-and-strings/01-string-vs-str/README.md); for now just read it and move on.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p1-03-01-shared-and-mutable-refs
```

Two of them only look; three of them write. None needs `.clone()`, and none returns anything purely to give it back. Three of them need a `*` somewhere, and working out where is half the exercise.

Besides the right answer, the tests check one more thing: that an argument you were only lent is still the caller's after the call.

### Build

Take `examples/02-giving-it-back.rs` from [1.2.4](../../02-ownership-and-memory/04-ownership-across-functions/README.md) and rewrite every function in it to borrow.

Then count: how many return values disappeared? How many `let` lines in `main` got simpler?

Then write a small program of your own: a `Vec<String>` of report lines in `main`, and three functions over it —

- one that reports how many lines there are,
- one that adds a new line,
- one that empties it.

All three work through references. Now look at them: which took `&` and which took `&mut`? How many of them returned anything? If the answer to the last one is "one", you wrote it right.

### Challenge (optional)

**Part one.** Run this and read the error. Write down its name and code, then guess what the next lesson is going to say:

```rust
let mut lines = vec![String::from("alpha")];
let reader = &lines;
lines.push(String::from("beta"));
println!("{}", reader.len());
```

**Part two.** This doesn't compile, and the error is `E0382` — the one you know from [1.2.2](../../02-ownership-and-memory/02-move-semantics/README.md). Why here? (Hint: `&mut T` is not `Copy`.)

```rust
let mut count = 10;
let first = &mut count;
let second = first;
*first += 1;
println!("{second}");
```

**Part three.** Write a `fn` taking a `&Vec<String>` and returning a reference to its longest string. It compiles. Now write one taking **two** `&Vec<String>` arguments and still returning a reference: you get `E0106`. Read its help text and write down the new word — [1.3.3](../03-borrow-scopes-and-nll/README.md) and then Phase 2 go after it.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| borrowing | using a value without taking it | nearly every function signature |
| shared reference `&T` | read-only arrow; any number at once | the default |
| mutable reference `&mut T` | an arrow you may write through | when you must change the caller's value |
| `*` (dereference) | "the value at the end of this arrow" | always on the left of an `=` |
| auto-deref | the method dot inserts `*` for you | `look.len()` on a `&String` |
| `E0596` | the owner isn't `mut` | one word away from fixed |
| `E0507` behind a reference | trying to take out of a borrow | means you asked the wrong question |

### What you now know

- `&` makes an arrow to the caller's value; nothing is copied and ownership doesn't move.
- A reference owns nothing, so the end of one frees nothing.
- `&T` is read-only and `Copy`; `&mut T` can write and is not `Copy`.
- `&mut x` is only possible when `x` is itself `mut`.
- You need `*` for assignment; for method calls the compiler inserts it.
- Looping over a `&mut Vec<i32>` gives you a `&mut i32` each turn.
- The take-it-and-give-it-back pattern is replaced by a single `&`.

### What comes back later

- **The full aliasing rule, and the errors `E0499` and `E0502`** — [1.3.2 — The rules of the borrow checker](../02-borrow-checker-rules/README.md)
- **Exactly when a borrow ends** — [1.3.3 — Borrow scopes and NLL](../03-borrow-scopes-and-nll/README.md)
- **Borrowing part of a collection** — [1.3.4 — Slices](../04-slices/README.md)
- **Why `&str` is nearly always better than `&String`** — [1.4.1 — `String` vs `str`](../../04-text-and-strings/01-string-vs-str/README.md)
- **Returning a reference from a function, and the word "lifetime"** — [Phase 2 — Lifetimes and elision](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)
- **Changing something through a `&T`, when you really must** — [Phase 2 — `RefCell` and interior mutability](../../../phase2-intermediate/05-smart-pointers/03-refcell-and-interior-mutability/README.md)

### Can you explain?

- What borrowing is, in one sentence, without using the word "pointer".
- Why `&lines` costs the same for a thousand-element `Vec` as for a three-element one.
- Why `&mut x` requires `x` to be `mut`.
- When you have to write `*` and when you don't.
- Why `let mine = *text;` is rejected when `text` is a `&String`.
- What the `-> (usize, Vec<String>)` pattern from the last lesson turns into now.

---

## Going further

- [The Rust Book — References and borrowing](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) — the same ground, officially.
- [Rust by Example — Borrowing](https://doc.rust-lang.org/rust-by-example/scope/borrow.html) — more short examples to run.
- [`rustc --explain E0596`](https://doc.rust-lang.org/error_codes/E0596.html) — get into the habit of opening every error code you meet like this.
