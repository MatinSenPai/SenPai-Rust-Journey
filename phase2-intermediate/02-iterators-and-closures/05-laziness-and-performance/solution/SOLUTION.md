# Solution — 2.2.5 Laziness and iterator performance

```rust
pub fn powers_of_two(n: usize) -> Vec<u64> {
    std::iter::successors(Some(1u64), |&x| Some(x * 2))
        .take(n)
        .collect()
}

pub fn cycle_to_length(pattern: &[i32], total_len: usize) -> Vec<i32> {
    pattern.iter().copied().cycle().take(total_len).collect()
}

pub fn first_n_multiples_of(k: u32, n: usize) -> Vec<u32> {
    (1..).map(|i| i * k).take(n).collect()
}

pub fn multiples_of_k_below(k: u32, limit: u32) -> Vec<u32> {
    if k == 0 {
        return Vec::new();
    }
    (1..).map(|i| i * k).take_while(|&m| m < limit).collect()
}

pub fn first_n_even_with_scan_count(numbers: &[i32], n: usize) -> (Vec<i32>, usize) {
    let mut found = Vec::new();
    let mut scanned = 0;
    for &value in numbers {
        if found.len() >= n {
            break;
        }
        scanned += 1;
        if value % 2 == 0 {
            found.push(value);
        }
    }
    (found, scanned)
}
```

None of these five functions needed a custom `struct` or a complicated loop — three of them are just the tools you saw in "The concept", aimed at their own input; the fourth had a small edge you had to stay alert for; the fifth didn't even need to borrow anything new from this lesson.

## `powers_of_two` — `successors`, no custom `struct` at all

```rust
std::iter::successors(Some(1u64), |&x| Some(x * 2))
    .take(n)
    .collect()
```

`successors` is exactly the generator you saw in "The direct payoff" — it starts at `1`, doubles the previous value each time, and since it never returns `None`, the only thing stopping it is `.take(n)`. For `n = 0`, `.take(0)` never calls the closure even once — the result is `vec![]`, exactly as the specification asked.

## `cycle_to_length` — `.cycle()`, with a free trick for empty input

```rust
pattern.iter().copied().cycle().take(total_len).collect()
```

No separate check for `pattern.is_empty()` was needed — the standard library gives you that behaviour for free: `.cycle()` over an empty source stays empty itself, it neither hangs nor panics. `.copied()` just turns `.iter()`'s `&i32`s into `i32`, which is what the function's signature asked for.

## `first_n_multiples_of` — the same open range `1..` from "The concept"

```rust
(1..).map(|i| i * k).take(n).collect()
```

The same `(1..).map(...).take(n)` pattern used to prove laziness, this time just to generate a sequence. Since `k` is never `0` — the specification guarantees that — there is no risk of an endless loop here.

## `multiples_of_k_below` — why `k == 0` has to be checked before anything else

```rust
if k == 0 {
    return Vec::new();
}
(1..).map(|i| i * k).take_while(|&m| m < limit).collect()
```

This is exactly the danger this lesson warned about. If `k` were zero, every multiple would be zero too, and `0 < limit` stays true forever for any positive `limit` — `.take_while()` would never see its condition fail, and without that `if`, this function would have become exactly the unwanted infinite iterator "Errors you will meet" talked about. That three-line `if k == 0` is exactly what stops it.

## `first_n_even_with_scan_count` — a plain loop, not a chain

```rust
let mut found = Vec::new();
let mut scanned = 0;
for &value in numbers {
    if found.len() >= n {
        break;
    }
    scanned += 1;
    if value % 2 == 0 {
        found.push(value);
    }
}
(found, scanned)
```

This one is deliberately a manual loop, not an adapter chain — exactly the "same amount of work, just a different name" argument from "The concept". `scanned` only ticks up once an element has genuinely been examined; the `found.len() >= n` check happens before that, so for `n = 0` the loop never looks at a single element and `scanned` stays zero — exactly what the specification promised.

## What this lesson was really about

- **None of these five functions needed generics, `Box`, or an `impl Trait` return type** — all of them worked on concrete types you already know.
- **`(1..)` and `successors` are both infinite sources; the only difference is where the next value comes from** — one from a counter, the other from the previous value itself.
- **`multiples_of_k_below` was the one place you genuinely had to stay alert** — without the `k == 0` check, exactly the thing this lesson warned about (an unwanted infinite iterator) would have happened right here.
- **`first_n_even_with_scan_count` was a reminder that "laziness" is an idea, not just a tool** — even with no adapters at all, the same "only look at as much as you need" logic can be written with a plain loop.
