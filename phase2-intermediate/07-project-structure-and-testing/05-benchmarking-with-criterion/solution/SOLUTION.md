# Solution

```rust
pub fn sum_of_squares_loop(n: u32) -> u64 {
    let mut total: u64 = 0;
    for i in 1..=n {
        total += (i as u64) * (i as u64);
    }
    total
}

pub fn sum_of_squares_iter(n: u32) -> u64 {
    (1..=n).map(|i| (i as u64) * (i as u64)).sum()
}
```

The `as u64` cast happens *before* the multiply in both — `i * i` would overflow a `u32` well before `n` gets large, since squaring is what overflows first, not the running total.

## The Build exercise's real numbers — and why they are surprising

`solution/benches/comparison.rs` adds the third benchmark group the Build exercise asks for, comparing both functions at `n = 10_000`. Run it: `cargo bench --manifest-path solution/Cargo.toml --bench comparison -- sum_of_squares`. Real output from this machine:

```text
sum_of_squares_loop_10000
                        time:   [5.7788 µs 5.7883 µs 5.7998 µs]
```

```text
sum_of_squares_iter_10000
                        time:   [1.9452 ns 1.9597 ns 1.9757 ns]
```

That is not the close race you'd expect from [2.2.5](../../../02-iterators-and-closures/05-laziness-and-performance/README.md)'s "zero-cost abstraction" — the iterator version measured roughly **3,000 times faster**, not roughly the same. Both are correct (`both_implementations_agree_across_a_range` checks every `n` from 0 to 49), so this isn't a bug in either one.

Timing both at a few very different `n` values makes the cause obvious:

```text
n=       10000  loop=       8.6µs   iter=         0ns
n=     1000000  loop=     849.2µs   iter=         0ns
n=   100000000  loop=   55.7348ms   iter=         0ns
```

`sum_of_squares_loop`'s time scales with `n`, as an honest O(n) loop should. `sum_of_squares_iter`'s time does not move at all, at any size — the signature of the compiler having proven the entire `.map().sum()` reduction equals a closed-form arithmetic formula (the sum of the first `n` squares has a well-known one) and replacing the loop with it outright, not running it faster.

This is worth being precise about, because it is a genuinely different mechanism from the `square`/`black_box` comparison in the lesson body, not the same trick showing up twice:

- The `square(7)` vs. `square(black_box(7))` gap was **constant folding**: the compiler knew the exact input (a literal `7`) and precomputed *that one answer* at compile time. Wrapping the input in `black_box` defeats exactly this, because the compiler can no longer assume it knows the value.
- This gap is **algebraic strength reduction**: `black_box(n)` genuinely hides the runtime value of `n` — the compiler has no idea whether it's `10_000` or `100_000_000` — but it doesn't need to know, because it has proven a formula that is correct for *every possible* `n`. `black_box` was never meant to stop that, and arguably shouldn't: the closed-form answer is exactly as correct as the loop's.

Why the hand-written loop doesn't get the same treatment while the `.sum()` chain does is a question about which LLVM pass recognizes which shape — answering that for certain would mean reading the generated assembly, which is past this lesson's scope. The honest lesson is smaller and more useful than "iterators are 3,000x faster": the unit tests prove *what* the two functions compute; only a benchmark tells you whether they cost the same to compute it, and here, provably identical output hid a wildly different cost. That is precisely the gap this lesson exists to close.
