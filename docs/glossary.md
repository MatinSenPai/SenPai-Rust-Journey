# Glossary

Plain-English definitions, added to as new jargon shows up in lessons. If a
term confuses you and it's not here yet, add it once you've figured it out —
future-you (and anyone else following this repo) will thank you.

- **Compiler** — a program that translates source code (Rust) into machine
  code (a binary your OS can run directly), *before* you run it. Contrast
  with Python, which is interpreted line-by-line at run time by `python3`.
- **Binary / executable** — the compiled output file `cargo build` produces
  (under `target/debug/` or `target/release/`). No Rust toolchain is needed to
  run it, unlike a `.py` file which always needs a Python interpreter present.
- **Crate** — Rust's unit of compilation and distribution — roughly
  equivalent to a Python "package" (a `pip`-installable thing with a name and
  version), but a crate can produce either a library (`lib.rs`) or an
  executable (`main.rs`).
- **Cargo** — Rust's build tool + package manager, playing the role of
  `pip` + `venv` + `setuptools` + a task runner, all in one CLI.
- **Workspace** — a group of crates that share one dependency lock file and
  one build output directory. This whole repo is one workspace.
- **Ownership** — Rust's core memory-management rule: every value has exactly
  one owner responsible for cleaning it up. No garbage collector, no manual
  `free()` — the compiler enforces this at compile time.
- **Move** — when a value's ownership transfers to a new variable, the old
  variable becomes invalid. Unlike Python, where `b = a` just adds another
  name pointing at the same object.
- **Borrow / reference (`&`, `&mut`)** — temporary, checked access to a value
  you don't own. You can have many read-only borrows, or exactly one mutable
  borrow, never both at once ("aliasing XOR mutability").
- **Lifetime** — the compiler's bookkeeping for *how long* a reference stays
  valid, written as `'a`. Almost always inferred; you only write it
  explicitly when the compiler can't figure out the relationship itself.
- **Lifetime elision** — the compiler's rules for inferring a reference's
  lifetime without you writing `'a`: each elided input reference gets its
  own, a lone input lifetime flows to every elided output, and on a method
  it's `&self`'s lifetime that flows there instead. Struct fields are the
  one place elision never reaches — always written out by hand.
- **`Option<T>`** — Rust's answer to "this might not have a value" — instead
  of `None`/`null` sneaking in anywhere, absence is an explicit type you must
  handle before you can use the value.
- **`Result<T, E>`** — Rust's answer to recoverable errors — a function that
  can fail returns `Result`, forcing the caller to handle both the success
  (`Ok`) and failure (`Err`) case, instead of relying on exceptions.
- **Trait** — Rust's version of an interface/protocol: a set of methods a
  type promises to implement. Similar in spirit to a Python Protocol or ABC,
  but resolved at compile time by default.
- **Required method** — a trait method with no body, ending in `;`. Every
  implementor must supply its own, or `impl Trait for Type` will not
  compile.
- **Default method** — a trait method with a body, written once inside the
  trait itself. Every implementor gets it for free, unless it overrides it
  with its own version.
- **Generic** — code written once, parameterized over a type, e.g.
  `fn largest<T>(list: &[T]) -> &T`. Compiled separately for each concrete
  type used (monomorphization) rather than resolved at runtime like Python's
  duck typing.
- **Associated type** — a placeholder type a trait declares and each
  implementor fills in exactly once, e.g. `Iterator`'s `type Item`. Unlike a
  generic parameter, a type can only implement the trait one way — there is
  one `Item` per `Iterator`, not a family of them to choose from at each call
  site.
- **Generic trait parameter** — a type parameter written on the trait itself,
  `trait Converts<T>`, rather than on a function or struct. Unlike an
  associated type, one type can implement the trait more than once — once
  per concrete `T` it supports, e.g. `Converts<f64>` and `Converts<String>`
  for the same type — at the cost of a call site sometimes needing an
  explicit type to say which `impl` it means.
- **`TryFrom` / `TryInto`** — the fallible sibling of `From`/`Into`:
  `try_from` returns `Result<U, Self::Error>` instead of handing back `U`
  directly, for a conversion that might have to be rejected (an
  out-of-range number, a string that fails validation). Implement
  `TryFrom<T> for U` and `TryInto<U> for T` arrives for free, the same way
  `Into` does for `From`.
- **Widening** — a numeric conversion into a type with more room, so it can
  never fail (`u8` into `i32`). Always infallible, always a `From`.
- **Narrowing** — a numeric conversion into a type with less room, so it
  might not fit (`i32` into `u8`). Always fallible, always a `TryFrom`.
