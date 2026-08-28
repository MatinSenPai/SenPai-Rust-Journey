# 2.4.3 — `Deref`, `AsRef`, `Borrow`, `ToOwned`

## At a glance

After this lesson you can:

- Implement `Deref` and `DerefMut` for a newtype wrapper of your own, so it behaves like its inner type both at method-call sites and through the `*` operator — and say that `Box`, `String`, and `Vec` all lean on exactly this mechanism.
- Write a function that takes `impl AsRef<str>` instead of one fixed type, and explain why calling it with a `&str`, a `String`, or (for `AsRef<Path>`) a `PathBuf` costs the caller nothing extra.
- Say why `Borrow<T>` is a stronger contract than `AsRef<T>`, and state exactly what breaks in a `HashMap` when a `Borrow` impl disagrees with its own `Hash`/`Eq`.
- Turn a borrowed `&str` into an owned `String` with `ToOwned`, not `Clone`, and say why `str` needs `ToOwned` at all.

**Time:** ~90 minutes · **Prerequisites:**
[2.4.2 — Lifetimes in structs and methods](../02-lifetimes-in-structs-and-methods/README.md),
[2.3.1 — Defining and implementing traits](../../03-traits-and-generics/01-defining-and-implementing-traits/README.md),
[2.3.6 — Supertraits and blanket impls](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.md),
[2.1.2 — `HashMap` in depth](../../01-collections/02-hashmap-in-depth/README.md)

---

## Why this matters

Since Phase 1, you have written a function that takes `&str` and, without thinking twice, handed it a `&String` — and it worked. This course's glossary has had a name for that since those early days: **Deref coercion**, the compiler turning `&String` into `&str` at a call site. Until today it was just an accepted fact. Today you find out exactly which trait sits behind it.

[2.1.2](../../01-collections/02-hashmap-in-depth/README.md) went one step further: you built a `HashMap<String, V>`, then `.get()`-ed it with a plain `&str` literal — without ever building a fresh `String` just to ask the question. That lesson said this was not a special case carved out of `HashMap`'s own code; a more general rule sits behind it, a trait called `Borrow`, and it asked you to simply trust that it works until you reached "Phase 2.4" — here. This lesson pays that promise off exactly: it shows you the mechanism behind `Borrow`, and says why it is a good deal more specific than ordinary auto-deref.

Add to that every method you have ever called on a `Box<T>` or a `Vec<T>` without asking "which type actually owns this method?", and all of it traces back to the same source: a handful of small traits in the standard library, each doing one narrow job, riding along underneath nearly everything in Rust. Today you see four of them from both sides — you learn exactly why something you've relied on for months works, and you learn to build the same ergonomics into types of your own.

---

## The concept

### `Deref`: how a type gets to behave like a pointer

A plain newtype wrapper around a `Vec<String>`:

```rust
use std::ops::Deref;

struct Watchlist(Vec<String>);

impl Deref for Watchlist {
    type Target = Vec<String>;

    fn deref(&self) -> &Vec<String> {
        &self.0
    }
}
```

Without this `impl`, `Watchlist` has no methods at all — no `.len()`, no indexing, nothing; `struct Watchlist(Vec<String>);` is just a one-field tuple, even though that one field happens to be a `Vec`. The `Deref` trait wants exactly one method: `deref(&self) -> &Self::Target`, where `Target` says "when someone goes through me, what do they land on?" That one `impl` changes everything:

```rust
let list = Watchlist(vec!["Frieren".to_string(), "Bocchi the Rock".to_string()]);

println!("count: {}", list.len());
println!("first: {}", list[0]);
println!("via *: {}", (*list).len());
```

```text
count: 2
first: Frieren
via *: 2
```

Three lines, three faces of the same mechanism. `list.len()` works because method lookup checks `Watchlist` first, finds no `len`, then the compiler tries `*list` on its own — calling `Deref::deref(&list)` — and finds `len` there, on `Vec<String>`. This is what the glossary already called **auto-deref**; now you know exactly what it calls. `list[0]` takes the same road, because indexing is a trait too (`Index`), one `Watchlist` doesn't have but `Vec<String>` does. And `(*list)` is the explicit dereference itself — the same `*` you met in [1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md), just aimed at a type of your own this time.

