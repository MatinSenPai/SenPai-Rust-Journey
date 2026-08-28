# 2.2.5 — Laziness and iterator performance

## At a glance

After this lesson you can:

- Prove — by counting, not by taking anyone's word for it — that a multi-stage chain only does work once it is consumed, and that each element runs through the **whole** chain one at a time, rather than each stage running once over all the data.
- Spot the trap of a `.collect()` accidentally left in the middle of a chain in your own code, and explain exactly what makes it expensive.
- Build an infinite iterator (`repeat`, `.cycle()`, `successors`) and say with confidence why such a thing could only exist under a lazy model in the first place.

**Time:** ~65 minutes · **Prerequisites:** [2.2.4 — Implementing `Iterator` and `IntoIterator` for your own type](../04-implementing-iterator/README.md), and specifically [2.2.2 — Iterator adapters](../02-iterator-adapters/README.md)

---

## Why this matters

Back in [2.2.2](../02-iterator-adapters/README.md), one sentence went by that you probably never felt the weight of: "adapters are lazy; nothing happens until you consume them." You believed it — maybe you even saw the output that backed it up — but you never had to *prove* it, and you never saw exactly what door that laziness opens. This lesson settles that debt.

Three things are about to happen.

First, we prove the laziness claim by counting, not by asserting it. We build a chain from an unbounded source, put `.take(3)` on top, and count exactly how many times each stage actually ran. The number you get back is exactly what a hand-written loop doing the same job would take — no more, no less. That is exactly what people mean when they call iterator chains a "zero-cost abstraction": writing them declaratively costs nothing extra at run time.

Second, that same laziness has a trap built into it. A `.collect()` that accidentally lands in the middle of a chain — not at the end of it — takes away the exact thing laziness was giving you: a full pass and a real allocation that were never needed.

Third — and this is where everything clicks into place — without that laziness, it would be impossible to write an iterator that never runs out at all. You cannot build an infinite `Vec`; you can build a *program* that does nothing until something finite asks it for values. `std::iter::repeat`, `.cycle()`, and `std::iter::successors` are all direct products of that one idea, and would not make sense without it.

---

## The concept

### From building to consuming: where the work actually happens

Build a chain whose every stage prints something in the middle of its work — then just build it, without consuming it:

```rust
let chain = vec![1, 2, 3, 4, 5]
    .into_iter()
    .map(|n| {
        println!("  map saw {n}");
        n * 2
    })
    .filter(|n| {
        println!("  filter saw {n}");
        n % 4 == 0
    });
println!("chain built — nothing printed above this line from map/filter");
```

```text
chain built — nothing printed above this line from map/filter
```

That's it. No "map saw 1", no "filter saw...". `chain` is a value — a small piece of data that says "a `vec![1,2,3,4,5]`, then this closure, then that one" — not something that has already *run*. Now consume it:

```rust
let result: Vec<i32> = chain.collect();
println!("result: {result:?}");
```

```text
  map saw 1
  filter saw 2
  map saw 2
  filter saw 4
  map saw 3
  filter saw 6
  map saw 4
  filter saw 8
  map saw 5
  filter saw 10
result: [4, 8]
```

Only now, the moment `.collect()` is called, do the closures actually run. And look at the order of the prints: "map saw 1" is immediately followed by "filter saw 2" — not all five values running through `map` first and then all five through `filter`. That order is exactly what the next section makes precise.

### Each element travels the whole chain, not each stage the whole data

Now try the same idea on an **unbounded** source — an open range `1..` with no ceiling at all — and put `.take(3)` on top. Each closure gets its own counter:

```rust
let mut map_calls = 0u32;
let mut filter_calls = 0u32;
let chain_result: Vec<i32> = (1..)
    .map(|n| {
        map_calls += 1;
        n * 2
    })
    .filter(|n| {
        filter_calls += 1;
        n % 3 == 0
    })
    .take(3)
    .collect();
println!("chain result: {chain_result:?}");
println!("chain: map ran {map_calls} times, filter ran {filter_calls} times");
```

```text
chain result: [6, 12, 18]
chain: map ran 9 times, filter ran 9 times
```

