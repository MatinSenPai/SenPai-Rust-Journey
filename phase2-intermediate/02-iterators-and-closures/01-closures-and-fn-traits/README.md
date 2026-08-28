# 2.2.1 — Closures, `Fn`/`FnMut`/`FnOnce`, and `move`

## At a glance

After this lesson you can:

- Read a closure's body and say what it captures and how (shared reference, mutable reference, or full ownership with `move`) — without running the compiler to find out.
- Say which of `Fn`, `FnMut`, and `FnOnce` a given closure implements and why, and explain why those three form a hierarchy rather than three unrelated choices.
- Say exactly when `move` is required, fix `E0596`, `E0597`, and `E0382` yourself, and choose between an `Fn`, `FnMut`, or `FnOnce` parameter for a real function signature.

**Time:** ~60 minutes · **Prerequisites:**
[2.1.4 — Choosing a collection](../../01-collections/04-choosing-a-collection/README.md), and specifically
[1.2.2 — Move semantics](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.md) and
[1.3.1 — Shared and mutable references](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md)

---

## Why this matters

All through module 2.1 this word kept showing up, unexplained. In
[2.1.1](../../01-collections/01-vec-depth/README.md) we wrote
`list.retain(|entry| !entry.watched)` and immediately said: "that
`|entry| ...` is a **closure** — a tiny, unnamed function you wrote right on
the spot. Module 2.2.1 explains it properly; for now, just read it." The same
lesson also wrote `.sort_by(|a, b| a.rating.total_cmp(&b.rating))` and
`.dedup_by_key(|entry| entry.0.clone())` — three closures, three different
uses, all left unexplained.

This lesson pays that debt off. If you've written Python before, the idea of
a closure itself is not foreign:

```python
threshold = 10
is_long = lambda title: len(title) > threshold
```

`is_long` never needed `threshold` passed in as a parameter — it just reached
out and grabbed it from the surrounding scope. Rust allows exactly the same
thing. The difference starts exactly where Python's story ends: in Python,
everything is a reference to a reference-counted object, so a Python closure
is never forced to choose between "just look at it" and "take ownership of
it" — there is nothing left to choose. In Rust, the borrow checker applies
the exact same rule you learned in
[1.2](../../../phase1-fundamentals/02-ownership-and-memory/README.md) and
[1.3](../../../phase1-fundamentals/03-borrowing-and-references/README.md) —
one owner, or any number of shared borrows, or exactly one mutable borrow —
to every closure you write, too. That is where this lesson starts: a
closure, from Rust's point of view, is an ownership problem, not a
convenience of syntax.

---

## The concept

### A closure is a real value with a real, unnameable type

Closure syntax always has the same shape: a parameter list between two
vertical bars, then a body — a single expression, or a `{ ... }` block if you
need more than one line:

```rust
let add_one = |x: i32| x + 1;
let add_two = |x: i32| x + 2;
println!("add_one(5) = {}", add_one(5));
println!("add_two(5) = {}", add_two(5));
```

```text
add_one(5) = 6
add_two(5) = 7
```

`add_one` is an ordinary variable, exactly like any other `let` — this time
it just happens to hold something callable. Its type? Not `i32`, not
`String`, not anything you could type into source code. The compiler builds
a brand-new, anonymous struct for every closure you write; `std::any::type_name`
proves it:

```rust
fn type_name_of<V>(_value: &V) -> String {
    std::any::type_name::<V>().to_string()
}

println!("{}", type_name_of(&add_one));
```

```text
01_closure_is_a_value::main::{{closure}}
```

`{{closure}}` is the compiler's own notation, not something you could type —
it isn't even valid Rust syntax. Even two closures that look completely
identical, like `add_one` and `add_two` above, print names that *look* the
same here but are two entirely different types — try putting one where the
other's variable already lives, and see "Errors you will meet" for exactly
what that produces. That one fact — every closure, its own unique type — is
the real reason behind the "Closures as parameters and return values"
section further down this page.

### The default capture: by reference, whichever shape the body needs

When a closure uses a variable from around it, the default is to capture it
by reference — the same `&T` or `&mut T` from
[1.3.1](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.md),
except this time the compiler itself decides which one, based on what the
closure's body actually does with that variable:

```rust
let name = String::from("Rin");
let greet = || println!("hello, {name}");
greet();
println!("still have `name` here: {name}");
```

```text
hello, Rin
still have `name` here: Rin
```

`greet` only reads `name`, so it captured it by shared reference — and
`name` is still fully yours afterward, exactly like any other shared borrow.

Now a closure that mutates:

```rust
let mut hits = 0;
let mut record_hit = || {
    hits += 1;
};
record_hit();
record_hit();
record_hit();
println!("hits: {hits}");
```

```text
hits: 3
```

This time the body mutates `hits`, so the closure captured it by mutable
reference. And because the *binding* `record_hit` itself now holds an
exclusive borrow, the binding has to be `mut` too — the same aliasing rule
from 1.3.1, this time applied to a variable that holds a closure. Forget it,
and the compiler stops you; the full error is in "Errors you will meet".

### Capturing with `move`: taking full ownership

Sometimes a reference is not enough. The closure has to outlive the scope it
was written in — returned from a function, handed off to run on another
thread, or simply stored somewhere that outlives the natural lifetime of that
borrow. The `move` keyword exists for exactly these cases: it forces the
closure to take full ownership of everything it captures, instead of
borrowing it.

```rust
let printer;
{
    let message = String::from("hi from the inner scope");
    printer = move || println!("{message}");
}
printer();
```

```text
hi from the inner scope
```

`message` is local to that inner block — it is dropped when the block ends.
If `printer` had only borrowed a reference to `message`, that reference
would be dangling by the time `printer()` runs outside the block. `move`
removes the problem at the root: `message` no longer belongs to the inner
block, it belongs to the closure itself, and the closure carries it wherever
it goes. Without `move`, this exact code produces `E0597` — the broken
version is in "Errors you will meet".

```senpai-visual
{"kind":"ownership","labels":["closure borrows `message`","inner scope ends","`message` is dropped","the borrow is now dangling","`move` takes ownership instead"]}
```

One thing not to conflate: `move` only decides *how* a variable is
captured, not how many times the closure can be called. Those are two
entirely separate questions:

```rust
let name = String::from("Rin");
let greet = move || println!("hello, {name}");

greet();
greet();
greet();
```

```text
hello, Rin
hello, Rin
hello, Rin
```

`greet` took full ownership of `name` — but its body only ever reads it, it
never moves it out. So it is still callable as many times as you like.
Capturing `move` means "take it fully," not "you get one use." What decides
how many times a closure can be called is something else entirely — the
subject of the next section.

### `Fn`, `FnMut`, `FnOnce`: a hierarchy, not three unrelated choices

Depending on what its body does with what it captured, every closure
implements one (or more) of these three traits:

```rust
let greeting = String::from("hi");
let read_only = || println!("read-only: {greeting}");
read_only();
read_only();

let mut hits = 0;
let mut count_calls = || {
    hits += 1;
    println!("mutate: called {hits} time(s)");
};
count_calls();
count_calls();

let payload = String::from("payload");
let consume = move || payload;
let taken = consume();
println!("consume: took ownership of {taken}");
```

```text
read-only: hi
read-only: hi
mutate: called 1 time(s)
mutate: called 2 time(s)
consume: took ownership of payload
```

- **`Fn`** — the body only reads what it captured. Callable any number of
  times.
- **`FnMut`** — the body mutates what it captured. Still callable any number
  of times, but each call may change whatever was captured.
- **`FnOnce`** — the body moves what it captured out of the closure itself.
  Callable **once**, because after the first call there is nothing left to
  capture.

These three form a hierarchy: every `Fn` closure is also a valid `FnMut`
("only reading" is a special case of "doesn't need exclusive access"), and
every `FnMut` closure is also a valid `FnOnce` ("mutating repeatedly" is a
special case of "callable at least once"). The direction only runs one way:
every `Fn` is also an `FnOnce`, but not every `FnOnce` is a `Fn` — exactly
what the `consume` example above just showed.

```senpai-visual
{"kind":"concept","labels":["captures nothing: `fn`","only reads: `Fn`","mutates: `FnMut`","consumes: `FnOnce`"]}
```