- **Supertrait** — a trait that another trait names as a precondition,
  `trait B: A`: no type may implement `B` without having already implemented
  `A`. A trait-level dependency, not inheritance — nothing from `A` is
  shared or reused automatically, and you still write both `impl` blocks.
- **Blanket impl** — an `impl` written once for every type that satisfies a
  bound, `impl<T: Bound> Trait for T`, instead of one concrete type at a
  time. `Into` exists this way: the standard library wrote
  `impl<T, U> Into<U> for T where U: From<T>` a single time, and it covers
  every `From` impl anyone ever writes.
- **Orphan rule** — you may implement a trait for a type only if the trait
  or the type is defined in your own crate. It keeps two unrelated crates
  from ever writing conflicting impls of the same foreign trait for the
  same foreign type.
- **`unsafe`** — an escape hatch that lets you do a small set of operations
  the compiler can't verify are safe (raw pointer deref, calling C code,
  etc.), with the promise that *you've* verified it by hand. Most Rust code
  never needs it.
- **`async`/`await`, Future** — Rust's model for concurrent I/O-bound work: an
  `async fn` returns a `Future`, a value representing "work that will
  complete later," which does nothing until a runtime (like `tokio`) polls it.
- **Runtime (async)** — the scheduler that actually drives `Future`s to
  completion (spawns tasks, wakes them up when I/O is ready). Rust's standard
  library deliberately ships without one — you choose (almost always
  `tokio` for backend work).
- **`Arc`, `Mutex`** — `Arc` ("atomic reference count") lets multiple threads
  share ownership of a value; `Mutex` ensures only one thread can mutate it at
  a time. The combination (`Arc<Mutex<T>>`) is the most common way to share
  mutable state across threads/tasks.
- **Idempotency** — an operation that produces the same end result no matter
  how many times it's applied (e.g. "set balance to $10" vs. "add $10").
  Critical for retried network requests and job queues.
- **Backpressure** — a system's way of saying "slow down" to whatever is
  sending it work, instead of silently queuing forever or falling over.

## Memory and ownership

- **Destructor** — the code that runs as a value is destroyed. In Rust that is
  `Drop::drop`, and you never call it yourself; the compiler inserts the call
  at the closing brace of the owner's scope.
- **RAII** (resource acquisition is initialisation) — acquire the resource when
  the value is created, release it in the destructor. Scope closes the file,
  not discipline. It is why Rust has no `finally` block and does not need one.
- **Dereference (`*`)** — following a reference to the value at the other end.
  `*count += 1` changes the number; `count += 1` would try to change the arrow.
- **Auto-deref** — the compiler inserting `*` for you on a method call, which
  is why `text.len()` works whether `text` is a `String` or a `&String`.
- **`Rc<T>` / `Arc<T>`** — smart pointers for shared ownership: more than one
  variable can be a real, simultaneous owner of the same heap value, and it
  is only freed once the last one drops. Cloning one never copies the data —
  it increments a reference count and hands back a second pointer to the
  same allocation. `Rc` is single-threaded (a plain, non-atomic count);
  `Arc` ("atomically reference counted") is the thread-safe sibling, using
  atomic increments instead — slower per clone, but safe to share across
  threads. Both only ever hand out `&T`, never `&mut T`.
- **Aliasing rule** — any number of shared borrows, *or* exactly one mutable
  borrow, never both at once. The single rule the borrow checker enforces, and
  the reason a data race cannot be written in safe Rust.
- **Data race** — two threads touching the same memory at the same time with at
  least one of them writing. The aliasing rule makes it unrepresentable.
- **Iterator invalidation** — mutating a collection while walking it, so the
  walk is left pointing at memory that has moved. A run-time crash in C++ and a
  compile error here.
- **Borrow scope** — how long a borrow actually lasts: from where it is taken
  to its **last use**, not to the end of the block.
- **Non-lexical lifetimes (NLL)** — the rule that gives borrows that shorter,
  use-based scope. Before Rust 2018 a borrow lasted to the closing brace, and a
  great deal of correct code was rejected.
- **Two-phase borrow** — the compiler's allowance that makes
  `items.push(items.len())` legal: the arguments are evaluated before the
  mutable borrow becomes active.
- **Slice (`&[T]`, `&str`)** — a borrowed view of a contiguous run of values.
  Two words: where it starts and how many there are.
- **Fat pointer** — a reference carrying a second word alongside the address. A
  slice carries a length; a trait object carries a vtable.
- **Smart pointer** — a struct that behaves like a pointer (you can follow it
  to reach a value) but also owns what it points to, and can carry extra
  behaviour a plain reference can't. `Box<T>` is the simplest one.
