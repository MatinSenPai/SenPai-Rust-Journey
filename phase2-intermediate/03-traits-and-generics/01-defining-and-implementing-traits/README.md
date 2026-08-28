# 2.3.1 — Defining and implementing traits

## At a glance

After this lesson you can:

- Say exactly what a trait is — a contract of method signatures, with no fields at all, not a base class — and where that comparison to a Python ABC or Protocol breaks down.
- Define a trait of your own with one required method and one default method, implement it for two completely unrelated types — one accepting the default, one overriding it — and explain why an identical call works on both afterward.
- Explain why calling a trait's method requires importing the trait itself, even when the type is otherwise fully visible, and read and fix `E0046` and `E0599` yourself.

**Time:** ~45 minutes · **Prerequisites:** [2.2.4 — Implementing `Iterator` and `IntoIterator` for your own type](../../02-iterators-and-closures/04-implementing-iterator/README.md)

---

## Why this matters

The word "trait" is not new to you. [2.2.1](../../02-iterators-and-closures/01-closures-and-fn-traits/README.md) told you every closure implements one of three traits depending on its body — `Fn`, `FnMut`, `FnOnce` — and said right there that generics and `impl Trait` "get their full lesson in module 2.3". And in [2.2.4](../../02-iterators-and-closures/04-implementing-iterator/README.md) you wrote this exact shape with your own hands:

```rust
impl Iterator for Fibonacci {
    type Item = u64;
    // fn next(&mut self) -> Option<u64> { ... } — what you wrote in 2.2.4
}
```

That was a **trait implementation**. Only the trait itself — `Iterator` — had already been defined for you, by the standard library. Your job was only to fulfill a contract someone else had already written.

What's different today is this: you write the contract yourself, instead of only fulfilling one. That is not a small thing. Without it, every time several of your own types need to share one capability — all of them can be "summarized," all of them can be "validated," whatever it is — you either lock your code to one specific struct, or you memorize a separately-named function for every type by hand. A trait you wrote gives that contract a formal name, and from here on the compiler enforces it, not your memory.

---

## The concept

### What a trait is: a contract, not a base class

A trait is a set of method *signatures* — a method's name, its parameters, its return type — with no fields at all. No data lives inside a trait; only behavior. That is exactly what separates it from a base class in an object-oriented language: a base class usually hands down both data and behavior; a trait only promises behavior, and whatever type accepts that promise brings its own data along.

The closest thing you have probably used in Python is an `ABC` class (`abc.ABC` plus the `@abstractmethod` decorator) or a `Protocol`:

```python
class Summarize(ABC):
    @abstractmethod
    def title(self) -> str: ...
```

The idea is similar — a behavioral contract that several unrelated classes can accept. The real difference is *when* it gets checked. Python only discovers a missing override the moment you try to instantiate the incomplete class — or, with a plain `Protocol`, possibly never, until the exact line that calls the missing method blows the program up. Rust checks every trait implementation block at compile time, before any code runs at all. "Errors you will meet" shows you that difference with a real example.

### Declaring a trait: a required signature and a default implementation

Let's write a real one — a contract for "anything that can describe itself":

```rust
trait Summarize {
    fn title(&self) -> String;

    fn summary(&self) -> String {
        format!("{} (no summary available)", self.title())
    }
}
```

Two methods, two completely different shapes. `title`'s definition has no body — just a signature ending in `;`. That makes it a **required method**: any type that implements `Summarize` must supply its own, or it will not compile. `summary`'s definition, though, has a body — that makes it a **default method**: every implementor gets it for free, exactly as written, unless it chooses to override it.

One thing worth pausing on: the default body of `summary` calls the `title` method — a method that, right here, in this very definition, has no body and no concrete type behind it yet. This compiles because the trait itself guarantees it: whatever type ends up here is required to have a `title`. The compiler doesn't need to know the concrete type to know the call is safe; it only needs to know that whatever it is has implemented `Summarize`.

### Implementing it for the first type

A trait on its own is only a contract — no code runs until something implements it. You fulfill it with `impl Trait for Type`:

```rust
struct AnimeSeries {
    title: String,
    episodes: u32,
}

impl Summarize for AnimeSeries {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {} episodes", self.title(), self.episodes)
    }
}
```