And the important part: you never write which trait a closure implements
yourself. The compiler reads the body and infers the tightest one that
applies — exactly how `read_only`, `count_calls`, and `consume` above ended
up with three different fates, without anyone telling the compiler which was
which.

### Closures as parameters and return values

From the first section: every closure has a unique, unnameable type. That
means a function that wants to accept "a closure" can never write that type
in its signature — it has to be generic over whatever trait it actually
needs:

```rust
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

fn apply_twice_v2(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(f(x))
}

println!("{}", apply_twice(|x| x + 10, 1));
println!("{}", apply_twice_v2(|x| x + 10, 1));
```

```text
21
21
```

`apply_twice` and `apply_twice_v2` have exactly the same signature, spelled
two ways: a generic parameter bounded by `Fn(i32) -> i32`, or `impl Trait` in
argument position — pure shorthand for the same thing. Generics and
`impl Trait` get their full lesson in module 2.3; today, this much is
enough: because `F` gets replaced with the real closure's type at each call
site, the compiler builds a separate copy of the function for every distinct
closure you pass — nobody ever needs to know the name of that `{{closure}}`
type.

The return path has the same problem, mirrored: a function that wants to
return "a closure" still can't write its type. Generics don't help here (the
return type isn't chosen by the caller, the function itself has to name it),
so `impl Trait` in return position is what shows up instead:

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

let add_five = make_adder(5);
println!("{}", add_five(1));
println!("{}", add_five(100));
```

```text
6
105
```

`impl Fn(i32) -> i32` means "some concrete type that implements this, I'm
not telling you which one." `move` is required here too: without it, the
closure would try to borrow `n`, a parameter that is gone the moment
`make_adder` returns. (One note for later: `impl Trait` in return position
only promises a *single* concrete type; when the real type varies depending
on a branch, you need `Box<dyn Fn(i32) -> i32>` instead — the subject of the
trait-objects lesson.)

`FnMut` and `FnOnce` show up in signatures for exactly the same underlying
reason — not taste, but how many times the function plans to call the
closure:

```rust
fn call_n_times<F: FnMut()>(mut f: F, n: u32) {
    for _ in 0..n {
        f();
    }
}

fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}

let mut log = Vec::new();
call_n_times(|| log.push("tick"), 3);
println!("{log:?}");

let owned = String::from("payload");
println!("{}", run_once(move || owned));
```

```text
["tick", "tick", "tick"]
payload
```

`call_n_times` may call `f` many times — and a realistic caller usually wants
to change something on each call (a counter, a log) — so it needs `FnMut`,
not the stricter `Fn`. `run_once` is the opposite: it only ever calls once,
so the weakest trait is enough for it — `FnOnce` — and that weakness is
exactly what lets a closure that consumes what it captured through the door.

The hierarchy's direction is one-way, and the compiler enforces it: a
closure that is only `FnOnce` gets rejected wherever `Fn` was asked for —
the reverse (handing an `Fn` closure to something that only needed `FnOnce`)
always works, because `Fn` is already a valid `FnOnce`. Try `run_once` above
with a plain `Fn` closure — no problem; but call `apply_twice` with an
`FnOnce`-only closure, and you get `E0525` — the full error is in the next
section.

### Function pointers: the trivial, capture-free case

An ordinary function, used as a value, captures nothing from any scope — so
it gets `Fn`, `FnMut`, and `FnOnce` all at once, for free, with no
conditions attached:

```rust
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

fn add_ten(x: i32) -> i32 {
    x + 10
}