- **`Box<T>`** — the simplest smart pointer: a value moved onto the heap, with
  a single owner that frees it when the `Box` itself drops. Its own size never
  depends on `T`'s size or contents — only on whether `T` is `Sized` (one
  word) or not (a fat pointer, still fixed).
- **Recursive type** — a type that (potentially) contains itself, e.g. an enum
  variant holding another value of the same enum. Without indirection this has
  no finite size, so the compiler rejects it outright; wrapping the recursive
  field in `Box<T>` fixes it, since `Box<T>` is always one pointer wide no
  matter what it points to.
- **Expression tree** — a recursive type shaped like `Num(f64) | Add(_, _) |
  Mul(_, _)`, where each operator variant holds its own operands. The classic,
  concrete example of a recursive type that actually earns its keep.
- **Boxed trait object (`Box<dyn Trait>`)** — a trait object (see Trait
  object) given an owner via `Box`, so it can be stored, returned, or put in a
  collection like `Vec<Box<dyn Trait>>` instead of only existing as a borrow.
- **`Weak<T>`** — a non-owning handle to a value managed by `Rc`/`Arc`. Holding
  one does not keep the value alive and does not count toward the strong
  count; `.upgrade()` is the only way to reach the value, returning
  `Option<Rc<T>>` — `Some` while a strong owner still exists, `None` once the
  last one has dropped it.
- **Reference cycle** — two or more values holding *strong* references to each
  other in a loop, so no value's strong count ever reaches zero and
  `Drop::drop` never runs for any of them — a genuine memory leak, entirely in
  safe Rust.
- **`Rc::new_cyclic`** — a constructor that hands its closure a `Weak<T>`
  pointing at the value being built, before that value exists as an `Rc`. Lets
  a parent's children each hold a working weak back-reference to it, set
  once, with no interior mutability needed.

## Text

- **String literal** — text written in the source, e.g. `"hello"`. Baked into
  the executable and typed `&'static str`, so it is a view, never an owner.
- **Unsized type** — a type whose size is not known at compile time, such as
  bare `str` or `[T]`. You can never hold one directly, only behind a
  reference or a `Box`.
- **Deref coercion** — the compiler turning a `&String` into a `&str` (or a
  `&Vec<T>` into a `&[T]`) at a call site. It is why taking `&str` in a
  parameter costs the caller nothing.
- **Unicode scalar value** — one code point, which is what a Rust `char` holds.
  Four bytes in memory, one to four bytes when written as UTF-8.
- **UTF-8** — the encoding Rust strings always use. ASCII takes one byte,
  Persian and Arabic letters two, most other scripts three, emoji four.
- **Continuation byte** — every byte of a multi-byte character after the first,
  recognisable because it starts with the bits `10`. Never a character on its
  own, which is what makes a mis-aimed slice detectable.
- **Char boundary** — a byte offset where a character actually starts. Slicing
  anywhere else panics rather than producing broken text.
- **Combining mark** — a character that modifies the one before it, like a
  Persian fatha. One thing on screen, two Unicode scalars.
- **Grapheme cluster** — what a person means by "a character": one or more
  scalars that display as a single unit. `.chars().count()` does not count
  these, and the standard library deliberately does not offer them.
- **ZWNJ (zero-width non-joiner, `\u{200C}`)** — the Persian half-space that
  keeps letters from joining, as in «می‌روم». Three bytes, one `char`, no
  width — so it silently breaks any layout that counts characters as columns.
- **Normalisation** — rewriting text into a canonical form so that two spellings
  of the same thing compare equal. Persian text needs it: the Arabic ك and the
  Persian ک look alike and are different characters.
- **`Display` / `Debug`** — the two ways of turning a value into text.
  `Display` (`{}`) is for the user; `Debug` (`{:?}`) is for you. They are
  separate traits because they are separate audiences.

## Your own types

- **Tuple struct** — a struct whose fields have positions instead of names:
  `struct Meters(f64);`. Reached with `.0`.
- **Unit struct** — a struct with no fields at all: `struct Marker;`. Zero
  bytes, and useful purely as a type.
- **Newtype pattern** — wrapping a primitive in a tuple struct so the type
  system can tell two things apart that are both, underneath, a `u64`. Free at
  run time, and it turns "I passed the arguments in the wrong order" from a
  production incident into a compile error.
- **Refutable / irrefutable pattern** — a pattern that might not match
  (`Some(x)`) versus one that always does (`(a, b)`). `let` needs an
  irrefutable one, which is why `let Some(x) = ...` alone is an error.