### `DerefMut`: the same mechanism, for writing

`Deref` only ever hands back a `&Target` — fine for reading, but not for something like `.push()`, which needs `&mut self`. The `DerefMut` trait adds exactly that, with one more method:

```rust
impl DerefMut for Watchlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}
```

Its own definition, `trait DerefMut: Deref`, is a **supertrait** — the same pattern you saw in [2.3.6](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.md): you cannot implement `DerefMut` without `Deref`, because `DerefMut` leans on the very `Target` that `Deref` already defined. Now `Watchlist` can be written through too:

```rust
let mut list = Watchlist(vec!["Frieren".to_string()]);
list.push("Trigun".to_string());
println!("after push: {}", list.len());

let inner: &Vec<String> = &list;
println!("coerced: {inner:?}");
```

```text
after push: 2
coerced: ["Frieren", "Trigun"]
```

`list.push(...)` walks the exact same path as `list.len()` did, except the compiler needs a `&mut` this time and gets it from `DerefMut::deref_mut`. The last line shows something else: `&list` (a `&Watchlist`) is handed straight to a spot expecting `&Vec<String>` — no `*`, no explicit call to anything. That is Deref coercion again, this time on a type you wrote yourself.

```senpai-visual
{"kind":"concept","labels":["list.push(x)","no push on Watchlist","auto-deref via DerefMut","push on Vec<String>"]}
```

### The mechanism behind what you already knew from Phase 1

That `&String -> &str` you saw over and over in Phase 1 was never magic — the standard library wrote `impl Deref<Target = str> for String` exactly once, the same way you just wrote one for `Watchlist` above, and you have been riding on that single `impl` since day one:

```rust
fn print_str(s: &str) {
    println!("str: {s}");
}

fn print_slice(items: &[String]) {
    println!("slice len: {}", items.len());
}
```

```rust
let owned = String::from("hello");
print_str(&owned);

let list = Watchlist(vec!["Frieren".to_string(), "Bocchi the Rock".to_string()]);
print_slice(&list);
```

```text
str: hello
slice len: 2
```

The second line is the more interesting one: `print_slice` wants a `&[String]`, but `&list` is a `&Watchlist`. The compiler takes **two** auto-deref steps, not one: first `Watchlist -> Vec<String>` (the `impl` you wrote), then `Vec<String> -> [String]` (another `impl Deref`, this one written by `Vec<T>` itself, inside the standard library). Deref coercion chains — it takes as many hops as there are `impl Deref`s lined up back to back.

```senpai-visual
{"kind":"concept","labels":["&Watchlist","hop 1: Deref to &Vec<String>","hop 2: Deref to &[String]","print_slice(&[String])"]}
```

`Box<T>` carries the exact same trait, which is why a `Box<String>` behaves like a `String` without a single method written on it explicitly. The full story of `Box` — heap allocation, ownership — belongs to [2.6.1](../../06-smart-pointers/01-box-and-heap-allocation/README.md); for now it's enough to know that `Box`, `String`, and `Vec` all get the "behave like what I wrap" ergonomics from this one trait, not from three separate mechanisms.

### `AsRef<T>`: a cheap view as `&T`

`Deref` says "treat me as *the* thing I wrap, everywhere" — a whole-type, all-the-time contract. Sometimes you want something smaller: one particular function, saying, just at that one call site, "give me anything that can cheaply become a `&str`." That is exactly `AsRef<T>` — a separate trait, with just this one method:

```rust
fn shout(name: impl AsRef<str>) -> String {
    format!("{}!", name.as_ref().to_uppercase())
}
```

As [2.3.7](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) showed you, `impl AsRef<str>` in a parameter position is exactly a generic bound, just spelled shorter — you could have written `fn shout<T: AsRef<str>>(name: T) -> String` instead. One body, three calling shapes:

```rust
println!("{}", shout("frieren"));

let owned = String::from("bocchi");
println!("{}", shout(&owned));
println!("still own it: {owned}");
println!("{}", shout(owned));
```

```text
FRIEREN!
BOCCHI!
still own it: bocchi
BOCCHI!
```

A `&str` literal, a borrowed `&String`, and an owned `String` moved straight in — all three go through the same `.as_ref()` with no extra allocation; `str` and `String` both already implement `AsRef<str>`. This is exactly why so many standard-library functions take `impl AsRef<Path>` instead of one fixed type — `std::fs::read_to_string`, for one. The same function body, unchanged, works with a `&str`, a `String`, or a `PathBuf`, because all three implement `AsRef<Path>` — the identical pattern you just saw with `shout`, just aimed at `Path` instead of `str`.

### `Borrow<T>`: `AsRef`'s shape, a stricter contract

This is the moment [2.1.2](../../01-collections/02-hashmap-in-depth/README.md) promised would come back. `Borrow<T>`'s own definition reads almost word for word like `AsRef<T>`'s:

```rust
trait AsRef<T: ?Sized> {
    fn as_ref(&self) -> &T;
}

trait Borrow<T: ?Sized> {
    fn borrow(&self) -> &T;
}
```

The difference is not in the signature — it is in a **contract** the compiler never checks for you: `Borrow`'s own documentation states that `Hash`, `Eq`, and `Ord` must be equivalent for borrowed and owned values — `x.borrow() == y.borrow()` should give the same result as `x == y`. This is the exact trait `HashMap::get` is built on — its real signature reads `fn get<Q>(&self, k: &Q) -> Option<&V> where K: Borrow<Q>, Q: Hash + Eq + ?Sized`. You can write a small version of that same signature yourself:

```rust
fn find_by_ref<'a, K, Q, V>(map: &'a HashMap<K, V>, key: &Q) -> Option<&'a V>
where
    K: Borrow<Q> + Hash + Eq,
    Q: Hash + Eq + ?Sized,
{
    map.get(key)
}
```

And the same `ratings` map [2.1.2](../../01-collections/02-hashmap-in-depth/README.md) built, once more:

```rust
let mut ratings: HashMap<String, u8> = HashMap::new();
ratings.insert(String::from("Frieren"), 10);
ratings.insert(String::from("Bocchi the Rock"), 9);

println!("{:?}", find_by_ref(&ratings, "Frieren"));
```

```text
Some(10)
```

`String: Borrow<str>` holds — the standard library wrote that `impl` precisely because a `String` and a `&str` holding the same text hash and compare equal the same way. That's why `find_by_ref(&ratings, "Frieren")` compiles and does the right thing: it finds the key with a plain `&str` literal, no fresh `String` required.

Now the real question: why is this contract so strict? Take a type that claims to be case-insensitive:

```rust
struct CiKey(String);

impl PartialEq for CiKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_lowercase() == other.0.to_lowercase()
    }
}
impl Eq for CiKey {}
```

```rust
impl Hash for CiKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_lowercase().hash(state);
    }
}

impl Borrow<str> for CiKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}
```

`Hash` and `Eq` both lowercase the string before comparing — reasonable so far. But `borrow()` hands back the **original** string, un-lowercased. A deliberate mismatch. Run it:

```rust
let mut ratings: HashMap<CiKey, u8> = HashMap::new();
ratings.insert(CiKey("Frieren".to_string()), 10);

println!("len: {}", ratings.len());
println!("get(\"Frieren\"): {:?}", ratings.get("Frieren"));
println!("get(\"frieren\"): {:?}", ratings.get("frieren"));
```

```text
len: 1
get("Frieren"): None
get("frieren"): None
```

