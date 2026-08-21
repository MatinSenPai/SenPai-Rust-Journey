# 1.5.2 — Tuple structs and the newtype pattern

## At a glance

After this lesson you can:

- Define a tuple struct and a unit struct, and reach their fields with `.0`.
- Say why `fn transfer(from: u64, to: u64, rial: u64)` is a bug that hasn't happened yet, and write a signature in which that bug is not expressible.
- Build a newtype that cannot be constructed wrong, and say what it costs at run time.
- Read and fix `E0308` between two newtypes, `E0369` on `+`, and `E0616` on a private field.

**Time:** ~45 minutes · **Prerequisites:** [1.5.1 — Structs and methods](../01-structs-and-methods/README.md) · [1.1.2 — Scalar types and overflow](../../01-foundations/02-scalar-types-and-overflow/README.md)

---

## Why this matters

[1.1.2](../../01-foundations/02-scalar-types-and-overflow/README.md) taught you not to keep money in an `f64` — store it as an integer in the smallest unit, rial. That lesson gave you the right representation.

It gave you no protection.

Because now an account balance is an `i64`, an account number is a `u64`, an order quantity is a `u64`, and a user id is a `u64`. To the compiler they are all the same thing. Look at this signature:

```rust
fn transfer(from: u64, to: u64, rial: u64)
```

Three arguments, one type. Every ordering compiles. The day somebody writes `transfer(user_id, amount, target_id)` in a busy handler, the compiler says nothing and the money goes to the wrong account. This bug has happened in real backends, and it has nothing to do with how careful the programmer was — it has to do with nobody having asked the types to help.

This lesson gives you one small tool that makes that line impossible to write. The tool costs nothing at run time, and it is among the most practically useful things in this whole phase for backend work.

---

## The concept

### A struct whose fields have no names

The last lesson's structs gave every field a name. Sometimes the name adds nothing: when the struct *is* one thing, writing `Meters { meters: 1.83 }` is silly.

```rust
#[derive(Debug)]
struct Meters(f64);

#[derive(Debug)]
struct Rgb(u8, u8, u8);

let height = Meters(1.83);
println!("debug form:   {height:?}");
println!("field .0:     {}", height.0);

let orange = Rgb(255, 165, 0);
println!("rgb:          {} {} {}", orange.0, orange.1, orange.2);
println!("rgb debug:    {orange:?}");
```

```text
debug form:   Meters(1.83)
field .0:     1.83
rgb:          255 165 0
rgb debug:    Rgb(255, 165, 0)
```

That is a **tuple struct**: it has fields, they have no names, and you reach them by position — `.0`, `.1`, `.2`. That is **positional field access**.

Three details that save trouble early:

- A tuple struct definition ends with a `;`. `struct Meters(f64);` — the semicolon is part of the syntax.
- `Meters` is both the type's name and the name of a function that builds it. `Meters(1.83)` is a real call.
- The numbering starts at zero, like the tuples in [1.1.3](../../01-foundations/03-compound-types-and-destructuring/README.md).

Working rule: up to three fields, and only where the order is obvious (as in `Rgb`), a tuple struct is fine. Past that, put the names back — `thing.3` tells a code reviewer nothing.

### Taking the value back out

`.0` is one way. The second is the destructuring you already did on tuples:

```rust
let Meters(raw) = height;
println!("destructured: {raw}");
```

```text
destructured: 1.83
```

The left of that `let` is a pattern, and `Meters(raw)` says "open this and call the thing inside `raw`". It is the same shape as `let (a, b) = pair;`, with the type's name in front.

Both are correct. `.0` is shorter; destructuring is better where you want to give the value a meaningful name.

### A struct with no fields at all

Zero fields is allowed too:

```rust
#[derive(Debug)]
struct Marker;

let marker = Marker;
println!("unit struct:  {marker:?}");
println!("size of Marker: {}", size_of::<Marker>());
```

```text
unit struct:  Marker
size of Marker: 0
```

**Zero bytes.** That is a **unit struct**, named after the empty tuple `()`, which is also zero bytes.