println!("{}", apply_twice(add_ten, 1));
let as_pointer: fn(i32) -> i32 = add_ten;
println!("{}", apply_twice(as_pointer, 1));
```

```text
21
21
```

`add_ten`'s type, when used as a value, is `fn(i32) -> i32` — a **function
pointer**: lowercase `fn`, a concrete type, not the trait `Fn`. Since it has
nothing to capture, it slots in anywhere an `Fn` closure is expected, with no
manual conversion — the simplest case of everything you saw in this lesson.

---

## Hands on

```sh
cargo run -p p2-02-01-closures-and-fn-traits --example 01-closure-is-a-value
cargo run -p p2-02-01-closures-and-fn-traits --example 02-capture-by-reference
cargo run -p p2-02-01-closures-and-fn-traits --example 03-move-lets-a-closure-outlive-its-scope
cargo run -p p2-02-01-closures-and-fn-traits --example 04-move-does-not-mean-fnonce
cargo run -p p2-02-01-closures-and-fn-traits --example 05-fn-fnmut-fnonce-hierarchy
cargo run -p p2-02-01-closures-and-fn-traits --example 06-closures-as-parameters-and-return-values
cargo run -p p2-02-01-closures-and-fn-traits --example 07-fnmut-and-fnonce-parameters
cargo run -p p2-02-01-closures-and-fn-traits --example 08-function-pointers
```

Then the five broken ones:

```sh
cargo run -p p2-02-01-closures-and-fn-traits --example 09-closures-have-distinct-types --features broken
cargo run -p p2-02-01-closures-and-fn-traits --example 10-forgot-mut-on-capturing-closure --features broken
cargo run -p p2-02-01-closures-and-fn-traits --example 11-missing-move-dangling-borrow --features broken
cargo run -p p2-02-01-closures-and-fn-traits --example 12-fnonce-closure-called-twice --features broken
cargo run -p p2-02-01-closures-and-fn-traits --example 13-fn-bound-rejects-fnonce-only --features broken
```

Then try these:

1. In `02-capture-by-reference`, call `record_hit` ten times instead of
   three. Then remove `mut` from in front of `record_hit` and read exactly
   what the compiler suggests.
2. In `05-fn-fnmut-fnonce-hierarchy`, add one more `consume();` line right
   after `let taken = consume();`. Which error do you get, and which line
   inside the error message says *why*?
3. In `06-closures-as-parameters-and-return-values`, build
   `make_adder(-3)` instead of `make_adder(5)` and call it on a few
   different numbers.

---

## Errors you will meet

### `E0308` — two closures, even with the same signature, are two different types

```text
error[E0308]: mismatched types
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\09-closures-have-distinct-types.rs:15:13
   |
12 |     let add_one = |x: i32| x + 1;
   |                   -------- the expected closure
13 |     let add_two = |x: i32| x + 2;
   |                   -------- the found closure
14 |     let mut which = add_one;
15 |     which = add_two;
   |             ^^^^^^^ expected closure, found a different closure
   |
   = note: expected closure `{closure@phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\09-closures-have-distinct-types.rs:12:19: 12:27}`
              found closure `{closure@phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\09-closures-have-distinct-types.rs:13:19: 13:27}`
   = note: no two closures, even if identical, have the same type
   = help: consider boxing your closure and/or using it as a trait object

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is actually objecting to:** `add_one` and `add_two` both
have signature `Fn(i32) -> i32`, but the two *distinct* closures don't share
a type. `which` took its type from `add_one` at its first assignment;
putting `add_two` into it is like trying to put an `i32` where a `bool`
belongs.

**The fix:** keep two different closures in two separate variables:

```rust
let add_one = |x: i32| x + 1;
let add_two = |x: i32| x + 2;
println!("{}", add_one(5));
println!("{}", add_two(5));
```

**Why this is the fix:** nothing here ever actually needed these two to
share one variable. If you genuinely need one variable that might hold
either closure depending on a condition, you have to erase the concrete
type — exactly the `Box<dyn Fn(i32) -> i32>` the concept section pointed
at. And this exact type difference is what justifies "Closures as
parameters and return values": a function that wants to accept either of
these closures has to be generic.

### `E0596` — a closure that mutates has to sit in a `mut` binding

```text
error[E0596]: cannot borrow `increment` as mutable, as it is not declared as mutable
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\10-forgot-mut-on-capturing-closure.rs:15:5
   |
13 |         count += 1;
   |         ----- calling `increment` requires mutable binding due to mutable borrow of `count`
14 |     };
15 |     increment();
   |     ^^^^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
12 |     let mut increment = || {
   |         +++

For more information about this error, try `rustc --explain E0596`.
```

**What the compiler is actually objecting to:** `increment`'s body wants to
mutate `count`, so the closure itself holds a mutable reference. Calling
`increment()` means using that mutable reference — and, like any other use
of a mutable borrow, that requires a `mut` binding.

**The fix:** put `mut` in front of the binding:

```rust
let mut count = 0;
let mut increment = || {
    count += 1;
};
increment();
println!("{count}");
```

```text
1
```

**Why this is the fix:** this is exactly the rule from 1.3.1, applied to a
variable holding a closure instead of a plain `&mut`: whatever holds an
exclusive borrow has to be `mut` itself.

### `E0597` — without `move`, a closure cannot outlive the scope it was written in

```text
error[E0597]: `message` does not live long enough
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\11-missing-move-dangling-borrow.rs:14:33
   |
13 |         let message = String::from("hi from the inner scope");
   |             ------- binding `message` declared here
14 |         printer = || println!("{message}");
   |                   --            ^^^^^^^ borrowed value does not live long enough
   |                   |
   |                   value captured here
15 |     }
   |     - `message` dropped here while still borrowed
16 |     printer();
   |     ------- borrow later used here

For more information about this error, try `rustc --explain E0597`.
```

**What the compiler is actually objecting to:** without `move`, `printer`
only borrowed `message`. `message` was dropped at the end of that inner
block — but `printer`, and the borrow it holds, is still alive outside that
block. This is exactly what
[1.2.2](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.md)
taught you: a borrow must never outlive the thing it points to.

**The fix:** add `move`:

```rust
let printer;
{
    let message = String::from("hi from the inner scope");
    printer = move || println!("{message}");
}
printer();
```

```text
hi from the inner scope
```

**Why this is the fix:** with `move`, the closure no longer borrows —
it takes full ownership of `message`. It no longer matters when that inner
block ends; `message` doesn't belong to the block anymore, it belongs to
`printer`.

### `E0382` — calling an `FnOnce` closure consumes the closure itself

```text
error[E0382]: use of moved value: `consume`
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\12-fnonce-closure-called-twice.rs:17:5
   |
16 |     consume();
   |     --------- `consume` moved due to this call
17 |     consume();
   |     ^^^^^^^ value used here after move
   |
note: closure cannot be invoked more than once because it moves the variable `name` out of its environment
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\12-fnonce-closure-called-twice.rs:13:21
   |
13 |         let owned = name;
   |                     ^^^^
note: this value implements `FnOnce`, which causes it to be moved when called
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\12-fnonce-closure-called-twice.rs:16:5
   |
16 |     consume();
   |     ^^^^^^^

For more information about this error, try `rustc --explain E0382`.
```

**What the compiler is actually objecting to:** `consume`'s body moves
`name` out of its own captures with `let owned = name;` — so `consume` only
implements `FnOnce`. Calling an `FnOnce` closure moves the closure itself;
the first call consumed `consume`, and the second call has nothing left to
call.

**The fix:** if you genuinely need multiple calls, don't move what you
captured out — clone it instead:

```rust
let name = String::from("Rin");
let consume = move || {
    let owned = name.clone();
    println!("consumed: {owned}");
};
consume();
consume();
```

```text
consumed: Rin
consumed: Rin
```

**Why this is the fix:** `.clone()` only builds a fresh copy from `name` and
leaves `name` itself untouched inside the closure's captures — the body no
longer moves anything out of itself, so the closure is no longer `FnOnce`,
it's `Fn`, and it stays callable as many times as you like.

### `E0525` — an `Fn` parameter rejects a closure that is only `FnOnce`

```text
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\13-fn-bound-rejects-fnonce-only.rs:18:27
   |
18 |     let consume_and_add = move |x: i32| {
   |                           ^^^^^^^^^^^^^ this closure implements `FnOnce`, not `Fn`
19 |         drop(bonus);
   |              ----- closure is `FnOnce` because it moves the variable `bonus` out of its environment
...
22 |     println!("{}", apply_twice(consume_and_add, 5));
   |                    ----------- --------------- the requirement to implement `Fn` derives from here
   |                    |
   |                    required by a bound introduced by this call
   |
note: required by a bound in `apply_twice`
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\13-fn-bound-rejects-fnonce-only.rs:12:19
   |
12 | fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
   |                   ^^^^^^^^^^^^^^ required by this bound in `apply_twice`

For more information about this error, try `rustc --explain E0525`.
```