The key is unquestionably there — `len()` says 1. But no spelling, not the exact `"Frieren"`, not lowercase `"frieren"`, finds it. Here's why: at `.insert()` time, `HashMap` builds the hash with `CiKey::hash` — which lowercases and hashes `"frieren"`. At `.get("Frieren")` time, `HashMap` builds the hash with `str::hash` instead — because the query type is `str` this time, not `CiKey` — and that hashes the untouched `"Frieren"`; a completely different number, so the lookup does not even reach the right bucket. `.get("frieren")` does reach the right bucket (its hashed bytes match what was hashed at insert time), but then a plain `str` equality check follows: does the stored string (`"Frieren"`, exactly what `borrow()` handed back, un-lowercased) equal the query (`"frieren"`)? No — this comparison no longer uses `CiKey`'s lowercasing `Eq`, it uses ordinary `str::eq`. Result: a key that is unquestionably present by its own `Eq` becomes unreachable through `.get()`, under any spelling at all.

```senpai-visual
{"kind":"concept","labels":["insert hashes via CiKey (lowercase)","get(Frieren) hashes via str (exact)","different bucket","key unreachable"]}
```

This is exactly the example `Borrow`'s own documentation uses too — a case-insensitive key that must not implement `Borrow<str>`, for exactly this reason; it should reach for `AsRef<str>` instead, which makes no such promise. `Handle`, in this lesson's exercises, asks you for the correct version of the same idea.

### `ToOwned`: `Clone`, generalized for when the owned type differs

`str` is an **unsized type** — the term you already know from [1.4.1](../../../phase1-fundamentals/04-text-and-strings/01-string-vs-str/README.md): its size is never known at compile time, so you can never hold one directly, only behind a reference or a `Box`. That means `Clone::clone(&self) -> Self` makes no sense at all for `str` — it would have to return a `str` **by value**, and returning something unsized by value does not compile. So `str: Clone` does not hold, and trying your Phase 1 habit on a `&str` shows the trap:

```rust
let borrowed: &str = "Frieren";
let still_borrowed: &str = borrowed.clone();
println!("still borrowed: {still_borrowed}");
```

The compiler itself flags this, right here:

```text
warning: call to `.clone()` on a reference in this situation does nothing
  --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\06-toowned-str-and-slice.rs:10:40
   |
10 |     let still_borrowed: &str = borrowed.clone();
   |                                        ^^^^^^^^ help: remove this redundant call
   |
   = note: the type `str` does not implement `Clone`, so calling `clone` on `&str` copies the reference, which does not do anything and can be removed
   = note: `#[warn(noop_method_call)]` on by default
```

It compiles, it runs, and `still_borrowed` is still a `&str` — because `&str` itself is `Copy`, so `.clone()` just copies the reference, not the text it points at. No `String` was built; just another address pointing at the same text.

The right tool is the `ToOwned` trait (`Clone`, generalized):

```rust
trait Clone {
    fn clone(&self) -> Self;
}

trait ToOwned {
    type Owned: Borrow<Self>;
    fn to_owned(&self) -> Self::Owned;
}
```

The whole difference is those two lines: `Clone::clone` has to return the exact same type (`Self`); `ToOwned::to_owned` has its own associated type, `Owned`, which does not have to be `Self` — and its own definition even says `Owned: Borrow<Self>`, meaning whichever owned type you pick has to be borrowable back into `Self`. For `str`, that `Owned` is `String` — the exact `impl Borrow<str> for String` you used in the section above, now part of `ToOwned`'s own contract too:

```rust
let owned: String = borrowed.to_owned();
println!("owned: {owned}");