Right now it looks useless, and honestly it is: a unit struct earns its keep when you can attach behaviour to a type that carries no data — which means traits, in [Phase 2](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md). For now just know it exists and is free, because you will meet it constantly in library code.

### Three `u64`s and a bug waiting to happen

Back to that signature. Run this and see what the compiler has to say:

```rust
fn transfer_untyped(from: u64, to: u64, rial: u64) -> String {
    format!("{rial} rial: {from} -> {to}")
}

// The amount went into the `to` slot.
println!("wrong:  {}", transfer_untyped(1001, 250_000, 2002));
println!("right:  {}", transfer_untyped(1001, 2002, 250_000));
```

```text
wrong:  2002 rial: 1001 -> 250000
right:  250000 rial: 1001 -> 2002
```

The compiler said **nothing**. Both lines compiled, both ran, and one of them sent 2002 rial to account number 250000.

The Python bridge: Python has `NewType`, which does something similar — but only when you run `mypy`, and it is completely gone at run time. That is exactly where the analogy stops: in Rust the check is not optional and is not a separate step. Either the code compiles or it doesn't.

### The newtype pattern

The fix is one line: instead of passing bare numbers around, wrap each meaning in its own tuple struct.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct AccountId(u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Rial(i64);

fn transfer(from: AccountId, to: AccountId, amount: Rial) -> String {
    format!("{} rial: {} -> {}", amount.amount(), from.0, to.0)
}

println!("{}", transfer(alice, bob, Rial::new(250_000)));
```

```text
250000 rial: 1001 -> 2002
```

That is the **newtype pattern**: a single-field tuple struct around a type you already have, purely so that a new type exists for the compiler to know about.

And now the central point of the whole lesson. Two newtypes wrapping the *same* type are, as far as the compiler is concerned, entirely unrelated:

```rust
struct Meters(f64);
struct Feet(f64);

fn describe(height: Meters) -> String {
    format!("{} m", height.0)
}

let measured = Feet(6.0);
println!("{}", describe(measured));
```

```text
error[E0308]: mismatched types
  --> examples\04-mixed-up-units.rs:17:29
   |
17 |     println!("{}", describe(measured));
   |                    -------- ^^^^^^^^ expected `Meters`, found `Feet`
   |                    |
   |                    arguments to this function are incorrect
```

`expected 'Meters', found 'Feet'` — two structs each holding exactly one `f64`, the same size, indistinguishable in memory. The compiler still refuses one where the other was asked for. **That sentence is the entire argument for the pattern:** a type is more than a memory layout; a type is a meaning, and the compiler keeps track of meanings.

### Making a newtype worth using

If you end up writing `.0` everywhere, the newtype is just friction. Three small things make it usable: a constructor, an accessor, and some `derive`s.

```rust
impl Rial {
    fn new(amount: i64) -> Rial {
        Rial(amount)
    }

    fn amount(self) -> i64 {
        self.0
    }
}

println!("debug:  {:?}", Rial::new(250_000));
println!("equal:  {}", Rial::new(5) == Rial::new(5));
```

```text
debug:  Rial(250000)
equal:  true
```

- `new` is the convention from [1.5.1](../01-structs-and-methods/README.md), on a tuple struct. It is the only door in.
- `amount` returns the value inside. It is called `amount`, not `get_amount`; a `get` prefix is not the Rust convention.
- `self` rather than `&self`, because `Rial` is `Copy` and copying eight bytes is cheaper than following a reference — that is [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md) again.
- `#[derive(Debug, Clone, Copy, PartialEq, Eq)]` gives you the four behaviours nearly every newtype wants in one line: printing, copying, and comparison with `==`. Note that `Debug` prints the tuple shape: `Rial(250000)`.

### A newtype as a validation boundary

So far the newtype only gave a name. Its second job: **making a value that cannot be constructed wrong.**

Keep the field private so the constructor is the only way in. Say, a percentage that must never exceed 100:

```rust
mod rates {
    #[derive(Debug, Clone, Copy)]
    pub struct Percent(u8);

    impl Percent {
        pub fn new(value: u8) -> Percent {
            if value > 100 {
                Percent(100)
            } else {
                Percent(value)
            }
        }
    }
}
```

```rust
let fee = rates::Percent::new(9);
let silly = rates::Percent::new(240);
println!("fee:    {}%", fee.value());
println!("clamped: {}% (asked for 240)", silly.value());
```

```text
fee:    9%
clamped: 100% (asked for 240)
```

(`value()` is the accessor from the previous section, `pub` this time.)

`Percent(u8)` with no `pub` means the field is invisible outside the `rates` module — the privacy rule from [1.5.1](../01-structs-and-methods/README.md) — and writing `fee.0` from outside gets you `E0616`, which is in the errors section. The consequence is the point: **every `Percent` that exists in the program went through the constructor.** No function downstream has to check again.

Be honest about one thing here: clamping to 100 is not the *right* answer. An input of 240 is an error, and hiding it behind a 100 is what Python's `try/except: pass` does. The right answer is for the constructor to say "no" and explain why — that is, to return a `Result`. That tool arrives in [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md), and you rewrite this same `Percent` there. Until then, clamping is what today's tools can build — and it matters that you know why it isn't enough.

### What it costs

The fair question: how much does all this extra typing slow the program down?

```rust
println!("f64        {} bytes", size_of::<f64>());
println!("Meters     {} bytes", size_of::<Meters>());
println!("Feet       {} bytes", size_of::<Feet>());
println!("u64        {} bytes", size_of::<u64>());
println!("AccountId  {} bytes", size_of::<AccountId>());
```

```text
f64        8 bytes
Meters     8 bytes
Feet       8 bytes
u64        8 bytes
AccountId  8 bytes
```

**Nothing.** The wrapper adds zero bytes and leaves no trace in the machine code; adding two `Meters` is the same instruction as adding two `f64`s was. That is what "zero-cost abstraction" means.

The cost lands somewhere else: **on your fingers, at compile time.** Because `Meters` and `Feet` are unrelated, converting between them is your job:

```rust
fn to_feet(distance: Meters) -> Feet {
    Feet(distance.0 * 3.280_84)
}

let height = Meters(1.83);
let converted = to_feet(height);
println!("{height:?} is {converted:?} — {} feet", converted.0);
```

```text
Meters(1.83) is Feet(6.0039372) — 6.0039372 feet
```

Likewise: `price + fee` on two `Rial`s does not compile, because a newtype does not inherit the arithmetic of the type inside it. That error is `E0369` and it is in the next section in full. Today's answer is `Rial(price.0 + fee.0)`; the long-term answer is to implement the `Add` trait for `Rial` so `+` means something, and that is [Phase 2](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md).

Take that trade with your eyes open: you write a few conversion functions, and in exchange a whole family of bugs becomes permanently inexpressible.

### When a newtype is the wrong call

The pattern is good but it isn't free, and applying it everywhere makes code worse. Three signs you don't need one here:

- **The value never sits next to something of the same type.** A `count: usize` living in three lines inside one function has nothing to get confused with.
- **The wrapper never comes off.** If in practice you write `.0` everywhere, you are working with the inner type and the newtype has only added noise.
- **The boundary is somewhere else.** For data coming straight out of a database or a JSON body, the right place to validate is the input edge, where you work with `Result` rather than a clamping constructor.

And the reverse — three places where it almost always pays: identifiers (`UserId`, `OrderId`), money and units (`Rial`, `Meters`, `Milliseconds`), and any string with a rule attached (`Email`, `Slug`, `PhoneNumber`).

---

## Hands on

```sh
cargo run -p p1-05-02-tuple-structs-and-newtype --example 01-tuple-and-unit-structs
cargo run -p p1-05-02-tuple-structs-and-newtype --example 02-newtype-in-action
cargo run -p p1-05-02-tuple-structs-and-newtype --example 03-what-a-newtype-costs
```

Then the three broken ones:

```sh
cargo run -p p1-05-02-tuple-structs-and-newtype --example 04-mixed-up-units --features broken
cargo run -p p1-05-02-tuple-structs-and-newtype --example 05-adding-two-newtypes --features broken
cargo run -p p1-05-02-tuple-structs-and-newtype --example 06-private-inner --features broken
```

Then try:

1. In `01-tuple-and-unit-structs`, change `Rgb` to `struct Rgb(u8, u8, u8, u8)` and print its size again. What is the number now?
2. In `02-newtype-in-action`, change `transfer(alice, bob, ...)` to `transfer(alice, Rial::new(5), bob)`. How many errors does the compiler give, and which ones?
3. In `03-what-a-newtype-costs`, add a unit struct `Marker` of your own plus a `struct Empty(Marker);`, and print the size of `Empty`. What did you guess, and what happened?

---

## Errors you will meet

### `E0308` — two wrappers around one type are two types

```text
error[E0308]: mismatched types
  --> examples\04-mixed-up-units.rs:17:29
   |
17 |     println!("{}", describe(measured));
   |                    -------- ^^^^^^^^ expected `Meters`, found `Feet`
   |                    |
   |                    arguments to this function are incorrect
   |
note: function defined here
  --> examples\04-mixed-up-units.rs:11:4
   |
11 | fn describe(height: Meters) -> String {
   |    ^^^^^^^^ --------------
```

**What the compiler is objecting to:** `describe` wants a `Meters` and you gave it a `Feet`. Both hold exactly one `f64`, both are eight bytes, and in memory they are identical — but their types are not the same, and in Rust the type is what gets checked. `expected 'Meters', found 'Feet'` is the entire reason this pattern exists.

**The fix:** either pass the right value, or write the conversion out:

```rust
let measured = Feet(6.0);
println!("{}", describe(to_meters(measured)));
```

**Why that's the fix:** because converting feet to metres involves a multiplication, and somebody has to write that multiplication. Before the newtype it got forgotten and nobody found out; now the compiler won't move on until you write it.

The same file has a second error worth seeing:

```text
error[E0308]: mismatched types
  --> examples\04-mixed-up-units.rs:20:29
   |
20 |     println!("{}", describe(raw));
   |                    -------- ^^^ expected `Meters`, found `f64`
   |                    |
   |                    arguments to this function are incorrect
   |
note: function defined here
  --> examples\04-mixed-up-units.rs:11:4
   |
11 | fn describe(height: Meters) -> String {
   |    ^^^^^^^^ --------------
help: try wrapping the expression in `Meters`
   |
20 |     println!("{}", describe(Meters(raw)));
   |                             +++++++   +
```

A bare `f64` isn't accepted either: the newtype is closed in both directions. Read that `help` — the compiler is telling you to wrap the number in `Meters`. That is precisely the moment to ask yourself "where did this number come from, and what unit is it in?", and making you ask that question is the pattern's real job.

### `E0369` — `+` doesn't work on a newtype

```text
error[E0369]: cannot add `Rial` to `Rial`
  --> examples\05-adding-two-newtypes.rs:14:23
   |
14 |     let total = price + fee;
   |                 ----- ^ --- Rial
   |                 |
   |                 Rial
   |
note: an implementation of `Add` might be missing for `Rial`
  --> examples\05-adding-two-newtypes.rs:8:1
   |
 8 | struct Rial(i64);
   | ^^^^^^^^^^^ must implement `Add`
```

(There is one more `note` after this pointing into the standard library's source; its path differs on every machine, so it isn't reproduced here.)

**What the compiler is objecting to:** `Rial` wraps an `i64` but does not inherit addition from it. Rust has no inheritance: `+` works on a type because that type implements the `Add` trait, and `Rial` doesn't. `struct Rial(i64);` says "this is a new type", not "this is an `i64` under another name".

**The fix, with today's tools:** open the wrapper, add, wrap again:

```rust
let total = Rial(price.0 + fee.0);
```

**Why that's the fix:** addition is meaningful on the number inside, and the result is still an amount, so wrapping it back up is right. And notice what this *doesn't* stop you writing: `Rial(price.0 + weight.0)` compiles too, if `weight` is a `Grams` — because `.0` drops you back into the untyped world of `i64`. Every `.0` is a moment where you set the protection aside. Write few of them, in known places.

The long-term answer is to write `impl Add for Rial` so `+` works directly. Implementing traits is [Phase 2](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md); for now, just know it is possible.

### `E0616` — a private field, which is exactly what you wanted

```text
error[E0616]: field `0` of struct `Percent` is private
  --> examples\06-private-inner.rs:25:24
   |
25 |     println!("{}", fee.0);
   |                        ^ private field
```

**What the compiler is objecting to:** `Percent` is public but the field inside it is not. From outside the `rates` module you cannot read `.0` — and, more importantly, you cannot construct `Percent(240)`.

**The fix:** add an accessor:

```rust
impl Percent {
    pub fn value(self) -> u8 {
        self.0
    }
}
```

**Why that's the fix:** because this error is not a failure, it is the lock you asked for. Making the field `pub` throws the guarantee away: anyone can then build `Percent(240)`, and going through the constructor stops meaning anything. An accessor grants reading and withholds constructing, and that difference is the whole point.

Note that privacy in Rust is per **module**, not per type. Inside `mod rates` itself, `self.0` is free — which is why `value` works at all.

---

## Exercises

### Warm up

<details>
<summary>Does <code>struct Meters(f64);</code> need a semicolon?</summary>

Yes. Tuple-struct and unit-struct definitions end with `;`. Only the braced form with named fields doesn't take one.

</details>

<details>
<summary>If <code>Meters</code> and <code>Feet</code> both wrap an <code>f64</code>, why won't the compiler accept one for the other?</summary>

Because a type is not a memory layout; a type is a meaning. Being the same size and the same shape in memory has nothing to do with being the same type. The error is `E0308`, reading `expected 'Meters', found 'Feet'`.

</details>

<details>
<summary>Does this print or fail? <code>println!("{}", Rial(5).0 + Rial(7).0);</code></summary>

It prints `12`. Both `.0`s are there, so we are adding two `i64`s, not two `Rial`s. It is `Rial(5) + Rial(7)` that gives `E0369`.

</details>

<details>
<summary>How many bytes is a <code>struct Marker;</code>? And a <code>struct Wrapper(Marker);</code>?</summary>

Both zero. A unit struct carries no data, and wrapping something that is zero bytes stays zero bytes.

</details>

<details>
<summary>What does a <code>Percent</code> with a private field guarantee that a plain <code>u8</code> does not?</summary>

That every `Percent` in existence came through the constructor, so its value is in range. A `u8` can be 240 and nobody finds out.

</details>

<details>
<summary>What does wrapping a <code>u64</code> in a newtype cost at run time?</summary>

Nothing. The size is the same and so are the machine instructions. The cost is at compile time and on your fingers: you write the conversions.

</details>

### Repair

Fix `examples/04-mixed-up-units.rs` **two** ways:

1. By writing `fn to_meters(distance: Feet) -> Meters` and calling it (one foot is 0.3048 metres).
2. By wrapping that bare `f64` in `Meters` — exactly what the `help` suggests.

Then say which is more likely right in a real program, and why the question can't be answered until you know where that `6.0` came from.

Then fix `examples/05-adding-two-newtypes.rs` by unwrapping and re-wrapping, and write one sentence on what you gave up by doing that.

Then fix `examples/06-private-inner.rs` two ways: once by adding an accessor, once by making the field `pub`. Say why the second is almost always wrong.

### Implement

Seven small things in `src/lib.rs`:

```sh
cargo test -p p1-05-02-tuple-structs-and-newtype
```

Three types are written for you — `AccountId`, `Rial` and `Percent` — and you write the constructors, the accessors, and the three functions that speak in those types.

Notice the `pub` on `AccountId`'s field and its absence on `Rial` and `Percent`; the doc comments explain it, and the reason for the choice is what this lesson just argued.

`Percent::of` is the only one that needs thought: the order of the multiplication and the division changes the answer, and the doc comment says exactly which answer is right.

### Build

Write a newtype called `Slug` wrapping a `String` that guarantees the string inside is never empty: given an empty input, a `Slug` must hold `"untitled"`.

- `pub fn new(text: String) -> Slug`
- `pub fn as_text(&self) -> &str`

`&self` is right here rather than `self`, because `String` is not `Copy` — the rule from [1.2.3](../../02-ownership-and-memory/03-clone-and-copy/README.md).

Then write a `pub fn path(slug: &Slug) -> String` that puts `"/posts/"` in front of the text.

Then ask yourself this and answer it in a sentence: if an empty input had to produce an error instead of a replacement, what would `new`'s signature look like?

### Challenge (optional)

**Part one.** Guess this, then run it:

```rust
struct Wrapper(u64);
struct Deep(Wrapper);
struct Deeper(Deep);

println!("{}", size_of::<Deeper>());
```

How many bytes? Why? And how many `.0`s does it take to reach the number?

**Part two.** Rewrite `Percent::new` so that instead of clamping, an input above 100 stops the program with a clear message. Then write why that is almost always wrong in a library, though fine in an internal test. The right answer is a `Result`, and it arrives in [1.6.3](../../06-absence-and-failure/03-result-and-question-mark/README.md).

**Part three.** This one reaches past today's lesson and says so: look up `std::ops::Add` in the documentation and see what you would have to write for `price + fee` to work on two `Rial`s. You don't have to write it — just look at the shape of the `impl`, then meet it properly in [Phase 2](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md).

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| tuple struct | a struct whose fields have no names | single-field wrappers, `Rgb`, coordinates |
| positional field access | `.0`, `.1`, `.2` | reading unnamed fields |
| unit struct | a struct with zero fields and zero bytes | markers, later with traits |
| newtype pattern | a single-field wrapper making a brand-new type | `AccountId`, `Rial`, `Meters` |
| validation boundary | private field + constructor = a value that can't be built wrong | `Percent`, `Email` |
| `E0308` | two different wrappers around one type | the `expected`/`found` text is the argument |
| `E0369` | `+` isn't defined on a newtype | unwrap, add, wrap again |
| `E0616` | a private field was read from outside | add an accessor, don't add `pub` |

### What you now know

- A tuple struct has fields but no names, and you open it with `.0` or with destructuring.
- Tuple-struct and unit-struct definitions end with `;`, and `Meters` is both a type and a constructor.
- A unit struct is zero bytes.
- Two newtypes around the same type are two entirely separate types, and `E0308` proves it.
- A constructor, an accessor and some `derive`s are what make a newtype usable.
- A private field plus a constructor is a value that cannot be built invalid.
- The run-time cost is zero; the compile-time cost is the conversions you write.
- Every `.0` sets the protection aside for a moment. Write few, in known places.

### What comes back later

- **A constructor that can fail and say why** — [1.6.3 — `Result` and `?`](../../06-absence-and-failure/03-result-and-question-mark/README.md)
- **A value that might not be there, without `null`** — [1.6.1 — `Option`](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **A type with several different shapes, not one wrapped value** — [1.5.3 — Enums as data](../03-enums-as-data/README.md)
- **Implementing traits, including `Add` so `+` works on a newtype** — [Phase 2 — Defining and implementing traits](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md)
- **Fallible conversion with `TryFrom`, the grown-up form of a validating constructor** — [Phase 2 — `TryFrom`](../../../phase2-intermediate/08-rust-toolbox/04-tryfrom-fallible-conversions/README.md)

### Can you explain?

- What is the difference between a tuple struct and a struct with named fields, in one sentence?
- Why is `fn transfer(from: u64, to: u64, rial: u64)` dangerous?
- What is the difference between two newtypes around one `u64`? How does the compiler confirm your answer?
- What exactly does a private field in a newtype guarantee?
- What does a newtype cost at run time, and where does the real cost land?
- Where would you not reach for a newtype?

---

## Going further

- [The Rust Book — tuple structs](https://doc.rust-lang.org/book/ch05-01-defining-structs.html#using-tuple-structs-without-named-fields-to-create-different-types) — the same ground, officially.
- [Rust API Guidelines — newtypes](https://rust-lang.github.io/api-guidelines/type-safety.html#newtypes-provide-static-distinctions-c-newtype) — the official API design guide, on precisely this pattern.
- [`std::mem::size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html) — the function you use to check the "zero-cost" claim yourself.
