# 1.5.1 — Structs and methods

## At a glance

After this lesson you can:

- Declare a `struct` with named fields, build one, and read and change its fields.
- Choose between `&self`, `&mut self` and `self`, and say what each one means for the caller.
- Write an associated function like `Self::new` and explain why `new` is a convention rather than a keyword.
- Pull in the obvious behaviour with `#[derive(...)]` — `Debug`, `Clone`, `PartialEq` — and read `E0616` when a field is private.

**Time:** ~60 minutes · **Prerequisites:**
[1.1.3 — Compound types and destructuring](../../01-foundations/03-compound-types-and-destructuring/README.md) ·
[1.2.3 — `Clone` and `Copy`](../../02-ownership-and-memory/03-clone-and-copy/README.md) ·
[1.3.1 — Shared and mutable references](../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md)

---

## Why this matters

The tuple lesson ended owing you something: "a tuple is bearable up to about three fields. Past that, `thing.3` becomes a puzzle and you owe the fields names." This lesson pays that debt.

But a struct isn't just "a tuple with names". It brings three things you haven't had until now:

1. **A new type.** `Anime` isn't an alias for `(String, u32, u32, bool)` — it's a type that is only itself. A function that wants an `Anime` will not take a tuple of the same shape.
2. **Somewhere for behaviour to live.** An `impl` block attaches functions to the type, so you stop hunting for which free function goes with which data.
3. **A boundary.** Fields are private outside the module unless you say otherwise. Which means you can guarantee that `watched` never exceeds `episodes` — and have the compiler stand behind the guarantee.

And one ownership point that the whole of Phase 1 has been preparing you for: when you write a method, its signature decides whether the caller **keeps** their value, **keeps it and sees it change**, or **loses it**. That's `&T`, `&mut T` and moving, arriving on a type you wrote yourself.

---

## The concept

### The tuple that got too big

Four values that belong together, as a tuple:

```rust
let as_tuple = (String::from("Cowboy Bebop"), 26_u32, 26_u32, true);
println!("tuple:   {} {} {}", as_tuple.0, as_tuple.1, as_tuple.3);
```

```text
tuple:   Cowboy Bebop 26 true
```

It compiles and it works. The problem is elsewhere: `as_tuple.1` and `as_tuple.2` are both `u32`. One is how long the series is and one is how far you've got — but which is which? Swap them and the compiler says nothing at all.

The same four values with names:

```rust
struct Anime {
    title: String,
    episodes: u32,
    watched: u32,
    favourite: bool,
}

let bebop = Anime {
    title: String::from("Cowboy Bebop"),
    episodes: 26,
    watched: 26,
    favourite: true,
};
```

```text
struct:  Cowboy Bebop 26/26 favourite=true
```

`struct` declares a **new type**. The field names are part of that type, not a convention you carry in your head.

The second half is a **struct literal**, and three things about it are worth stating:

- **It isn't a function call.** There's no order to remember; write `favourite` first and `title` last if you like.
- **Every field must be there.** Leave one out and you get `E0063`, and the error names the ones you're missing.
- **It takes the values.** That `String::from("Cowboy Bebop")` now belongs to `bebop`. A struct owns its data.

> **Python bridge:** the closest thing is a `@dataclass`. Where the bridge breaks: here every field's type is checked at compile time, there's no `__dict__` to bolt a new field onto at run time, and misspelling a field name is a compile error rather than the quiet creation of a new attribute.

### Reading a field, and changing one

Reading is a dot: `bebop.title`. So is writing — provided the binding is `mut`:

```rust
let mut frieren = Anime {
    title: String::from("Frieren"),
    episodes: 28,
    watched: 3,
    favourite: false,
};
frieren.watched += 1;
frieren.favourite = true;
```

```text
changed: Frieren 4/28 favourite=true
```

Look where the `mut` is: on the **binding**, not on a field. There is no `mut watched: u32` in a struct declaration. A struct is mutable as a whole or not at all.