```rust
let death_note = AnimeSeries {
    title: "Death Note".to_string(),
    episodes: 37,
};
println!("{}", death_note.title());
println!("{}", death_note.summary());
```

```text
Death Note
Death Note — 37 episodes
```

`AnimeSeries` supplied both `title` (required) and its own `summary` — not the default. That was a real choice: it could have left `summary` out entirely and gotten the default. It chose to write its own because it wanted to include the episode count too, which the default has no way of knowing about.

### The same trait, for a completely unrelated type

This is where that investment pays off. `MangaVolume` has nothing to do with `AnimeSeries` — no shared base struct, no inheritance, nothing. The only thing connecting these two types is that both promised `Summarize`:

```rust
struct MangaVolume {
    title: String,
}

impl Summarize for MangaVolume {
    fn title(&self) -> String {
        self.title.clone()
    }

    // No `summary` override here, on purpose: MangaVolume relies entirely
    // on Summarize's default implementation.
}
```

```rust
let berserk = MangaVolume {
    title: "Berserk Vol. 1".to_string(),
};
println!("{}", death_note.summary());
println!("{}", berserk.summary());
```

```text
Death Note — 37 episodes
Berserk Vol. 1 (no summary available)
```

The exact same call, twice, on two completely unrelated types. Calling `summary` on `death_note` runs `AnimeSeries`'s own version; the same call on `berserk` — which never wrote one — runs the trait's default, which calls `title` itself, and this time it's `MangaVolume`'s version that answers. Neither type knows the other exists. Both simply accepted one contract.

```senpai-visual
{"kind":"concept","labels":["summary called on berserk","no override in impl","trait's default body runs","title called from inside","MangaVolume's impl runs","final string returns"]}
```

### A trait must be in scope

One last thing, and this is the one that catches almost everyone off guard: calling a trait's method on a value requires the trait itself to be imported — even when the type is fully `pub` and otherwise reachable. Put `Summarize` inside a module:

```rust
mod catalog {
    pub trait Summarize {
        fn summary(&self) -> String;
    }

    pub struct AnimeSeries {
        pub title: String,
    }

    impl Summarize for AnimeSeries {
        fn summary(&self) -> String {
            self.title.clone()
        }
    }
}
```

```rust
use catalog::Summarize;

fn main() {
    let death_note = catalog::AnimeSeries {
        title: "Death Note".to_string(),
    };
    println!("{}", death_note.summary());
}
```

```text
Death Note
```

Remove the `use catalog::Summarize;` line — `AnimeSeries` is still fully `pub`, you can still construct it, but calling `summary` on it no longer compiles. The compiler knows this method exists somewhere; it just won't let you call it unless you also bring the trait itself into scope. The broken version, with its full error, is in "Errors you will meet".

---

## Hands on

```sh
cargo run -p p2-03-01-defining-and-implementing-traits --example 01-declaring-and-implementing-a-trait
cargo run -p p2-03-01-defining-and-implementing-traits --example 02-same-trait-two-unrelated-types
cargo run -p p2-03-01-defining-and-implementing-traits --example 03-trait-must-be-in-scope
```

Then the two broken ones:

```sh
cargo run -p p2-03-01-defining-and-implementing-traits --example 04-missing-required-method --features broken
cargo run -p p2-03-01-defining-and-implementing-traits --example 05-trait-not-in-scope --features broken
```

Then try these:

1. In `02-same-trait-two-unrelated-types`, add one more line that prints the result of calling `title` on `berserk`. Does it compile? What does it print?
2. In `03-trait-must-be-in-scope`, comment out `use catalog::Summarize;` and predict the error you will get — then compare it with `examples/05-trait-not-in-scope.rs`.
3. In `01-declaring-and-implementing-a-trait`, delete the entire `summary` override from `impl Summarize for AnimeSeries`. Predict what calling `summary` on `death_note` prints now, then run it and check.

---

## Errors you will meet

### `E0046` — not all trait items implemented

```text
error[E0046]: not all trait items implemented, missing: `title`
  --> phase2-intermediate\03-traits-and-generics\01-defining-and-implementing-traits\examples\04-missing-required-method.rs:17:1
   |
 6 |     fn title(&self) -> String;
   |     -------------------------- `title` from trait
...
17 | impl Summarize for LightNovel {}
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ missing `title` in implementation

For more information about this error, try `rustc --explain E0046`.
```