**What the compiler is actually objecting to:** `apply_twice` is going to
call `f` twice, so its bound is `Fn`. `consume_and_add` moves `bonus` out of
its own captures with `drop(bonus)` — so it only implements `FnOnce`. The
hierarchy only runs one direction: every `Fn` is also an `FnOnce`, but this
closure is the other way around, and the compiler will not accept that.

**The fix:** read `bonus` instead of moving it out:

```rust
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

let bonus = String::from("bonus");
let add_with_log = move |x: i32| {
    println!("adding for {bonus}");
    x + 1
};
println!("{}", apply_twice(add_with_log, 5));
```

```text
adding for bonus
adding for bonus
7
```

**Why this is the fix:** `println!("{bonus}")` only reads `bonus` — nothing
moves out of the closure's captures. `add_with_log` is now `Fn`, and
`apply_twice` can call it twice without a problem.

---

## Exercises

### Warm up

<details>
<summary>Does this compile?</summary>

```rust
let text = String::from("hi");
let show = || println!("{text}");
show();
println!("{text}");
```

</details>

<details>
<summary>Answer</summary>

Yes. `show` only reads `text`, so it captured it by shared reference —
`text` stays fully usable afterward.

</details>

<details>
<summary>What does this print?</summary>

```rust
let mut total = 0;
let mut add = |n: i32| total += n;
add(3);
add(4);
println!("{total}");
```

</details>

<details>
<summary>Answer</summary>

```text
7
```

Each call mutates `total`; `add` is an `FnMut`, and consecutive calls
accumulate: 0 + 3 + 4 = 7.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let data = vec![1, 2, 3];
let consume = move || data;
let first = consume();
let second = consume();
```

</details>

<details>
<summary>Answer</summary>

No. `consume`'s body moves `data` out of its own captures and returns it —
so `consume` is only `FnOnce`. The first call moves `consume` itself; the
second call gives an `E0382`, exactly like the `consume`/`name` example in
"Errors you will meet".

</details>

<details>
<summary>Which of <code>Fn</code>, <code>FnMut</code>, and <code>FnOnce</code> does this closure implement?</summary>

```rust
let word = String::from("hi");
let show_len = move || word.len();
```

</details>

<details>
<summary>Answer</summary>

All three — `Fn` (and therefore `FnMut` and `FnOnce` too). `word` was fully
captured with `move`, but the body only calls `.len()` on it — it only ever
reads it, never mutates it, and never moves it out of the closure.

</details>

<details>
<summary>Does this compile?</summary>

```rust
fn needs_fn<F: Fn()>(f: F) {
    f();
    f();
}