That's 1.1.1's rule arriving on your own type: mutability is a property of the **binding**, not of the data. Forget it and you get `E0594`, which is in the errors section.

### Field init shorthand

When the variable you already have is named after the field, don't write the name twice:

```rust
fn start_watching(title: String, episodes: u32) -> Anime {
    let watched = 0;
    Anime {
        title,
        episodes,
        watched,
        favourite: false,
    }
}
```

```text
started: Mushishi 0/26
```

`title,` means exactly `title: title`. That's **field init shorthand**, and it's everywhere in real Rust — especially inside `new` functions, where the parameters are deliberately named after the fields.

The long form is legal too, and clippy will mention it with `redundant_field_names`.

### One struct from another

You want to rewatch the same series: everything as before, but `watched` back to zero and `favourite` on.

```rust
let rewatch = Anime {
    watched: 0,
    favourite: true,
    ..started
};
println!("left of started: episodes {}", started.episodes);
```

```text
rewatch: Mushishi 0/26
left of started: episodes 26
```

`..started` means "and every other field from that one". That's **struct update syntax**. It has to come last inside the braces, and it takes no trailing comma.

Now the part that catches people out: **`..started` doesn't copy, it moves.**

That `title` is a `String`, so it moved into `rewatch`. The two `u32` fields are `Copy` (1.2.3), so they were copied and can still be read — which is why `started.episodes` worked above. But `started` as a whole value is gone: pass it to a function, or read `started.title`, and you get `E0382`. This is a **partial move**.

### `#[derive(Debug)]` and seeing the whole thing

`println!("{}", bebop)` doesn't compile, and that's deliberate: Rust has no idea how your type should be shown to a user. But for you — for when you just want to see what's inside — there's a ready-made answer:

```rust
#[derive(Debug, Clone, PartialEq)]
struct Anime {
    title: String,
    episodes: u32,
    watched: u32,
    favourite: bool,
}
```

```rust
println!("{bebop:?}");
println!("{bebop:#?}");
```

```text
Anime { title: "Cowboy Bebop", episodes: 26, watched: 26, favourite: true }

Anime {
    title: "Cowboy Bebop",
    episodes: 26,
    watched: 26,
    favourite: true,
}
```

`derive` is what you met in 1.2.3: "write the obvious implementation for me". For `Debug`, obvious means "the type's name, then every field with its own name".

- `{:?}` is one line. For a log.
- `{:#?}` is the same information with one field per line. For a human. That's the `#` flag from 1.4.3's formatting mini-language.

And notice `Debug` puts the string in quotes. That's on purpose: `Debug` shows you what a value **is**, not how it reads.

Skip the derive and write `{:?}` anyway and you get `E0277`, below.

### `Clone` and `PartialEq`, derived

That one `derive` line gave `Anime` two more things:

```rust
let copy = bebop.clone();
println!("bebop == copy:   {}", bebop == copy);
let mut edited = bebop.clone();
edited.watched = 1;
println!("bebop == edited: {}", bebop == edited);
```

```text
bebop == copy:   true
bebop == edited: false
```

A derived `Clone` clones every field. Here that's one allocation for the `String` and a plain copy of the three small fields — 1.2.3's counting, now on your own type.

A derived `PartialEq` compares every field. So `==` on an `Anime` means precisely "all four fields are equal". If the right notion of equality for your type is something else — only `title` matters, say — you have to write it yourself, and writing traits by hand is Phase 2 work.

`Copy` isn't available here: the `String` blocks it, and the error is 1.2.3's `E0204`.

### Methods, and `&self`

An `impl` block attaches functions to a type:

```rust
impl Anime {
    fn remaining(&self) -> u32 {
        self.episodes - self.watched
    }
}
```

```text
remaining:  28
```

`&self` is short for `self: &Self`, and `Self` inside an `impl` block means "the type this block is for" — here, `Anime`.

So `remaining` takes a **shared reference** to the struct: it only looks. The caller keeps their value and can go on using it.