**What the compiler is objecting to:** the `impl Summarize for LightNovel` block is open, with no method inside it at all. `title`'s definition in the trait had no body, so it was required; `LightNovel` never delivered on that promise.

**The fix:** write the missing method:

```rust
impl Summarize for LightNovel {
    fn title(&self) -> String {
        self.title.clone()
    }
}
```

**Why this is the fix:** overriding `summary` (which has a default) was and still is optional; `title` (which has no body) was never optional. The compiler wants exactly what the trait promised from the start: a `title` from every implementor.

### `E0599` — method not found, because its trait is not in scope

```text
error[E0599]: no method named `summary` found for struct `AnimeSeries` in the current scope
  --> phase2-intermediate\03-traits-and-generics\01-defining-and-implementing-traits\examples\05-trait-not-in-scope.rs:25:31
   |
 7 |         fn summary(&self) -> String;
   |            ------- the method is available for `AnimeSeries` here
...
10 |     pub struct AnimeSeries {
   |     ---------------------- method `summary` not found for this struct
...
25 |     println!("{}", death_note.summary());
   |                               ^^^^^^^ method not found in `AnimeSeries`
   |
   = help: items from traits can only be used if the trait is in scope
help: trait `Summarize` which provides `summary` is implemented but not in scope; perhaps you want to import it
   |
 5 + use crate::catalog::Summarize;
   |

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is objecting to:** `AnimeSeries` is fully `pub`, and the compiler itself says "the method is available for `AnimeSeries` here" — but because `Summarize` was never imported, it won't let you call it. It even names the exact `use` you're missing.

**The fix:** add the compiler's own suggestion:

```rust
use catalog::Summarize;
```

**Why this is the fix:** the method existing is not enough; the trait it is defined on has to be in scope too. That rule holds even when you take the type from another module — which is exactly what the compiler's message is telling you directly.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
trait Greet {
    fn hello(&self) -> String;
}
```

</details>

<details>
<summary>Answer</summary>

Yes. A trait on its own needs no implementor at all. Nothing here has promised `Greet` yet — and that is entirely fine; Rust does not force anything to accept a contract just because it exists.

</details>

<details>
<summary>What does this print?</summary>

```rust
trait Greet {
    fn name(&self) -> String;
    fn hello(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Robot;

impl Greet for Robot {
    fn name(&self) -> String {
        "R2".to_string()
    }
}

println!("{}", Robot.hello());
```

</details>

<details>
<summary>Answer</summary>

```text
Hello, R2!
```

`Robot` never wrote its own `hello`, so the default runs — and that default calls `name`, which is `Robot`'s own version.

</details>

<details>
<summary>Does this compile?</summary>

```rust
trait Greet {
    fn name(&self) -> String;
    fn hello(&self) -> String {
        format!("Hello, {}!", self.name())
    }
}

struct Cat;

impl Greet for Cat {
    fn hello(&self) -> String {
        "meow".to_string()
    }
}
```

</details>

<details>
<summary>Answer</summary>

No. `Cat` chose to override `hello` (which has a default) — that is entirely allowed. But it never wrote `name` (which is required), and overriding the optional method does not stand in for the required one. You get an error like the one you saw in "Errors you will meet".

</details>

<details>
<summary>Does this compile?</summary>

```rust
mod shapes {
    pub trait Area {
        fn area(&self) -> f64;
    }

    pub struct Square {
        pub side: f64,
    }

    impl Area for Square {
        fn area(&self) -> f64 {
            self.side * self.side
        }
    }
}

fn main() {
    let sq = shapes::Square { side: 3.0 };
    println!("{}", sq.area());
}
```

</details>

<details>
<summary>Answer</summary>