let numbers: &[i32] = &[1, 2, 3];
let owned_numbers: Vec<i32> = numbers.to_owned();
println!("owned_numbers: {owned_numbers:?}");
```

```text
owned: Frieren
owned_numbers: [1, 2, 3]
```

The same pattern works for `[T]` too — another unsized slice, with `Vec<T>` as its `Owned`. And for types that genuinely do implement `Clone` (most of what you've written so far), you've never had to think about `ToOwned` because a blanket impl — the same pattern from [2.3.6](../../03-traits-and-generics/06-supertraits-blanket-impls-orphan-rule/README.md) — already turns every `Clone` into a `ToOwned` for free: `impl<T: Clone> ToOwned for T { type Owned = T; ... }`. `str` is the exception, not the rule — precisely because `Clone` cannot be defined on it at all.

### Decision: `Deref`, `AsRef`, or `Borrow`?

| Trait | Reach for it as a parameter bound when... | Implement it for your own type when... |
|---|---|---|
| `Deref`/`DerefMut` | almost never written directly as a bound | your type genuinely **is** the thing it wraps — a smart pointer, a transparent wrapper |
| `AsRef<T>` | one function body should accept whatever the caller hands over — `&str`/`&Path`/... — with no copy | your type can cheaply produce a `&T` view, for a caller who explicitly asks for exactly that |
| `Borrow<T>` | you rarely write it yourself; `HashMap`/`HashSet`/`BTreeMap` use it behind the scenes | your type is meant to be a collection's key **and** you can genuinely promise its `Hash`/`Eq`/`Ord` stay consistent with the borrowed form |

A shorter rule: if you just want one function, at one call site, to be flexible, reach for `AsRef<T>`. If you're building a type that genuinely *is* the thing it wraps, implement `Deref`. Only implement `Borrow<T>` when you can actually keep that strict `Hash`/`Eq`/`Ord` promise — otherwise, exactly like `CiKey` above, `AsRef<T>` is the safer choice.

---

## Hands on

```sh
cargo run -p p2-04-03-deref-asref-borrow --example 01-deref-and-derefmut
cargo run -p p2-04-03-deref-asref-borrow --example 02-deref-coercion-chain
cargo run -p p2-04-03-deref-asref-borrow --example 03-asref-generic-param
cargo run -p p2-04-03-deref-asref-borrow --example 04-borrow-hashmap-payoff
cargo run -p p2-04-03-deref-asref-borrow --example 05-borrow-contract-violation
cargo run -p p2-04-03-deref-asref-borrow --example 06-toowned-str-and-slice
```

Then the four broken ones:

```sh
cargo run -p p2-04-03-deref-asref-borrow --example 07-derefmut-missing-broken --features broken
cargo run -p p2-04-03-deref-asref-borrow --example 08-asref-wrong-return-type-broken --features broken
cargo run -p p2-04-03-deref-asref-borrow --example 09-hashmap-get-wrong-query-broken --features broken
cargo run -p p2-04-03-deref-asref-borrow --example 10-toowned-unsized-return-broken --features broken
```

Then try these:

1. In `05-borrow-contract-violation.rs`, change `borrow()` so it also lowercases the string: `fn borrow(&self) -> &str { &self.0.to_lowercase() }`. Does it compile? If not, read the compiler's message — why does even this simplest possible fix fail? (Hint: `.to_lowercase()` itself builds a fresh `String`, which only lives until that call to `borrow()` ends.)
2. In `06-toowned-str-and-slice.rs`, try `numbers.clone()` instead of `numbers.to_owned()` — and drop the `: Vec<i32>` type annotation too, or the mismatch between what `.clone()` actually returns (`&[i32]`) and what you told the compiler to expect will hide the warning behind a hard error instead. With the annotation gone, does it compile? What warning do you see, and why is it the same warning as the `&str` case?
3. In `04-borrow-hashmap-payoff.rs`, call `find_by_ref(&ratings, &5)` (a `&i32` instead of a `&str`). Which error do you see, and is it exactly the one from "Errors you will meet"?

---

## Errors you will meet

### `E0596` — a missing `DerefMut`

```text
error[E0596]: cannot borrow data in dereference of `Watchlist` as mutable
  --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\07-derefmut-missing-broken.rs:24:5
   |
24 |     list.push("Trigun".to_string());
   |     ^^^^ cannot borrow as mutable
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Watchlist`

For more information about this error, try `rustc --explain E0596`.
```

**What the compiler is actually objecting to:** `Watchlist` implements `Deref` but not `DerefMut`. `.push()` needs a `&mut Vec<String>`; auto-deref can get a `&Vec<String>` (read-only) from `Deref::deref`, but has no way to a mutable one, because that step only exists on `DerefMut`. The compiler's own message says exactly this.

**The fix:** implement `DerefMut` too:

```rust
impl DerefMut for Watchlist {
    fn deref_mut(&mut self) -> &mut Vec<String> {
        &mut self.0
    }
}
```

**Why this is the fix:** auto-deref now has a mutable step to take; `.push()` follows the exact same path `.len()` did, just with a `&mut` from `deref_mut` this time instead of a `&` from `deref`.

### `E0308` — an `AsRef<str>` that returns the wrong type

```text
error[E0308]: mismatched types
  --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\08-asref-wrong-return-type-broken.rs:13:9
   |
12 |     fn as_ref(&self) -> &str {
   |                         ---- expected `&str` because of return type
13 |         &self.0
   |         ^^^^^^^ expected `&str`, found `&Vec<String>`
   |
   = note: expected reference `&str`
              found reference `&Vec<String>`

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is actually objecting to:** `AsRef<str>` promises `as_ref` returns a `&str`. `Watchlist` wraps a `Vec<String>`, and `&self.0` is a `&Vec<String>` — not a `&str`, and there is no implicit conversion between the two.

**The fix:** match the target type to what `Watchlist` actually has:

```rust
impl AsRef<[String]> for Watchlist {
    fn as_ref(&self) -> &[String] {
        &self.0
    }
}
```

**Why this is the fix:** `Watchlist` is a `Vec<String>`, not a string — no meaningful `&str` comes out of it without building a fresh string (which is no longer "cheap," and `AsRef` is supposed to be cheap). `AsRef<[String]>` is a target that genuinely matches the type's contents — the same thing `Deref` already gave it for free.

### `E0277` — a key `HashMap::get` cannot `Borrow` into

```text
error[E0277]: the trait bound `String: Borrow<{integer}>` is not satisfied
    --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\09-hashmap-get-wrong-query-broken.rs:14:34
     |
  14 |     println!("{:?}", ratings.get(&5));
     |                              --- ^^ the trait `Borrow<{integer}>` is not implemented for `String`
     |                              |
     |                              required by a bound introduced by this call
     |