The call syntax is sugar. These two lines are the same operation:

```rust
println!("remaining:  {}", show.remaining());
println!("same call:  {}", Anime::remaining(&show));
```

```text
remaining:  26
same call:  26
```

Dot syntax inserts the `&` you never type. Know that it's there, because when a borrow error lands on a method call, that invisible `&` is what it's about.

> **Python bridge:** Python makes `self` explicit too, which is half the distance. The difference is that Rust makes you say **how** you want it, and Python has no word for that distinction at all.

### `&mut self` — a method that changes something

```rust
fn watch_one(&mut self) {
    if self.watched < self.episodes {
        self.watched += 1;
    }
}

show.watch_one();
show.watch_one();
```

```text
watched:    2/28
```

`&mut self` is an **exclusive reference**, exactly the `&mut T` of 1.3.1. The caller keeps their value and sees the change.

Two conditions come with it, both from the aliasing rule:

- The caller's binding has to be `mut`, or the compiler stops you.
- While the method runs, no other reference to that struct is alive.

### `self` — a method that swallows the value

```rust
fn into_title(self) -> String {
    self.title
}

let title = show.into_title();
println!("title:      {title}");
```

```text
title:      Frieren
```

Here `self` is the value itself, not a reference to it. The method takes ownership, and after that line the caller no longer has a struct.

Why would anyone write that? Because sometimes you want a field out without cloning it. `into_title` hands over that `String`'s buffer directly — zero allocations. The price is that the rest of the struct is dropped right there.

The naming convention says the same thing: a method starting with `into_` consumes the value. `to_` makes a copy, and `as_` is a cheap look. The standard library keeps to this throughout.

And the consumption hides behind the dot: `show.into_title()` looks nothing like a move. Use `show` afterwards and you get `E0382` — and the error says so in as many words.

### Which `self` to take

| You write | The caller | When |
|---|---|---|
| `&self` | keeps the value and can read it | reading, computing, formatting — most methods |
| `&mut self` | keeps it and sees the change; their binding must be `mut` | changing a field |
| `self` | loses the value | turning the whole thing into something else |
| no `self` | there is no value yet | constructors and other associated functions |

That table is the whole of modules 1.2 and 1.3 sitting on a type you wrote. A method signature is a **promise to the caller**, made once; changing it later breaks other people's code.

Working rule: default to `&self`. Reach for `&mut self` when you have to change something. Take `self` only when the value genuinely stops meaning anything afterwards.

### Associated functions, and why `new` isn't a keyword

A function inside `impl` that takes no `self` isn't a method:

```rust
impl Anime {
    fn new(title: String, episodes: u32) -> Self {
        Self {
            title,
            episodes,
            watched: 0,
            favourite: false,
        }
    }
}

let mut show = Anime::new(String::from("Frieren"), 28);
```

```text
remaining:  28
```

That's an **associated function**: attached to the type rather than to an instance, and called through the type with `::`.

Rust has no constructors. `new` is a name everyone agreed on; nothing in the language knows it. You can have `Anime::from_file`, or `Anime::empty`, or all three, or none.

And you've seen the shape before: `String::from`, `Vec::new`, `Vec::with_capacity`. All associated functions. Now you know what that `::` was.

> **Python bridge:** `__init__` is both special-cased by name and limited to one per class. `new` is neither. Where the bridge breaks: in Python the object exists first and `__init__` fills it in; here the value doesn't exist until all four fields do. A half-built state isn't expressible.

### Fields are private outside the module

Example 07 declares this inside a module and then, from outside it, reads `episodes`:

```rust
pub struct Anime {
    pub title: String,
    episodes: u32,
    watched: u32,
}
```

```text
error[E0616]: field `episodes` of struct `Anime` is private
```

`pub` on the struct makes the **type** usable elsewhere and says nothing about its fields. Each field is private on its own unless it gets a `pub`.

And private means private to the **module**, not to the struct. Code in the same module reads them freely — which is why the tests inside `src/lib.rs` can see private fields.