No. `Square` is fully `pub` and you can construct it, but `Area` was never imported — not with `use shapes::Area;`, not with anything else.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/04-missing-required-method.rs` so it compiles — by writing the missing `title` method, not by deleting anything else.
2. Fix `examples/05-trait-not-in-scope.rs` with a `use`. Then try this: if you put that same `use` outside `fn main` but still in the same file, does it still work?

### Implement

Four pieces in `src/lib.rs`:

```sh
cargo test -p p2-03-01-defining-and-implementing-traits
```

`trait Summarize` itself is already fully written — exactly what you saw in "The concept". Your job is the `impl` blocks: `AnimeSeries`, `MangaVolume`, a new type called `GameTitle`, and one plain function, `shelf_summary`. The exact format for every overridden `summary` is written in the doc comment right above that method — don't guess.

### Build

Define a **new** trait — not `Summarize` — with at least one required method and one default method. Implement it for at least one of the types already in `src/lib.rs` (or a brand-new type of your own), and write at least one test for it.

An idea, if you want one: a `Rateable` trait with a required method returning a number from 0 to 5, and a default method that turns that number into a string of `★` characters.

### Challenge (optional)

Add a new struct, `Playlist`, with one field, `pub items: Vec<AnimeSeries>`. Implement `Summarize` for it: `title` returns something like "Playlist (N items)", and `summary` joins together every item's own summary — by calling `summary` on each of them — with a separator you pick yourself; just document it in the function's doc comment.

(This part looks ahead.) This only works for `Vec<AnimeSeries>` — not `Vec<MangaVolume>`, not a mix of both. A single function that works for a slice of *any* type that has `Summarize` is exactly what generics ([2.3.2](../02-generic-functions-and-structs/README.md)) solve.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Trait | A contract of method signatures; no data | Any time several unrelated types need to share one behavior |
| Required method | A trait method with no body; every implementor must write its own | `title` in this lesson |
| Default method | A trait method with a body; free unless overridden | `summary` in this lesson |
| `impl Trait for Type` | The syntax that fulfills a contract for one concrete type | Every implementation in this lesson |
| Importing a trait | Bringing a trait into scope with `use` so its methods become callable | Any time the type comes from another module |

### What you now know

- A trait is only method signatures — no fields, no data — and that is exactly what separates it from a base class.
- A method with no body is required; a method with a body is a default, and overriding it is optional.
- A default body can call required methods that have no concrete type behind them yet, because the trait itself guarantees it.
- Two completely unrelated types, once they implement one shared trait, answer to an identical call afterward — with no inheritance, no shared field.
- The compiler checks every trait implementation block at compile time; a missing required method stops the build right there.
- Calling a trait's method requires the trait itself to be imported — even when the type is fully `pub` and reachable.

### What comes back later

- **Generic functions and structs, bounded by a trait you wrote yourself** — [2.3.2 — Generic functions and structs](../02-generic-functions-and-structs/README.md)
- **Associated types — `Iterator` from 2.2.4 already had one, `type Item`** — [2.3.5 — Associated types](../05-associated-types/README.md)
- **Deriving standard traits automatically instead of hand-writing every implementation** — [2.3.4 — The standard derives, implemented by hand](../04-standard-derives-by-hand/README.md)
- **Trait objects, and how the compiler actually decides which implementation runs** — [2.3.7 — Static versus dynamic dispatch](../07-static-vs-dynamic-dispatch/README.md)

### Can you explain?

- Why can't a trait hold a field, the way a struct can?
- What's the difference between a required method and a default method, and how does the compiler decide which one runs for a given implementor?
- Why does `summary`'s default body compile, even though it calls `title` and that method has no body anywhere in the trait?
- Why does `AnimeSeries` answer to a call to `summary`, even though it shares nothing with `MangaVolume`?
- Why did calling `summary` fail in the "a trait must be in scope" example, even though `AnimeSeries` was fully `pub`?
- Compare this to a Python `ABC` or `Protocol` — exactly where does that comparison break down?

---

## Going further

- [The Rust Book — Traits: Defining Shared Behavior](https://doc.rust-lang.org/book/ch10-02-traits.html) — the same ground, official and complete.
- [The Rust Reference — Traits](https://doc.rust-lang.org/reference/items/traits.html) — the exact syntax and rules.
- [Rust by Example — Traits](https://doc.rust-lang.org/rust-by-example/trait.html) — more, shorter examples.
- [`rustc --explain E0046`](https://doc.rust-lang.org/error_codes/E0046.html) and [`rustc --explain E0599`](https://doc.rust-lang.org/error_codes/E0599.html) — the official explanation of both errors you saw today.