- **Diverging** — an expression that never produces a value because control
  never comes back: `return`, `break`, `panic!`, `todo!`. Its type is `!`, so
  it fits wherever a value is wanted.
- **Panic** — an unrecoverable failure. It unwinds the stack, running every
  destructor on the way, and is for bugs — a broken invariant — not for
  failures a caller could reasonably handle.
- **Struct literal** — the expression that builds a struct,
  `Series { title, episodes }`. **Field init shorthand** lets you write
  `title` instead of `title: title` when the variable already has the name.
- **Struct update syntax** — `Series { watched: 0, ..other }`: take these
  fields from `other`. It *moves* out of `other` unless every field is `Copy`.
- **Partial move** — moving one field out of a struct, which leaves the struct
  itself unusable while the remaining fields are still fine.
- **Associated function** — a function in an `impl` block with no `self`, called
  as `Series::new(...)`. `new` is a convention, not a keyword.
- **Invariant** — something a type promises is always true of its values, kept
  true by making fields private and only changing them through methods.
- **Enum** — a type that is exactly one of several shapes. In Rust each shape
  may carry its own data, which is what makes it a **sum type** rather than the
  named-integer enum of C or Java.
- **Variant** — one of those shapes. **Discriminant** is the hidden tag saying
  which one a given value is.
- **Niche optimisation**, and the **null-pointer optimisation** as its most
  famous case — the compiler using an impossible value as the discriminant,
  which is why `Option<Box<T>>` is the same size as `Box<T>`: null is not a
  valid `Box`, so it can mean `None` for free. `Option<bool>` gets the same
  discount and is one byte, because a `bool` has 254 spare bit patterns.
- **Pattern / arm** — a pattern is a shape the compiler matches a value
  against; an **arm** is one `pattern => expression` line of a `match`.
- **Exhaustiveness** — the compiler's proof that a `match` covers every
  possible value. It is why adding an enum variant turns every place that
  needs updating into a compile error instead of a run-time surprise.
- **Guard** — an `if` condition on a match arm. Guards do *not* count towards
  exhaustiveness, because the compiler cannot evaluate them.
- **Range pattern** (`1..=9`), **alternative** (`a | b`), **wildcard** (`_`),
  **rest** (`..`) — the pattern forms for "in this span", "either of these",
  "anything, unnamed", and "the fields I have not listed".
- **`@` binding** — `n @ 1..=9`: match the pattern *and* keep the value under a
  name.
- **Unreachable arm** — an arm no value can reach because an earlier arm already
  covers it. A warning, not an error, and almost always a bug in arm order.
- **Unwinding** — what a panic does by default: walk back up the stack running
  every destructor on the way, so files close and locks release even as the
  program fails. `panic = "abort"` in a release profile skips all of it.
- **Closure** — a function written inline and passed as a value, `|x| x + 1`.
  It can capture variables from around it, which is what separates it from a
  plain `fn`.
- **Combinator** — a method that transforms a wrapped value without unwrapping
  it: `.map()`, `.and_then()`, `.filter()`, `.unwrap_or_else()`. **Eager**
  versions (`.unwrap_or(x)`) evaluate their argument every time; **lazy** ones
  (`.unwrap_or_else(|| x)`) only when it is needed.
- **Capture** — how a closure gets hold of a variable from its surrounding
  scope: by shared reference, by mutable reference, or (with `move`) by
  taking ownership. The compiler picks whichever the closure's body
  actually needs — never something you declare yourself.
- **`Fn` / `FnMut` / `FnOnce`** — the trait hierarchy a closure's captures put
  it into: `Fn` only reads them, `FnMut` also mutates them, `FnOnce` also
  moves one out. Every `Fn` closure is also `FnMut` and `FnOnce`; every
  `FnMut` closure is also `FnOnce`. Inferred from the closure's body, never
  written by hand.
- **`move` closure** — a closure that captures everything by value (taking
  ownership) instead of by reference. Needed whenever the closure must
  outlive the scope it was written in.
- **Function pointer (`fn`)** — the type of a plain, non-capturing function
  used as a value, e.g. `fn(i32) -> i32`. It has nothing to capture, so it
  implements `Fn`, `FnMut`, and `FnOnce` all at once, for free.

## Collections

- **Amortized** — describes a cost that is expensive occasionally and cheap
  the rest of the time, averaging out to something small overall. `Vec::push`
  is amortized O(1): most calls are a cheap write, and the occasional
  reallocation-and-copy is priced across all the cheap calls that earned it.
- **Entry API** — `map.entry(key).or_insert(default)` and its relatives: look
  up a key and decide what to do about a miss, in one operation instead of a
  separate check-then-insert that would look up the key twice.