help: the trait `Borrow<{integer}>` is not implemented for `String`
      but trait `Borrow<str>` is implemented for it
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\alloc\src\str.rs:229:1
     |
 229 | impl Borrow<str> for String {
     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
     = help: for that trait implementation, expected `str`, found `{integer}`
note: required by a bound in `HashMap::<K, V, S, A>::get`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\std\src\collections\hash\map.rs:1037:12
     |
1035 |     pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
     |            --- required by a bound in this associated function
1036 |     where
1037 |         K: Borrow<Q>,
     |            ^^^^^^^^^ required by this bound in `HashMap::<K, V, S, A>::get`

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually objecting to:** `HashMap<String, u8>::get<Q>` only exists when `String: Borrow<Q>`. The standard library only wrote that for `Q = str` (and, trivially, `Q = String`) — never for a number. `&5` means `Q` is an integer type, and `String: Borrow<{integer}>` was never written. The compiler's message even points straight at the exact `impl Borrow<str> for String` line inside the standard library itself.

**The fix:** query with a type `String` can actually `Borrow` into:

```rust
let ratings: HashMap<String, u8> = HashMap::new();
println!("{:?}", ratings.get("5"));
```

**Why this is the fix:** `"5"` is a `&str` this time, not a number; `String: Borrow<str>` holds, so `Q = str` satisfies the `K: Borrow<Q>` bound and this compiles (the map itself is empty, so the answer is `None` — but now it's a real answer, not a type error).

### `E0277` — a return type that isn't `Sized`

```text
error[E0277]: the size for values of type `str` cannot be known at compilation time
 --> phase2-intermediate\04-lifetimes-and-conversion\03-deref-asref-borrow\examples\10-toowned-unsized-return-broken.rs:9:22
  |
9 | fn widen(s: &str) -> str {
  |                      ^^^ doesn't have a size known at compile-time
  |
  = help: the trait `Sized` is not implemented for `str`
  = note: the return type of a function must have a statically known size

For more information about this error, try `rustc --explain E0277`.
```

**What the compiler is actually objecting to:** a function's return type must have a size known at compile time — the exact same rule that makes `str::clone(&self) -> Self` impossible too. `s.to_owned()` here builds a `String`, but the signature promises `str`; even though the body would produce something sensible, the signature itself is invalid from the start.

**The fix:** make the return type what `ToOwned` actually produces:

```rust
fn widen(s: &str) -> String {
    s.to_owned()
}
```

**Why this is the fix:** `String` has a size known at compile time (a pointer, a length, a capacity — three numbers, on the stack). That is exactly what `ToOwned::Owned` is meant to be for `str`; the signature now matches what `to_owned()` actually delivers.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
fn takes_str(s: &str) {
    println!("{s}");
}

let owned = String::from("hi");
takes_str(&owned);
```

</details>

<details>
<summary>Answer</summary>

Yes. This is the same Deref coercion you already know from Phase 1 — now you know it's `impl Deref<Target = str> for String`, written by the standard library, not by you.

</details>

<details>
<summary>If <code>Watchlist</code> only has <code>Deref</code> (no <code>DerefMut</code>), does this compile?</summary>

```rust
let mut list = Watchlist(vec!["a".to_string()]);
list.push("b".to_string());
```

</details>

<details>
<summary>Answer</summary>

No. `.push()` needs a `&mut Vec<String>`, and without `DerefMut`, auto-deref has no way to a mutable one. The error code is `E0596`.

</details>

<details>
<summary>What does this print?</summary>

```rust
let s: &str = "hi";
let c = s.clone();
println!("{}", std::mem::size_of_val(&c) == std::mem::size_of_val(&s));
```

</details>

<details>
<summary>Answer</summary>

```text
true
```

`c` and `s` are both `&str`, with exactly the same size — because `.clone()` on a `&str` only copies the reference, not whatever it points at. The compiler also flags this line with a `noop_method_call` warning.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let ratings: HashMap<String, u8> = HashMap::new();
println!("{:?}", ratings.get(&5));
```

</details>

<details>
<summary>Answer</summary>

No. `String: Borrow<i32>` is never written anywhere — the standard library only gives `String` a `Borrow<str>`. The error code is `E0277`.

</details>

<details>
<summary>A type of your own is case-insensitive — its <code>Hash</code> and <code>Eq</code> lowercase the string first. To make it lookup-able by a plain <code>&str</code> too, which trait do you implement: <code>Borrow&lt;str&gt;</code> or <code>AsRef&lt;str&gt;</code>?</summary>

</details>

<details>
<summary>Answer</summary>

`AsRef<str>`. `Borrow<str>` promises the borrowed form's (the untouched string's) `Hash`/`Eq` stays consistent with the owned form's (the lowercased one's) — and here it does not. `AsRef<str>` makes no such promise, so you aren't lying.

</details>

### Repair

Fix all four broken examples:

1. `examples/07-derefmut-missing-broken.rs` — implement `DerefMut` for `Watchlist`.
2. `examples/08-asref-wrong-return-type-broken.rs` — change the `AsRef` target from `str` to something `Watchlist` actually has.
3. `examples/09-hashmap-get-wrong-query-broken.rs` — call `.get()` with a key `String` can actually `Borrow` into.
4. `examples/10-toowned-unsized-return-broken.rs` — change `widen`'s return type to something `Sized`.

### Implement

Four pieces in `src/lib.rs`, each one trait:

```sh
cargo test -p p2-04-03-deref-asref-borrow
```

- `Playlist` — implement `Deref` and `DerefMut`.
- `DisplayName` — implement `AsRef<str>`.
- `Handle` — implement `Borrow<str>` so it stays consistent with the `Hash`/`Eq` already derived on it.
- `longest_word_owned` — read its spec carefully; the tie-breaking rule (the last word of equal length wins) is exactly what `Iterator::max_by_key` itself does on a tie.

### Build

Build a newtype wrapper of your own around something simple (a `String`, a `Vec<T>`, anything). Implement one of these four traits for it — whichever one genuinely fits that type — and write a comment saying why you picked that one and not another. Add at least two tests.

### Challenge (optional)