The source was unbounded — it had infinitely many numbers it *could* have handed over — and even so, `map` ran exactly 9 times, not one more. That only makes sense if evaluation is genuinely **demand-driven**: `.collect()` asks `.take(3)` for a value, `.take(3)` asks `.filter(...)` for a value, `.filter(...)` asks `.map(...)` for a value, and `.map(...)` asks `(1..)` for a value — exactly one number comes up, travels through the whole chain, and if `filter` rejects it, the whole request repeats for the next number. Nobody ever said "run everything through `map` first, then hand it all to `filter`" — because there is no such step to run.

This is the point where the behaviour earns its precise name: a **lazy iterator**, the thing you glimpsed in [2.2.2](../02-iterator-adapters/README.md), now counted and proven. It is a different claim from the eager/lazy pair you already know from `Option` combinators — there the question was about *one argument* (`.unwrap_or(x)` versus `.unwrap_or_else(|| x)`); here it is about an *entire chain*: the whole pipeline moves one element at a time, and only on demand.

### The exact same amount of work, just a different name: compared with a manual loop

If you are not sure that "9" is something specific to chains, write the identical job as a manual loop — no adapters at all:

```rust
let mut n = 1i32;
let mut loop_map_calls = 0u32;
let mut loop_filter_calls = 0u32;
let mut loop_result = Vec::new();
while loop_result.len() < 3 {
    loop_map_calls += 1;
    let doubled = n * 2;
    loop_filter_calls += 1;
    if doubled % 3 == 0 {
        loop_result.push(doubled);
    }
    n += 1;
}
println!("loop result:  {loop_result:?}");
println!("loop:  map ran {loop_map_calls} times, filter ran {loop_filter_calls} times");
```

```text
loop result:  [6, 12, 18]
loop:  map ran 9 times, filter ran 9 times
```

Identical. Same result, same 9 "doublings", same 9 "is this divisible" checks. The iterator chain did not do extra work, and it did not do less work either — it just named the loop's two steps ("double it", "keep only multiples of 3") instead of spelling out the bookkeeping by hand. This is exactly what "zero-cost abstraction" means: the declarative version does not pay any extra run-time cost for being more readable.

### Why this is always true: an adapter is just another `next()`

This behaviour is not luck — it is exactly what [2.2.4](../04-implementing-iterator/README.md) taught you, seen from the other side. When you implemented `Iterator` for a type yourself, everything that came after it — a `for` loop, a `.collect()`, a `.take()` — did exactly one thing: call `next()` over and over and ask "what's next?". `.map()` and `.filter()` have no magic of their own: they are just a `struct` holding another iterator inside them, and their own `next()` does the same thing — it asks the inner iterator for `next()`, and does something to the result if one comes back.

To see this with your own hands, do exactly what you did in [2.2.4](../04-implementing-iterator/README.md) again — a plain `struct`, a hand-written `impl Iterator`, plus a counter:

```rust
struct Doubling {
    inner: std::ops::Range<i32>,
    calls: u32,
}

impl Iterator for Doubling {
    type Item = i32;

    fn next(&mut self) -> Option<i32> {
        self.calls += 1;
        self.inner.next().map(|n| n * 2)
    }
}
```

`Doubling` borrows nothing special from the standard library — it is exactly the pattern you already wrote, just with a `calls` field that ticks up every time `next()` genuinely runs. Now put `.take(3)` on top — using `.by_ref()`, which takes a temporary borrow of the iterator so `doubling` itself is still usable afterward:

```rust
let mut doubling = Doubling {
    inner: 0..1_000_000,
    calls: 0,
};
println!("constructed — calls so far: {}", doubling.calls);

let first_three: Vec<i32> = doubling.by_ref().take(3).collect();
println!("first_three: {first_three:?}");
println!("next() actually ran {} times", doubling.calls);
```

```text
constructed — calls so far: 0
first_three: [0, 2, 4]
next() actually ran 3 times
```

Its inner range had a million numbers in it. `next()` ran three times. `Doubling` is exactly as lazy as `.map()` is — not because the standard library carves out a special case for your own type, but because **there is no other way `next()` could work**: nothing moves until someone calls it. That is exactly the mechanism sitting underneath `.map()`, `.filter()`, and every other adapter you have used so far.