- **Hasher** — the algorithm that turns a key into the number a hash map uses
  to place it. `HashMap`'s default (SipHash) resists a deliberately crafted
  input designed to collide keys into the same bucket (**hash flooding**), at
  the cost of being slower than a hasher that does not bother to resist it.
- **Ring buffer** — a fixed block of memory treated as circular, so pushing
  past the end wraps back to the start instead of needing more memory.
  `VecDeque` is one, which is why it is cheap at both ends and `Vec` is not.
- **Priority queue** — a structure that cheaply gives you only "the current
  largest (or smallest)," and promises nothing about anything else.
  `BinaryHeap` is one.
- **Double-ended queue** — a queue you can push and pop from either end,
  cheaply. `VecDeque` is one; a plain `Vec` is not (its front is `O(n)`).

## Iterators

- **Iterator** — anything implementing one method, `next(&mut self) -> Option<Self::Item>`.
  Every adapter and consumer in this section is built on nothing but repeated
  calls to that one method.
- **Iterator adapter** — a method that wraps an iterator in a new one
  describing an extra step (`.map()`, `.filter()`, `.take()`, `.zip()`, ...).
  Lazy: it builds a description of work, it does not run it — see
  **consuming adapter** below for what actually does.
- **Consuming adapter** — an iterator method that pulls every value through
  the pipeline and produces a final, non-iterator result, such as
  `.collect()`, `.sum()`, or `.count()`. Unlike a lazy iterator adapter
  (`.map()`, `.filter()`), calling one is what actually runs the pipeline.
- **`FromIterator`** — the trait `.collect()` is generic over. A type that
  implements it can be built from any iterator of the right item type —
  `Vec<T>`, `String`, `HashMap<K, V>`, `HashSet<T>`, and `Result<Vec<T>, E>`
  all do, which is why `.collect()` needs a turbofish or a type annotation to
  know which one you mean.
- **Short-circuiting** — stopping a computation as soon as its final answer
  is already known, instead of finishing every remaining step. Collecting an
  iterator of `Result` into `Result<Vec<T>, E>` short-circuits: the first
  `Err` becomes the whole result, and nothing after it is touched.
- **Lazy iterator (demand-driven evaluation)** — an adapter chain does
  nothing by itself; each element travels through the *entire* chain, one at
  a time, only when something downstream calls `.next()` for it. A
  different claim from the eager/lazy pair above, which is about one
  argument's evaluation, not a whole pipeline.
- **Zero-cost abstraction** — a high-level construct (an iterator chain, for
  instance) that compiles down to the same work as its hand-written
  equivalent — writing it declaratively costs nothing extra at run time.
- **Infinite iterator** — an iterator with no defined end (`std::iter::repeat`,
  `.cycle()`, an unbounded range). Safe to build only because nothing runs
  until a bounded consumer like `.take()` asks for values.
- **Generator** — a description of how to build the next value from the
  current one, not a list written out in advance. `std::iter::successors` is
  one: it starts at a first value and calls a function on the last one to
  build each next one, until that function returns `None`.
- **`.by_ref()`** — a temporary borrow of an iterator, so it is still usable
  afterward. Lets you `.take(n)` a few items now without losing the rest.
- **Mid-chain `.collect()` trap** — collecting partway through an adapter
  chain you meant to keep going, forcing an allocation and a full pass that
  laziness would otherwise have avoided.

## Traits and generics

- **Trait bound** — a constraint on a generic parameter, `T: SomeTrait`, that
  tells the compiler (and the reader) exactly what a placeholder type can do.
  An unbounded `T` supports nothing beyond taking, holding, and returning it
  — no comparing, printing, or cloning until a bound promises it. Written
  inline (`<T: Trait>`) or, once several pile up, after the signature with
  `where`.
- **Monomorphization** — the compiler generating one entirely separate,
  concrete copy of a generic function or type per distinct concrete type it
  is actually called with, at compile time. By the time a program runs, no
  unresolved type parameter is left anywhere — the mechanical reason generics
  cost nothing at run time.
- **Static dispatch** — deciding which function a call targets entirely at
  compile time. A generic function gets monomorphized into one compiled copy
  per concrete type actually used, and `impl Trait` (either position) is
  static dispatch too — there's still exactly one concrete type behind it.
- **Dynamic dispatch** — deciding which function a call targets at run time,
  by reading a vtable. One compiled function serves every concrete type
  behind a `dyn Trait`, at the cost of one extra pointer hop per call.
- **Trait object (`dyn Trait`)** — a value of some type implementing a
  trait, with the concrete type erased. Always lives behind a pointer
  (`&dyn Trait`, `Box<dyn Trait>`, `Rc<dyn Trait>`) because the erased type
  has no size of its own.