let msg = String::from("bye");
let closure = move || {
    drop(msg);
};
needs_fn(closure);
```

</details>

<details>
<summary>Answer</summary>

No, `E0525`. The given closure moves `msg` out of itself with `drop(msg)`,
so it only implements `FnOnce` — but `needs_fn` is bounded by `Fn`, because
it plans to call `f` twice.

</details>

### Repair

Fix all five broken examples:

1. Fix `examples/09-closures-have-distinct-types.rs` so it compiles —
   without moving a shared variable between two different closures.
2. Fix `examples/10-forgot-mut-on-capturing-closure.rs` by adding a `mut`.
3. Fix `examples/11-missing-move-dangling-borrow.rs` by adding `move`.
4. Fix `examples/12-fnonce-closure-called-twice.rs` so `consume` is callable
   twice, without changing the signature of what it prints.
5. Fix `examples/13-fn-bound-rejects-fnonce-only.rs` so the closure passed
   to `apply_twice` is `Fn`, not just `FnOnce`.

### Implement

Five functions in `src/lib.rs`:

```sh
cargo test -p p2-02-01-closures-and-fn-traits
```

Each function is exactly one of the shapes you saw in "The concept" — an
`Fn` parameter, a function returning `impl Fn`, an `FnMut` parameter, and an
`FnOnce` parameter. Each function's doc comment states exactly what it
returns and under which conditions; don't guess.

### Build

Write a function that takes an `FnMut() -> bool` closure and call it
`retry_until_success`: call it up to `max_tries` times, stopping the moment
it returns `true`, and report at the end whether it ever succeeded
(`bool`).

Then write a second version — pick its own signature — that also reports
*how many tries* it took to succeed (`Option<u32>`, or whatever shape you
decide). Document in its doc comment why you chose that return shape.

### Challenge (optional)

Using `std::mem::size_of_val(&closure)`, compare the sizes of three
closures: one that captures nothing, one that captures a single `i32` with
`move`, and one that captures two `String`s with `move`. What pattern do
you see? (Hint: think of a closure as a compiler-generated struct — what
would its fields have been?)

(This part looks ahead.) Look up `Box<dyn Fn(i32) -> i32>` in the standard
docs and compare it with `impl Fn(i32) -> i32`. Why could `make_adder`
return `impl Fn`, but a function that sometimes returns one closure and
sometimes a different one, depending on an `if`, cannot?

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Closure | an unnamed, inline function, `\|x\| ...` | call sites that need a small, throwaway function |
| Capture | how a closure reaches variables around it | the compiler's automatic choice: shared, mutable, or `move` |
| `move` | forces a closure to take full ownership | whenever the closure must outlive the scope it was written in |
| `Fn` | only reads; callable any number of times | parameters that only ever need to read |
| `FnMut` | mutates; callable any number of times | parameters called repeatedly that need to change something |
| `FnOnce` | moves what it captured out; callable once | parameters called exactly once that need ownership |
| Function pointer (`fn`) | the type of a plain, capture-free function | anywhere an `Fn`/`FnMut`/`FnOnce` closure is expected |

### What you now know

- A closure is a real value with a real type, unique and unnameable — not
  syntax sugar for something else.
- The default capture is by reference — shared if the body only reads,
  mutable if the body mutates — and the compiler decides which from the
  body alone.
- `move` only decides *how* something is captured, not how many times the
  closure can be called; you need it whenever the closure must outlive the
  scope it was written in.
- `Fn`, `FnMut`, and `FnOnce` form a hierarchy: every `Fn` is also an
  `FnMut`, every `FnMut` is also an `FnOnce` — and that direction never
  reverses.
- A function that accepts "a closure" needs to be generic or write
  `impl Trait` in argument position; a function that returns "a closure"
  needs `impl Trait` in return position — because a closure's real type is
  never something you can write down.
- An ordinary function, used as a value, captures nothing and implements
  `Fn`/`FnMut`/`FnOnce` all at once, for free.

### What comes back later

- **Iterator adapters, which lean on `Fn`/`FnMut` constantly without
  re-explaining them** — [02.2 — Iterator adapters](../02-iterator-adapters/README.md)
- **Generics, in full** — [03.1 — Generic functions and structs](../../03-traits-and-generics/02-generic-functions-and-structs/README.md)
- **`impl Trait` and `dyn Trait`, and when to reach for each** — [03.3 — Trait objects vs. static dispatch](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md)
- **Handing a closure to another thread, where `move` is almost always mandatory** — [07.1 — Threads, `Mutex`, `Arc`](../../08-concurrency/01-threads-mutex-arc/README.md)

### Can you explain?

- Why does `std::any::type_name` print something like `{{closure}}` for a
  closure, and what does that mean for writing its type by hand?
- What's the difference between the default capture (by reference) and
  capturing with `move`, and when do you choose which?
- Why is `move |x| x + n` still `Fn`, even though it captured `n` with
  `move`?
- Explain the `Fn`/`FnMut`/`FnOnce` hierarchy with an example of your own
  (not from this lesson) — which follows from which?
- Why does a function that accepts "a closure" need to be generic? Why does
  a function that returns "a closure" need `impl Trait`?
- Why does a function pointer implement all three of
  `Fn`/`FnMut`/`FnOnce`, for free?

---

## Going further

- [The Rust Book — Closures](https://doc.rust-lang.org/book/ch13-01-closures.html) — the same ground, official and complete.
- [The Rust Reference — Closure types](https://doc.rust-lang.org/reference/types/closure.html) — the exact detail of how the compiler turns a closure into a struct, and how it decides how to capture.
- [`std::ops::Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html), [`FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html), [`FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html) — the standard library's own definition of all three traits.
- [Rust by Example — Closures](https://doc.rust-lang.org/rust-by-example/fn/closures.html) — more examples, including this same `Fn`/`FnMut`/`FnOnce` trio with shorter code.