```senpai-visual
{"kind":"concept","labels":["take() wants a value","filter wants one","map wants one","range produces it","map doubles it","filter tests it"]}
```

The part worth holding onto: the request for a value starts at the lowest consumer and travels *down* — all the way to the original source — and only then does a value travel back *up*, one stage at a time. No stage ever waits for "all" of anything; it only ever waits for one `next()` from the stage below it.

(A side note, for later: `Doubling` here only wraps one specific `Range<i32>`, not *any* iterator at all. Writing a version that works over any inner iterator is exactly what generics are for — in [2.3.2](../../03-traits-and-generics/02-generic-functions-and-structs/README.md).)

### The trap: a `.collect()` left in the middle of a chain

Now go to the opposite side of this same power. Build a hundred-thousand-element source, double every element — but this time, by accident, call `.collect()` right here:

```rust
const SOURCE_LEN: i32 = 100_000;

let mut before_map_calls = 0u32;
let mut before_filter_calls = 0u32;
let doubled: Vec<i32> = (1..=SOURCE_LEN)
    .map(|n| {
        before_map_calls += 1;
        n * 2
    })
    .collect();
println!(
    "doubled: {} elements (map ran {before_map_calls} times)",
    doubled.len()
);
```

```text
doubled: 100000 elements (map ran 100000 times)
```

Nothing strange has happened yet — this `.collect()` was deliberate, it built a full 100,000-element `Vec`, and `map` ran exactly that many times because it was supposed to. The problem starts on the *next* line, when all you actually wanted from `doubled` was the first three multiples of 3:

```rust
let first_three: Vec<i32> = doubled
    .into_iter()
    .filter(|n| {
        before_filter_calls += 1;
        n % 3 == 0
    })
    .take(3)
    .collect();
println!("first_three: {first_three:?} (filter ran {before_filter_calls} times)");
```

```text
first_three: [6, 12, 18] (filter ran 9 times)
```

`filter` only ran 9 times — just as lazy as it ever was; there is nothing wrong with this second chain. **The problem is the earlier line, sitting on `doubled`.** That first `.collect()` had nothing to do with "find three multiples of 3" — it was just a middle step that made itself completely un-lazy: it ran `map` a hundred thousand times, allocated a real 100,000-element `Vec`, and then, after all of that, started over from the beginning of that same `Vec` to search it — even though only its first 9 elements were ever needed.

Now write the identical job without that middle `.collect()` — one continuous chain, start to finish:

```rust
let mut after_map_calls = 0u32;
let after: Vec<i32> = (1..=SOURCE_LEN)
    .map(|n| {
        after_map_calls += 1;
        n * 2
    })
    .filter(|n| n % 3 == 0)
    .take(3)
    .collect();
println!("after: {after:?} (map ran {after_map_calls} times)");
```

```text
after: [6, 12, 18] (map ran 9 times)
```

Same answer, 6, 12, 18 — but `map` ran only 9 times this time, not a hundred thousand, and no intermediate `Vec` — no `doubled`, nothing — ever came into existence. The only difference between these two versions was one extra `.collect()` sitting in the middle; that single line bought a hundred thousand times more work. The fix is deliberately simple: **only put `.collect()` at the end of a chain you actually want to stop right there — never wherever "you need a `Vec`".**

### The direct payoff: iterators that never run out

Go back to that `(1..)` from a couple of sections ago — an open range, with no upper bound at all. Nothing strange was said about it, but it really is strange: under an **eager** model — where every adapter finishes its work immediately — writing `(1..)` would not even be possible. You cannot build a `Vec` with infinitely many elements; memory runs out, and the program never reaches the next line. It works here because `(1..)` is only a *description* — "one more than the last one" — and until someone, through `.take()` or any other bounded consumer, actually asks for a value, no number is ever produced.

The standard library ships a few ready-made shapes of the same idea:

```rust
let repeated: Vec<&str> = std::iter::repeat("frieren").take(4).collect();
println!("{repeated:?}");

let pattern = [1, 2, 3];
let cycled: Vec<i32> = pattern.iter().copied().cycle().take(8).collect();
println!("{cycled:?}");

let empty: Vec<i32> = Vec::new();
let cycled_empty: Vec<i32> = empty.iter().copied().cycle().take(5).collect();
println!("{cycled_empty:?}");
```