This lesson showed you a value can be "borrowed" or "owned," and that `ToOwned` is the bridge between the two. The next lesson, [2.4.4](../04-cow-and-clone-on-write/README.md), builds a type that decides between the two for itself, at run time. Without using that type — using only what you learned today — build an `enum` with two variants, one for the borrowed case (a `&'a str` field) and one for the owned case (a `String` field), plus a method on that `enum` that returns a `&str` no matter which variant it is. If you build this, you have effectively built the shape of `Cow` by hand.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| `Deref` / `DerefMut` | make `*` and auto-deref possible; `deref(&self) -> &Target`, and `DerefMut: Deref` adds the `&mut` version | a wrapper type that should behave like the type it wraps |
| Chained deref coercion | several `impl Deref`s back to back, at one call site | `&Watchlist -> &Vec<String> -> &[String]`, `Box<T> -> T` |
| `AsRef<T>` | a trait for "can be cheaply viewed as `&T`"; `fn as_ref(&self) -> &T` | a function parameter that should accept `&str`/`String`/`PathBuf`/... alike |
| `Borrow<T>` | `AsRef`'s shape, with a contract: `Hash`/`Eq`/`Ord` must stay consistent between the borrowed and owned forms | `HashMap`/`HashSet`/`BTreeMap` keys that also need to be looked up by their borrowed form |
| `ToOwned` | `Clone`, generalized for unsized types; `to_owned(&self) -> Self::Owned`, and `Owned` need not be `Self` | `str -> String`, `[T] -> Vec<T>` |

### What you now know

- `Deref`/`DerefMut` need one `deref`/`deref_mut` method each, and they are what auto-deref and Deref coercion actually call — the same mechanism `Box`, `String`, and `Vec` use, and coercion can chain across several `impl`s at once.
- `AsRef<T>` is a cheap `&T` view that lets a function accept several different input shapes with one body, at no extra allocation.
- `Borrow<T>` has `AsRef<T>`'s exact shape, but adds a promise the compiler never checks: `Hash`/`Eq`/`Ord` must stay consistent between the borrowed and owned forms; breaking it builds a key that exists but cannot be found.
- `ToOwned` exists exactly where `Clone` cannot — unsized types like `str` — because its owned type (`Owned`) does not have to be `Self`.
- On the choice itself: reach for `AsRef<T>` for a flexible parameter, implement `Deref` for a type that genuinely is the thing it wraps, and only implement `Borrow<T>` when you can actually keep its strict promise.

### What comes back later

- **`Box<T>` and heap allocation, in full** — [2.6.1 — `Box` and heap allocation](../../06-smart-pointers/01-box-and-heap-allocation/README.md)
- **`Cow<'_, str>`: choosing between borrowed and owned, at run time** — [2.4.4](../04-cow-and-clone-on-write/README.md)

### Can you explain?

- Why does `list.len()` compile when `Watchlist` has no `len` method of its own?
- Explain Deref coercion in a way that includes the fact that it chains.
- `AsRef<T>` and `Deref` look similar on a simple wrapper. In your own words, say what different question each one answers.
- Why does a `Borrow` impl that disagrees with its own `Hash`/`Eq` compile fine but misbehave at run time?
- Why isn't `str: Clone`, and what exactly does `ToOwned` add on top of `Clone` that fixes that?

---

## Going further

- [`std::ops::Deref` documentation](https://doc.rust-lang.org/std/ops/trait.Deref.html) — including the "Deref coercion" section that formally explains the chaining you saw.
- [`std::convert::AsRef` documentation](https://doc.rust-lang.org/std/convert/trait.AsRef.html)
- [`std::borrow::Borrow` documentation](https://doc.rust-lang.org/std/borrow/trait.Borrow.html) — the same place the case-insensitive-key example, nearly identical to `CiKey` above, comes from.
- [`std::borrow::ToOwned` documentation](https://doc.rust-lang.org/std/borrow/trait.ToOwned.html)
- [The Rust Book — `Deref` and `Box<T>`](https://doc.rust-lang.org/book/ch15-02-deref.html)