This default is the opposite of Python's, where everything is public and `_name` is a request. Here the compiler enforces it. And it's exactly what turns a struct from a bag of data into a real type: if `watched` is private and the only way to raise it is `watch_one`, then "`watched` never exceeds `episodes`" stops being a hope and becomes a guarantee.

Modules get their own lesson in Phase 2. For now it's enough that a boundary exists and that your methods are the gate through it.

One last note: every struct in this lesson **owns** its data — it holds a `String`, not a `&str`. A struct that holds a reference instead needs an explicit lifetime, and that's Phase 2.

---

## Hands on

```sh
cargo run -p p1-05-01-structs-and-methods --example 01-from-tuple-to-struct
cargo run -p p1-05-01-structs-and-methods --example 02-debug-and-derives
cargo run -p p1-05-01-structs-and-methods --example 03-methods-and-self
```

Then the five broken ones:

```sh
cargo run -p p1-05-01-structs-and-methods --example 04-missing-a-field --features broken
cargo run -p p1-05-01-structs-and-methods --example 05-a-field-you-cannot-change --features broken
cargo run -p p1-05-01-structs-and-methods --example 06-printing-without-debug --features broken
cargo run -p p1-05-01-structs-and-methods --example 07-a-private-field --features broken
cargo run -p p1-05-01-structs-and-methods --example 08-consumed-by-its-own-method --features broken
```

Then try:

1. In `01`, add `println!("{}", started.title);` after the last line. Read the error — it's the other half of that partial move.
2. In `02`, take `PartialEq` out of the derive but keep `bebop == copy`. What does the compiler say, and what does it suggest?
3. In `03`, change `fn remaining(&self)` to `fn remaining(self)` and run it again. Which line breaks first, and why?

---

## Errors you will meet

### `E0063` — a field left out

```text
error[E0063]: missing fields `favourite` and `watched` in initializer of `Anime`
  --> examples\04-missing-a-field.rs:15:16
   |
15 |     let show = Anime {
   |                ^^^^^ missing `favourite` and `watched`
```

**What the compiler is objecting to:** a struct literal names every field or it isn't a value of that type at all. A half-built value isn't expressible in Rust — which is exactly what an incomplete `__init__` gives you in Python.

**The fix:** write the missing fields, or, if you don't have sensible values to hand, add an associated function like `Anime::new` that supplies the starting ones.

**Why that's the fix:** notice the error **names** the missing fields rather than just saying the count is wrong. On a twenty-field struct that one line is the difference between ten seconds and ten minutes.

And if you keep finding yourself typing the same starting values, that's the sign this struct wants a `new`.

### `E0594` and `E0596` — one mistake, two errors

```text
error[E0594]: cannot assign to `show.watched`, as `show` is not declared as mutable
  --> examples\05-a-field-you-cannot-change.rs:27:5
   |
27 |     show.watched += 1;
   |     ^^^^^^^^^^^^^^^^^ cannot assign
   |
help: consider changing this to be mutable
   |
21 |     let mut show = Anime {
   |         +++
```

```text
error[E0596]: cannot borrow `show` as mutable, as it is not declared as mutable
  --> examples\05-a-field-you-cannot-change.rs:28:5
   |
28 |     show.watch_one();
   |     ^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
21 |     let mut show = Anime {
   |         +++
```

**What the compiler is objecting to:** one thing, twice. `show` was bound with a plain `let`, so it's immutable. `E0594` is what you get assigning to a field directly; `E0596` is what you get calling a `&mut self` method, because that's where the invisible `&mut` is taken.

**The fix:** `let mut show = ...`. Both `help` blocks point at exactly that.

**Why that's the fix:** because `mut` is on the binding, not on the field. There's no way to make a single field mutable, and that's deliberate — "this value may change under you" is a statement made in one place about the whole value.

The same rule bites through `&self` too: writing `self.title = ...` inside `fn rename(&self)` is `E0594` again, because that reference is read-only. The fix there is `&mut self`.