```text
["frieren", "frieren", "frieren", "frieren"]
[1, 2, 3, 1, 2, 3, 1, 2]
[]
```

`std::iter::repeat(x)` hands back the same value forever. `.cycle()` takes a **bounded** iterator and restarts it from the beginning, forever — with one sensible exception: cycling an empty source stays empty (there is nothing to repeat, so nothing comes back; it does not hang). None of these do a moment of work earlier than `.take(...)` asks for it — which is exactly what makes this code *writable* at all.

A third tool is more useful when the next value is built out of the previous one:

```rust
let powers: Vec<u32> = std::iter::successors(Some(1u32), |&x| Some(x * 2))
    .take(6)
    .collect();
println!("{powers:?}");
```

```text
[1, 2, 4, 8, 16, 32]
```

`std::iter::successors(first, next)` starts at `first` and calls `next` on the last value each time to build the one after it; the moment `next` returns `None`, the sequence stops right there — but here it never does, so without `.take(6)` it would keep going forever. Think of this as a **generator**: a description of "how to build the next value from the current one", not a list written out in advance.

---

## Hands on

```sh
cargo run -p p2-02-05-laziness-and-performance --example 01-nothing-runs-until-you-consume
cargo run -p p2-02-05-laziness-and-performance --example 02-demand-driven-and-loop-equivalence
cargo run -p p2-02-05-laziness-and-performance --example 03-mid-chain-collect-trap
cargo run -p p2-02-05-laziness-and-performance --example 04-infinite-iterator-family
cargo run -p p2-02-05-laziness-and-performance --example 05-custom-iterator-is-lazy-too
```

Then the two broken ones:

```sh
cargo run -p p2-02-05-laziness-and-performance --example 06-len-on-infinite-iterator --features broken
cargo run -p p2-02-05-laziness-and-performance --example 07-cycle-needs-clone --features broken
```

Then try:

1. In `02-demand-driven-and-loop-equivalence`, change `.take(3)` to `.take(1)`. Before running it, guess how many times `map` and `filter` run, and what the result is — then check.
2. In `03-mid-chain-collect-trap`, change `SOURCE_LEN` from `100_000` to `1_000_000`. How does the BEFORE version's `map ran ... times` number change? How does the AFTER version's?
3. In `04-infinite-iterator-family`, change `pattern.iter().copied().cycle().take(8)` to `.take(2)`. Did it even need to restart `pattern` from the beginning?

---

## Errors you will meet

### No error at all — a mid-chain `.collect()` works, it is just expensive

The same thing you saw in "The trap", framed as an error: the BEFORE version's code compiles completely cleanly, runs completely cleanly, and gives a completely correct answer — `[6, 12, 18]`, identical to the AFTER version.

**What the compiler is objecting to:** nothing. Nowhere does it say "this `.collect()` is unnecessary" — type-wise, it is perfectly valid: you built a `Vec<i32>`, then chained more adapters on top of that `Vec`. The compiler has no idea you *intended* the chain to keep going; it only sees two pieces of correct code.

**The fix:** remove the middle `.collect()` and make it one chain:

```rust
let after: Vec<i32> = (1..=100_000)
    .map(|n| n * 2)
    .filter(|n| n % 3 == 0)
    .take(3)
    .collect();
```

**Why that's the fix:** with one `.collect()`, `map` runs only 9 times instead of a hundred thousand — exactly the number you counted for real in "The trap". Nothing about the surface of the code signals this difference; both versions look almost identical and produce identical output. The only way to catch this class of bug is knowing the rule: **only place `.collect()` where you genuinely want the chain to stop, never wherever "you happen to need a `Vec`".**

### `E0599` — `.len()` does not mean anything on an infinite iterator

```text
error[E0599]: no method named `len` found for struct `std::iter::Repeat<A>` in the current scope
    --> phase2-intermediate\02-iterators-and-closures\05-laziness-and-performance\examples\06-len-on-infinite-iterator.rs:11:28
     |
  11 |     println!("{}", forever.len());
     |                            ^^^
     |
help: there is a method `le` with a similar name, but with different arguments
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:3977:5
     |
3977 | /     fn le<I>(self, other: I) -> bool
3978 | |     where
3979 | |         I: IntoIterator,
3980 | |         Self::Item: PartialOrd<I::Item>,
3981 | |         Self: Sized,
     | |____________________^

For more information about this error, try `rustc --explain E0599`.
```