- **vtable** (virtual method table) — the small table of function pointers
  stored alongside a trait object's data, one entry per trait method, used
  to find the right implementation at each call. The second word of the fat
  pointer a trait object actually is.
- **Object safety** (also called **dyn compatibility** — the newer, more
  official name in the compiler's own error text) — the condition a trait
  must meet to become `dyn Trait`. A method returning `Self` by value or a
  method with its own generic type parameter are the two most common ways
  to break it, because neither can get a fixed-size vtable slot.
- **`impl Trait`** — sugar with two different meanings depending on
  position. In an argument, it's exactly a generic bound, spelled without
  naming the type parameter. In a return type, it hides which concrete type
  is being returned while staying static dispatch — and it can only ever
  name one concrete type, never "this one or that one depending on a
  branch."
- **`Default`** — a trait for "give me a sensible starting value":
  `Default::default()`. `#[derive(Default)]` needs every field's type to
  implement it too; struct-update syntax (`..Default::default()`) leans on it
  to fill in whatever you don't set by hand.
- **Reflexivity** — the property that every value equals itself: `x == x` is
  always `true`. `PartialEq` does not require it; `Eq` is the marker trait
  that promises it on top. `f64` cannot honestly implement `Eq`, because
  `NaN == NaN` is `false`.
- **Total order** — a comparison where, for any two values, "which is
  smaller?" always has a definite answer. `Ord` promises one; `PartialOrd`
  does not — its `partial_cmp` can return `None`, exactly what happens
  whenever `NaN` is involved.
- **`Hash` (trait)** — turns a value into the number a `HashMap`/`HashSet`
  uses to place it, by feeding its fields into a `Hasher` in turn. Required
  on every key type; `#[derive(Hash)]` writes the field-by-field version for
  you.
- **`Hash`/`Eq` consistency** — the rule that two values equal by `Eq` must
  also hash equal. Nothing enforces it at compile time; break it and a
  `HashMap`/`HashSet` starts silently losing "duplicates" it should have
  recognized.

## Lifetimes and conversion

- **`Deref` / `DerefMut`** — the traits behind `*` and auto-deref.
  `Deref::deref(&self) -> &Self::Target` is what the compiler calls to follow
  a wrapper down to what it wraps, at a method call or via `*`; `DerefMut:
  Deref` (a supertrait) adds `deref_mut` for the `&mut` version. `Box`,
  `String`, and `Vec` all implement them for their own `Target`, and a chain
  of `impl Deref`s coerces in one hop per link — `&Watchlist -> &Vec<String>
  -> &[String]` is two.
- **`AsRef<T>`** — a trait for "can be cheaply viewed as `&T`":
  `fn as_ref(&self) -> &T`. Lets a function take `impl AsRef<str>` (or
  `AsRef<Path>`) and work identically whether the caller hands over a `&str`,
  a `String`, or a `PathBuf` — at zero extra cost to the caller.
- **`Borrow<T>`** — the same shape as `AsRef<T>`, but a stronger promise:
  `Hash`, `Eq`, and `Ord` must agree between the borrowed form and the owned
  one. This is what makes `HashMap<String, V>::get(&str)` legal — and what
  makes a `Borrow` impl that disagrees with its own `Hash`/`Eq` build a key
  that provably exists but cannot be found.
- **`ToOwned`** — the generalization of `Clone` for unsized borrowed types:
  `fn to_owned(&self) -> Self::Owned`, where `Owned` does not have to be
  `Self`. `str::to_owned() -> String` is the standard example, since `str`
  itself cannot be `Clone` (an unsized return type does not compile). Every
  `T: Clone` gets `ToOwned` for free via a blanket impl with `Owned = T`.
- **`Cow<'a, B>`** ("clone on write") — an enum with exactly two states,
  `Borrowed(&'a B)` and `Owned(B::Owned)`. Lets a function return a borrowed,
  zero-copy view in the common case and only allocate an owned value in the
  case that actually needs one — the caller cannot tell which it got except
  by cost. Derefs to `&B` (via `Deref`), so it is usable as a borrow without
  matching on it.
- **`.to_mut()`** — the method that actually performs the clone in
  copy-on-write: called on a `Cow`, it clones (via `ToOwned`) only if the
  `Cow` is not already `Owned`, then hands back a `&mut` to the owned form.
  The clone happens the moment `.to_mut()` is *called*, not when the `&mut`
  is actually written through.

## Error handling

- **`thiserror`** — a derive macro crate: `#[derive(thiserror::Error)]` plus
  one `#[error("...")]` attribute per variant generates the `Display` and
  `std::error::Error` impls you would otherwise hand-write. `#[source]` wires
  up `Error::source()`; `#[from]` additionally generates a `From` impl — but
  only for a field that is the variant's entire payload, since `From::from`
  never receives anything else to build the rest of the variant from.
- **`anyhow`** — a crate providing `anyhow::Error`, a single dynamic error
  type that can hold any value implementing `std::error::Error`. For code
  that only needs to propagate, log, or display a failure, never match on
  which kind it was.
- **`anyhow::Context`** — a trait adding `.context(msg)` /
  `.with_context(|| msg)` to any `Result`, attaching a human-readable
  message to a propagating error without discarding the original — the
  message becomes the top of the error's chain, the original error still
  reachable through `.source()`.
- **Library/binary boundary** — the rule that a library exposes a specific,
  matchable error type, because it cannot know whether its caller needs to
  branch on the failure; a binary — the outermost layer, with no caller of
  its own — may collapse everything into one dynamic error type instead,
  because nothing downstream of it will ever match on it.

## Error handling

- **`source()` / error chain** — the `std::error::Error` method
  `fn source(&self) -> Option<&(dyn Error + 'static)>`, which lets an error
  variant point at the lower-level error that actually caused it. Calling
  `.source()` repeatedly — on the result of the last call, starting from a
  top-level error — until it returns `None` walks an **error chain** from
  the failure down to its root cause.
- **`Box<dyn Error>`** — a type-erased "any error" container: one return
  type that covers every concrete error type implementing
  `std::error::Error`, at the cost of no longer being able to `match` on
  which one it actually is. The standard library's blanket
  `impl<E: Error> From<E> for Box<dyn Error>` is what lets `?` convert any
  such error into it with no hand-written `From` impl.
- **Downcasting** — recovering a concrete type from a `dyn Error + 'static`
  (or any `dyn Trait + 'static`) via `.downcast_ref::<T>()`, when the
  caller already suspects which `T` it might be. Not exhaustive like a
  `match` — a wrong guess just returns `None`, and nothing forces every
  case to be covered.
- **Error taxonomy** — grouping the many concrete ways a piece of code can
  fail into a small number of caller-relevant *categories* (validation,
  not-found, and internal/unexpected are a common three), instead of one
  variant per failure or one flat catch-all. The categories are what a
  caller actually needs to react differently to; how many distinct causes
  live inside one category is an implementation detail, not a reason for
  another top-level variant.

## Smart pointers

- **Interior mutability** — mutating a value through a shared (`&T`)
  reference, something the aliasing rule normally forbids outright at
  compile time. `Cell`/`RefCell` don't bend that rule; they move where it
  gets enforced — to run time for `RefCell`, or sidestep the question
  entirely for `Cell`, which never hands out a reference to the value in
  the first place.
- **`Cell<T>`** — the simplest interior-mutability wrapper: `.get()`,
  `.set()`, and `.replace()` copy or swap the whole value through `&self`,
  with no reference to the inside ever handed out — so there is nothing to
  track and nothing that can panic. `.get()` additionally requires
  `T: Copy`; `.set()`/`.replace()`/`.take()` do not.
- **`RefCell<T>`** — moves the aliasing rule's enforcement from compile
  time to run time. `.borrow()`/`.borrow_mut()` hand back guard types that
  track how many of each are currently alive, and panic — rather than
  refuse to compile — the moment a second, incompatible borrow is attempted
  while one is still live.
- **`Ref<T>` / `RefMut<T>`** — the guard types `RefCell::borrow()` and
  `.borrow_mut()` return. Both implement `Deref<Target = T>`; only `RefMut`
  also implements `DerefMut`, which is why writing through a `Ref` is a
  compile error, not a run-time panic.

## Testing and benchmarking

- **Unit test** — a `#[test]` function compiled as part of the crate it
  tests, typically inside a `#[cfg(test)] mod tests` block at the bottom of
  the same file. Because it compiles as part of that crate, it reaches
  private items no outside caller could even name.
- **Integration test** — a file under `tests/`, compiled as its own separate
  crate that depends on the library the same way an external user would. It
  can only reach `pub` items — `pub(crate)` is exactly as invisible to it as
  plain private.
- **Doc test** — a fenced code block inside a `///` (or `//!`) doc comment,
  compiled and run as a real test by `cargo test`. A line starting with `# `
  is compiled and run but hidden from the rendered documentation; a
  `should_panic`-tagged fence asserts the code panics, without checking the
  panic message the way `#[should_panic(expected = "...")]` can.
- **Test double** — a stand-in for a real dependency in a test, substituted
  through the same trait boundary the real dependency implements. Stub,
  fake, spy, and mock are the four common shapes; a given double often
  plays more than one role at once.
- **Stub** — a test double that returns a fixed, canned answer, useful for
  exercising error paths that are hard to trigger with the real
  dependency.
- **Fake** — a test double with real, working behavior, just simplified —
  e.g. an in-memory store standing in for a real database.
- **Spy** — a test double that records what was called, with what
  arguments, so a test can assert on it after the fact.
- **Mock** — a test double pre-loaded with expectations that verifies them
  itself, rather than leaving the assertion to the test body. Heavier than
  a spy; reached for when call order or call count needs checking.
- **Injection** — handing a dependency to the code that needs it from the
  outside (a constructor argument, typically), rather than that code
  constructing or naming the concrete dependency itself. In Rust this
  happens at the exact call site of `::new(...)`, through a generic bound
  or a `dyn Trait` — no separate DI framework or config file involved.
- **Property (property-based testing)** — a rule that must hold for *every*
  input in a domain, not just a handful of picked examples — e.g. "decoding
  what you encoded always gives back the original." Checked by generating
  many inputs and trying to break the rule, instead of hand-picking a few.
- **`proptest`** — a crate for property-based testing:
  `proptest! { #[test] fn name(x in strategy) { ... } }` turns a
  parameterized function into a test that runs against many generated inputs
  (256 by default) instead of one hand-picked one.
- **Strategy (proptest)** — a description of where proptest should draw
  values from, e.g. `any::<i32>()` (any possible value of the type) or
  `prop::collection::vec(any::<bool>(), 0..16)` (a `Vec<bool>` of bounded
  length). What a test parameter is bound to after `in`, inside `proptest!`.
- **Shrinking** — once proptest finds a failing input, it repeatedly
  simplifies it (toward zero, toward an empty collection) while it still
  fails, until nothing smaller reproduces the bug. A failure report always
  shows this smallest case, never the first random one that happened to fail.
- **Snapshot testing** — capturing a complex output once, reviewing it by
  hand as the source of truth, then having every later test run diff the
  current output against that saved copy — failing loudly on any unreviewed
  change, whether it turns out to be a bug or an intended update.
- **`insta`** — a snapshot-testing crate: `insta::assert_snapshot!(value)`
  compares `value`'s text against a committed `.snap` file, or writes a
  pending `.snap.new` file when there's nothing to compare against yet, or
  the output no longer matches.
- **`.snap` / `.snap.new`** — an approved snapshot (committed, the source of
  truth) versus a pending one insta just wrote because there was nothing to
  compare against, or the output changed. `cargo insta accept`/
  `cargo insta reject` (or renaming the file by hand) resolves a pending one.
- **`criterion`** — a statistical benchmarking harness: runs a function many
  times, discards the warmup samples, and reports a mean with a confidence
  interval plus flagged statistical outliers, instead of handing back one
  number to trust blindly. Registered as a `[[bench]]` target with
  `harness = false`, and wired up with `criterion_group!`/`criterion_main!`.
- **`black_box`** (`std::hint::black_box`) — a function that hides a value
  from the optimizer, so it can't be constant-folded away or proven unused.
  Not specific to benchmarking — its first use in this course stopped a
  bounds-check demo from being rejected at compile time — but its most
  common use is forcing a benchmarked computation to actually run every
  iteration instead of being precomputed once.

## Modules and project structure

- **Module** — Rust's unit of code organization inside a crate, declared
  with `mod`. Builds a *tree*: `mod foo { ... }` writes a module inline,
  right where it's declared; `mod foo;` (a declaration with no body) tells
  Rust to find that module's contents in a separate file instead —
  unlike Python, where every `.py` file is automatically a module with no
  declaration needed at all.
- **Visibility** — whether an item (a struct, a field, a function, a
  module) can be *named* from a given point in the code. Private by
  default: visible only inside the module that defines it, plus that
  module's descendants. `pub`, `pub(crate)`, and `pub(super)` each widen
  that reach by a different amount.
- **`pub(crate)`** — visible from anywhere inside the current crate, but
  not to an external crate depending on this one as a library. The common
  way to mark something "an implementation detail of my own codebase, not
  part of my public API."
- **`pub(super)`** — visible to the immediate parent module, and anywhere
  that parent module is itself visible from — one level up, no further.
- **Re-export (`pub use`)** — presenting an item at a shallower public path
  than the one it is actually defined at, without moving it. Lets a crate
  keep a flat, stable public API while its internal module tree is
  reorganized freely underneath.