### `E0277` — your type doesn't know how to print itself

```text
error[E0277]: `Anime` doesn't implement `Debug`
  --> examples\06-printing-without-debug.rs:20:15
   |
20 |     println!("{show:?}");
   |               ^^^^^^^^ `Anime` cannot be formatted using `{:?}` because it doesn't implement `Debug`
   |
   = help: the trait `Debug` is not implemented for `Anime`
   = note: add `#[derive(Debug)]` to `Anime` or manually `impl Debug for Anime`
help: consider annotating `Anime` with `#[derive(Debug)]`
   |
 7 + #[derive(Debug)]
 8 | struct Anime {
   |
```

**What the compiler is objecting to:** `{:?}` isn't free. Somebody has to have said how this type prints, and for your brand-new type nobody has.

**The fix:**

```rust
#[derive(Debug)]
struct Anime {
```

**Why that's the fix:** the error puts both options in front of you — derive it, or write it by hand. As long as the obvious printout is good enough, and for debugging it nearly always is, `derive` is the right answer. Writing it by hand becomes necessary when you want to hide something, like a password field.

And mind the difference between `{}` and `{:?}`. `{}` is for text a user reads and needs the `Display` trait; `{:?}` is for a programmer, and `Display` can't be derived — because Rust can't guess how your type should read to a human.

### `E0616` — a field the outside can't see

```text
error[E0616]: field `episodes` of struct `Anime` is private
  --> examples\07-a-private-field.rs:39:35
   |
39 |     println!("episodes: {}", show.episodes);
   |                                   ^^^^^^^^ private field
```

**What the compiler is objecting to:** `Anime` is public and `title` got a `pub`, but `episodes` didn't. `pub` on the struct doesn't reach its fields.

**The fix:** one of two, and choosing between them is a design decision:

```rust
impl Anime {
    pub fn episodes(&self) -> u32 {
        self.episodes
    }
}
```

Or `pub episodes: u32` in the declaration itself.

**Why that's the fix:** the reader method keeps the field private. Which means tomorrow you can change how it's stored — turn `episodes` from a counter into something computed — without breaking anybody's code. Making the field `pub` sells that freedom permanently.

Keep the default private and reach for `pub` only when the struct genuinely is nothing more than a bag of data.

### `E0382` — a method that took the value away

```text
error[E0382]: borrow of moved value: `show`
  --> examples\08-consumed-by-its-own-method.rs:27:41
   |
20 |     let show = Anime {
   |         ---- move occurs because `show` has type `Anime`, which does not implement the `Copy` trait
...
25 |     let title = show.into_title();
   |                      ------------ `show` moved due to this method call
26 |
27 |     println!("{title} has {} episodes", show.episodes);
   |                                         ^^^^^^^^^^^^^ value borrowed here after move
   |
note: `Anime::into_title` takes ownership of the receiver `self`, which moves `show`
  --> examples\08-consumed-by-its-own-method.rs:14:19
   |
14 |     fn into_title(self) -> String {
   |                   ^^^^
```

**What the compiler is objecting to:** `into_title` was written with `self`, so calling it is a move. This is 1.2.2's `E0382`, except this time the move was hiding behind a dot.

Read that last `note`: the compiler went and found the method's signature and underlined the `self`. It's telling you where the culprit is.

**The fix:** it depends what you actually wanted.

- If you need the struct afterwards, the method shouldn't take `self`. Make it `&self` and return a copy.
- If you don't, reorder: read `show.episodes` first, then call `into_title`.

**Why that's the fix:** because the question is never "how do I silence this", it's "what was this method promising". `into_` in the name means "I take this value and hand you a different one". If that isn't the promise you want, the signature is wrong, not the call site.

---

## Exercises

### Warm up

<details>
<summary>For <code>struct Anime { title: String, ... }</code>, how much sits on the stack and how much on the heap?</summary>

The fields sit next to each other on the stack: three words for the `String`, plus two `u32`s and a `bool`. The only thing on the heap is the `String`'s own text buffer. A struct adds nothing new to the heap; it just lays its fields out side by side.

</details>

<details>
<summary>Which of these two <code>println!</code>s compiles?</summary>

```rust
let b = Anime {
    watched: 1,
    ..a
};
println!("{}", a.episodes);
println!("{}", a.title);
```

The first. `a.episodes` is a `u32`, so it was copied and can still be read. `a.title` moved into `b`, and reading it is `E0382`.

</details>

<details>
<summary>Does this compile?</summary>

```rust
impl Anime {
    fn rename(&self, new_title: String) {
        self.title = new_title;
    }
}
```

No. `&self` is read-only, so assigning to `self.title` is `E0594`. The right signature is `&mut self`.

</details>

<details>
<summary>What's the difference between <code>Anime::new(...)</code> and <code>show.remaining()</code>?</summary>

The first is an associated function: it takes no `self` and is called through the type, because there's no instance yet. The second is a method: dot syntax makes a `&show` and passes it as `self`.

</details>

<details>
<summary>What happens if you call <code>new</code> something else, like <code>build</code>?</summary>

Nothing. `new` isn't a keyword and nothing in the language knows the name; it's a convention that makes other people's code easier to read.

</details>

<details>
<summary>Why can the tests in <code>src/lib.rs</code> read private fields when example 07 can't?</summary>

Because privacy is measured against the module, not against the struct. `mod tests` is inside the same file; example 07 stands outside `mod catalog`.

</details>

### Repair

Five broken examples, five errors. Fix each one **two** ways and say which is better:

1. `04-missing-a-field.rs` — once by writing the missing fields, once by adding an `Anime::new`.
2. `05-a-field-you-cannot-change.rs` — once by making the binding `mut`, once by deleting the two lines that change things. (The real question: why is one of those almost always the wrong answer?)
3. `06-printing-without-debug.rs` — once with the derive, once by printing each field with `{}`.
4. `07-a-private-field.rs` — once by making the field `pub`, once by adding a reader method.
5. `08-consumed-by-its-own-method.rs` — once by swapping the two lines round, once by changing the method's signature.

### Implement

Six members on `Series` in `src/lib.rs`:

```sh
cargo test -p p1-05-01-structs-and-methods
```

The fields are private and stay that way. Before writing each body, look at its signature and say why its `self` is the one it is: `new` has none, `remaining` and `summary` manage with `&self`, `watch_one` and `mark_favourite` need `&mut self`, and `into_title` swallows the value.

One of the six will panic in a particular case if you write it as a plain subtraction. Its documentation says which case.

### Build

Write a `WatchList` that holds several `Series` together:

```rust
pub struct WatchList {
    name: String,
    shows: Vec<Series>,
}
```

Give it three members:

- A `new` that makes an empty list with the given name.
- A method that adds a `Series` to the list. Choose the signature yourself, and say why the `Series` has to give up its ownership.
- A method returning the total number of unwatched episodes across the whole list. Write it with a `for` loop over `&self.shows`.

Then write one sentence: why should `shows` stay private, even when the outside code only wants to read it?

### Challenge (optional)

**Part one.** Methods that take `self` and return `Self` chain together:

```rust
let show = Anime::new(String::from("Frieren"), 28)
    .favourite()
    .watched(3);
```

Write `fn favourite(mut self) -> Self` and `fn watched(mut self, count: u32) -> Self` so that this works. Note `mut self`, not `&mut self`: you take the value, change it, and hand it back.

Then answer: between this and four lines of `let mut` plus assignments, which do you prefer and why? This pattern has a name, and you'll meet it again in Phase 3 on server configuration.

**Part two.** Remove `#[derive(PartialEq)]` from `Series` and run the tests. Which test breaks, and what's the error? Then think: if the right equality for two `Series` were equality of `title` alone, what would the derive have got wrong?

**Part three.** Write a tiny struct of your own with a `pub fn new() -> Self` that takes **no** arguments, and run `cargo clippy`. It has a suggestion. Read `new_without_default` in its documentation, say why clippy asks for it — and why it says nothing about `Series::new`. (The answer is the `Default` trait, and its lesson is in Phase 2.)

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| struct | a type made of named fields | anything whose fields want names |
| struct literal | `Anime { ... }` with every field | building an instance |
| field init shorthand | `title,` instead of `title: title` | nearly every `new` |
| struct update syntax | `..other` supplies the rest | changing a field or two of an existing value |
| partial move | the non-`Copy` fields have left | after `..other` |
| `impl` block | where a type's behaviour lives | every method |
| method | a function taking `self` | `show.remaining()` |
| associated function | a function in `impl` with no `self` | `Anime::new`, `String::from` |
| `Self` | the type this block is for | the return type of constructors |
| `#[derive(Debug)]` | gives you `{:?}` and `{:#?}` | anything you debug |
| `E0063` | a field missing from a literal | the error names them |
| `E0616` | a field is private outside the module | Rust's default |

### What you now know

- `struct` makes a new type, and the field names are part of it.
- `mut` is on the binding, not on a field; a struct is mutable as a whole or not at all.
- `..other` supplies the remaining fields and **moves** the ones that aren't `Copy`.
- `derive` writes the obvious implementation: `Debug` to see it, `Clone` to duplicate it, `PartialEq` for `==`.
- `&self` is a look, `&mut self` an exclusive look, `self` a swallow — and that choice is a promise to the caller.
- `show.remaining()` is sugar for `Anime::remaining(&show)`; dot syntax supplies the `&`.
- `new` isn't a keyword, it's a convention. Rust has no constructors.
- Fields are private outside the module, and that's what makes a struct a real type rather than a bag.

### What comes back later

- **Tuple structs and the newtype pattern, for when you want a type but not fields** — [1.5.2](../02-tuple-structs-and-newtype/README.md)
- **`enum`, for when a value is one of several shapes rather than all of these fields** — [1.5.3](../03-enums-as-data/README.md)
- **Taking a struct apart with a pattern** — [1.5.4 — `match` in depth](../04-match-in-depth/README.md)
- **`Option`, for a method that might have nothing to give back** — [1.6.1](../../06-absence-and-failure/01-option-and-null-safety/README.md)
- **Writing traits by hand, and `Display` versus `Debug`** — [Phase 2 — Defining and implementing traits](../../../phase2-intermediate/03-generics-and-traits/02-defining-and-implementing-traits/README.md)
- **A struct that holds a reference, and the lifetime it needs** — [Phase 2 — Lifetime basics](../../../phase2-intermediate/04-error-handling-and-lifetimes/03-lifetime-basics-and-elision/README.md)
- **Modules, and the boundary privacy is measured against** — [Phase 2 — Modules and visibility](../../../phase2-intermediate/06-project-organization-and-testing/01-modules-visibility-workspaces/README.md)

### Can you explain?

- What does a struct add over a tuple of the same shape?
- Why is `mut` on the binding rather than on a field?
- What happens to `other` after `..other`, and why are some of its fields still readable?
- What does each of `&self`, `&mut self` and `self` promise the caller?
- What exactly does `show.remaining()` translate into?
- Why isn't `new` a keyword, and what makes it work anyway?
- "The field is private" — private to what?

---

## Going further

- [The Rust Book — Structs](https://doc.rust-lang.org/book/ch05-00-structs.html) — the same ground, officially, with a rectangle-area example worth following through.
- [`std::fmt`](https://doc.rust-lang.org/std/fmt/) — the `Debug` section and the `#` flag that makes `{:#?}`.
- [Rust API Guidelines — naming](https://rust-lang.github.io/api-guidelines/naming.html) — the `as_` / `to_` / `into_` convention, and what each says about cost and ownership.
- [`clippy::new_without_default`](https://rust-lang.github.io/rust-clippy/master/#new_without_default) — the lint waiting for your first argument-free `new`.