**What the compiler is objecting to:** `.len()` is not defined on `Iterator` at all — only `ExactSizeIterator` has it, for iterators that already know exactly how many elements are left (a `Range`, or `.iter()` on a `Vec`). `std::iter::Repeat` never runs out, so it has nothing to call a "remaining length", and it never implements `ExactSizeIterator`. The compiler even suggests a similarly-named method (`le`), precisely because it found no real method by this name.

**The fix:** if you genuinely need a number, bound it first:

```rust
let forever = std::iter::repeat(1);
let taken: Vec<i32> = forever.take(5).collect();
println!("{}", taken.len());
```

```text
5
```

**Why that's the fix:** "how many are left" is not a meaningful question for something that never runs out — that is exactly why the compiler rejected it, not because of some arbitrary restriction. The question that *is* meaningful is "how many are in this piece I just took?" — and after `.take(5)`, that piece is an ordinary, fully bounded `Vec`.

### `E0277` and `E0599` — `.cycle()` does not work without `Clone`

```text
error[E0277]: the trait bound `Doubling: Clone` is not satisfied
    --> phase2-intermediate\02-iterators-and-closures\05-laziness-and-performance\examples\07-cycle-needs-clone.rs:30:37
     |
  30 |     let cycled: Vec<i32> = doubling.cycle().take(5).collect();
     |                                     ^^^^^ the trait `Clone` is not implemented for `Doubling`
     |
note: required by a bound in `cycle`
    --> C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\traits\iterator.rs:3592:23
     |
3590 |     fn cycle(self) -> Cycle<Self>
     |        ----- required by a bound in this associated function
3591 |     where
3592 |         Self: Sized + [const] Clone,
     |                       ^^^^^^^^^^^^^ required by this bound in `Iterator::cycle`
help: consider annotating `Doubling` with `#[derive(Clone)]`
     |
  11 + #[derive(Clone)]
  12 | struct Doubling {
     |

error[E0599]: the method `take` exists for struct `Cycle<Doubling>`, but its trait bounds were not satisfied
  --> phase2-intermediate\02-iterators-and-closures\05-laziness-and-performance\examples\07-cycle-needs-clone.rs:30:45
   |
11 | struct Doubling {
   | --------------- doesn't satisfy `Doubling: Clone`
...
30 |     let cycled: Vec<i32> = doubling.cycle().take(5).collect();
   |                                             ^^^^ method cannot be called on `Cycle<Doubling>` due to unsatisfied trait bounds
   |
  ::: C:\Users\khmja\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib/rustlib/src/rust\library\core\src\iter\adapters\cycle.rs:15:1
   |
15 | pub struct Cycle<I> {
   | ------------------- doesn't satisfy `Cycle<Doubling>: Iterator`
   |
   = note: the following trait bounds were not satisfied:
           `Doubling: Clone`
           which is required by `Cycle<Doubling>: Iterator`
           `Cycle<Doubling>: Iterator`
           which is required by `&mut Cycle<Doubling>: Iterator`
help: consider annotating `Doubling` with `#[derive(Clone)]`
   |
11 + #[derive(Clone)]
12 | struct Doubling {
   |

Some errors have detailed explanations: E0277, E0599.
For more information about an error, try `rustc --explain E0277`.
```

**What the compiler is objecting to:** two errors, one root cause. When `.cycle()` runs out of its original iterator, it has to be able to restart it, from the very beginning, all over again — and the only way to do that is to keep a fresh copy of the starting state around. That's why `cycle`'s own signature requires `Self: Clone`. `Doubling` never got `#[derive(Clone)]`, so that requirement fails; the first error says exactly this. The second error is just a cascade of the same problem: because `Cycle<Doubling>` is not an `Iterator` at all (precisely due to that missing bound), `.take()` cannot be found on it either — fixing the first error takes the second one with it.

**The fix:** exactly what the compiler suggested:

```rust
#[derive(Clone)]
struct Doubling {
    inner: std::ops::Range<i32>,
    calls: u32,
}
```

```text
[0, 2, 4, 0, 2]
```

**Why that's the fix:** both of `Doubling`'s fields — a `Range<i32>` and a `u32` — already implement `Clone` themselves, so `#[derive(Clone)]` has nothing extra to figure out; it just lets the compiler build exactly what it already proposed. Now `.cycle()` makes a fresh copy of `Doubling`'s *original* state (`inner: 0..3, calls: 0`) every time it runs out, and starts over from there — which is exactly what explains `[0, 2, 4, 0, 2]`: the first lap gives `0..3` (doubled: `0, 2, 4`), and the second lap starts again from `0..3`, not from wherever the first lap stopped.

---

## Exercises

### Warm up

<details>
<summary>What does this print?</summary>

```rust
let v = vec![1, 2, 3];
let chain = v.iter().map(|n| {
    println!("mapping {n}");
    n * 2
});
println!("done building");
```

</details>

<details>
<summary>Answer</summary>

```text
done building
```

Just that one line. `chain` was only built, never consumed — so the closure inside `.map()` never ran even once. "mapping 1" and the rest never print.

</details>

<details>
<summary>In the same <code>(1..).map(...).filter(|n| n % 3 == 0)</code> chain from "The concept", if you write <code>.take(1)</code> instead of <code>.take(3)</code>, what are the result and the <code>map</code>/<code>filter</code> call counts?</summary>

Decide before reading the answer.

</details>

<details>
<summary>Answer</summary>

```text
result=[6] map=3 filter=3
```

The first multiple of 3 is `6` itself (when `n = 3`), and `.take(1)` is satisfied right there. Both `map` and `filter` ran exactly three times — the same pattern as before, just with a smaller target.

</details>

<details>
<summary>Does <code>std::iter::repeat(1).len()</code> compile?</summary>

Decide before reading "Errors you will meet".

</details>

<details>
<summary>Answer</summary>

No — `E0599`. `.len()` is only defined on `ExactSizeIterator`, and `std::iter::Repeat` never runs out, so it never implements that trait.

</details>

<details>
<summary>What does <code>[1, 2].iter().cycle().take(5).copied().collect::&lt;Vec&lt;i32&gt;&gt;()</code> return?</summary>

Decide before reading on.

</details>

<details>
<summary>Answer</summary>

```text
[1, 2, 1, 2, 1]
```

`.cycle()` starts back at `1` right after `2`; `.take(5)` stops it in the middle of the third lap.

</details>

<details>
<summary>Does a <code>struct</code> that implements <code>Iterator</code> but has no <code>#[derive(Clone)]</code> compile with <code>.cycle()</code>?</summary>

Decide before reading on.

</details>

<details>
<summary>Answer</summary>

No. `.cycle()` needs `Self: Clone` — it has to be able to build a fresh copy of the starting state every time it runs out. Without `Clone`, you get `E0277`.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/06-len-on-infinite-iterator.rs` so it compiles — without assuming `forever` has a fixed length. If you genuinely need a number, bound it first.
2. Fix `examples/07-cycle-needs-clone.rs` so it compiles and runs — by adding exactly what the compiler's help message suggested.

And this one too — even though it neither panics nor gets rejected, it is just as real:

3. Write a version of `examples/03-mid-chain-collect-trap.rs` that turns the BEFORE version into one continuous chain too — that is, remove that first `.collect()` entirely — and run it again. How does the `map ran ... times` number change?

### Implement

Five functions in `src/lib.rs`, each built around one of this lesson's tools:

```sh
cargo test -p p2-02-05-laziness-and-performance
```

None of them need generics, `Box`, or an `impl Trait` return type — all of them work on concrete types you already know. Each function's doc comment states exactly what it returns; don't guess.

### Build

Write a `struct` that implements `Iterator` — pick any simple transformation you like (tripling, squaring, negating, turning a value into a `String`, anything) — with a `calls: u32` field that increments every time `next()` genuinely runs. Build one instance over a large source (a few hundred thousand elements), take just the first few with `.by_ref().take(n)`, and prove by printing `calls` afterward that it only ran as many times as it needed to — not one more. In a comment, explain why that result was predictable in advance, given how `Iterator` actually works.

### Challenge (optional)

Change `examples/02-demand-driven-and-loop-equivalence.rs` so that, instead of counting calls, it measures real elapsed time with `std::time::Instant` for both the chain version and the loop version, over a `.take(n)` with `n` much larger (several million). Are the timings as close as the call counts were? Measuring this kind of thing properly and repeatably, without operating-system noise, is exactly what [2.7.5 — Benchmarking with criterion](../../07-project-structure-and-testing/05-benchmarking-with-criterion/README.md) teaches you.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Lazy iterator (demand-driven evaluation) | an adapter does nothing by itself; each element travels the whole chain, one at a time, only when something downstream asks `.next()` for it | understanding why long chains carry no extra cost |
| Zero-cost abstraction | an iterator chain does exactly as much work as its hand-written equivalent | deciding without worrying about performance you haven't measured |
| Infinite iterator | an iterator with no defined end; safe because nothing runs until something bounded like `.take()` asks for values | `std::iter::repeat`, `.cycle()`, an open range (`1..`) |
| Generator | a description of how to build the next value from the current one, not a list written out in advance | what `std::iter::successors` is |
| `std::iter::successors` | a generator that builds the next value from the current one, until `None` comes back | sequences whose rule you know but whose length you don't |
| `.by_ref()` | a temporary borrow of an iterator, so it is still usable afterward | taking a first few elements without losing the rest |
| Mid-chain `.collect()` trap | a completely unrelated `Vec` built only to get past a middle step | spotting code that "works" but is expensive for no reason |

### What you now know

- An adapter chain is just a value, not something that has already run; work only starts when a consuming method is called.
- From an unbounded source, only as many elements as a `.take(n)` needs actually travel through the whole chain — never one extra.
- A chain and its equivalent hand-written loop do exactly the same amount of work; that is what makes them "zero-cost abstractions".
- This laziness is not luck: every adapter is just another `next()` that asks its inner iterator for `next()` — exactly the mechanism you wrote by hand in [2.2.4](../04-implementing-iterator/README.md).
- A `.collect()` accidentally left in the middle of a chain produces no error at all — it just builds a completely unnecessary `Vec` and buys an extra full pass.
- `std::iter::repeat`, `.cycle()`, and `std::iter::successors` only make sense under this lazy model; under an eager one, writing them would not have been possible at all.

### What comes back later

- **Generics — writing a `Doubling` that works over any inner iterator, not just `Range<i32>`** — [2.3.2 — Generic functions and structs](../../03-traits-and-generics/02-generic-functions-and-structs/README.md)
- **`impl Trait` in return position — returning the lazy iterator itself from a function, without `.collect()`ing it** — [2.3.7 — Static vs. dynamic dispatch](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md)
- **Measuring performance properly, with `criterion`** — [2.7.5 — Benchmarking with criterion](../../07-project-structure-and-testing/05-benchmarking-with-criterion/README.md)

### Can you explain?

- Why does building a `.map().filter()` chain on its own never run any closures?
- When you put `.take(3)` on an unbounded source, where does the request start, and which direction does it travel?
- Why do we consider an iterator chain and its equivalent manual loop "performance-equivalent"?
- Why is the `Doubling` you wrote yourself lazy for exactly the same reason `.map()` is?
- What does a mid-chain `.collect()` cost the program, even though it produces no error at all?
- Why could `std::iter::repeat(1)` not have existed at all under an eager model?

---

## Going further

- [`std::iter` docs](https://doc.rust-lang.org/std/iter/index.html) — the full list of iterator-building tools, including `repeat`, `successors`, `once`, and more.
- [`Iterator` trait docs](https://doc.rust-lang.org/std/iter/trait.Iterator.html) — `next()`'s own signature, and exactly what `.cycle()`, `.take()`, and the rest require of `Self`.
- [The Rust Book — comparing loop and iterator performance](https://doc.rust-lang.org/book/ch13-04-performance.html) — the official version of the claim you proved by counting in "The exact same amount of work, just a different name".
- [`std::iter::Cycle` docs](https://doc.rust-lang.org/std/iter/struct.Cycle.html) — the exact struct `.cycle()` returns; the same page states why `Self: Clone` is its condition.
