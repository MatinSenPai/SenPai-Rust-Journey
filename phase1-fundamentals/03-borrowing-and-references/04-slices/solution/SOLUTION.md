# Solution — 1.3.4 Slices

```rust
pub fn count_above(values: &[i32], threshold: i32) -> usize {
    let mut found = 0;
    for value in values {
        if *value > threshold {
            found += 1;
        }
    }
    found
}

pub fn middle(values: &[i32]) -> &[i32] {
    if values.len() < 3 {
        return &[];
    }
    &values[1..values.len() - 1]
}

pub fn halves(values: &[i32]) -> (&[i32], &[i32]) {
    values.split_at(values.len() / 2)
}

pub fn window_sum(values: &[i32], start: usize, length: usize) -> i32 {
    let window = &values[start..start + length];
    let mut sum = 0;
    for value in window {
        sum += value;
    }
    sum
}

pub fn double_in_place(values: &mut [i32]) {
    for value in values {
        *value *= 2;
    }
}
```

Not one of these allocates. Three of them hand back a view of the caller's own numbers.

## `count_above` — the parameter type is the whole point

```rust
for value in values {
    if *value > threshold {
```

`value` here is a `&i32`, not an `i32`, because looping over a `&[i32]` hands you a look at each element rather than the element. So the comparison needs a `*` to get at the number — the same dereference from [1.3.1](../../01-shared-and-mutable-refs/README.md).

You could also have written `if value > &threshold`, which compares two references and is exactly as correct and slightly stranger to read.

The interesting test isn't the arithmetic:

```rust
assert_eq!(count_above(&fixed, 3), 2);
assert_eq!(count_above(&grown, 3), 2);
assert_eq!(count_above(&grown[..2], 3), 1);
assert_eq!(count_above(&fixed[2..], 3), 1);
```

An array, a Vec, and a piece of each — one function, no overloads, no generics, nothing written twice. Had the parameter been `&Vec<i32>`, two of those four lines would be `E0308`.

## `middle` — the guard is the exercise

```rust
if values.len() < 3 {
    return &[];
}
&values[1..values.len() - 1]
```

Write it without the guard and it looks fine, and then it panics on a one-element slice: `values.len() - 1` is `0`, so the range is `1..0`, which runs backwards, and you get `slice index starts at 1 but ends at 0`.

On an *empty* slice it's worse still. `values.len() - 1` on a `usize` of `0` underflows — the overflow from [1.1.2](../../../01-foundations/02-scalar-types-and-overflow/README.md), which panics in debug and wraps to a colossal number in release. That's two different failures from one missing line, and only the first one shows up in testing.

`return &[];` works because an empty slice needs nothing to point at, so it borrows nothing and fits any lifetime. That's also why `&[]` is fine as a value here while `[]` would not be.

## `halves` — the standard library already had it

```rust
values.split_at(values.len() / 2)
```

One line, and it's the whole function. `split_at` returns exactly the tuple that was asked for, and integer division does the "extra one goes right" rule for free: `3 / 2` is `1`, so the cut is after one element and the second half gets two.

The hand-written version is also correct:

```rust
let cut = values.len() / 2;
(&values[..cut], &values[cut..])
```

Same result, three times the surface area for a mistake. This is the reason for that "read the `slice` documentation top to bottom once" in the lesson: `split_at`, `first`, `last`, `contains`, `starts_with`, `swap` and `sort` are all sitting there already.

## `window_sum` — the panic you don't have to write

```rust
let window = &values[start..start + length];
```

The specification says the function panics when the window runs past the end. Nothing here checks for that, and it panics anyway — taking the slice does the check itself, and its message names both numbers:

```text
range end index 7 out of range for slice of length 3
```

That's a better error than anything you'd have written by hand, and it's free. Adding your own `if` in front of it would produce a second, worse message for the same condition.

Note that the two-step version reads better than one long expression:

```rust
let window = &values[start..start + length];
let mut sum = 0;
for value in window { /* ... */ }
```

`window` costs nothing — it's two words on the stack — so naming the thing you're about to add up is free clarity.

## `double_in_place` — writing through the view

```rust
for value in values {
    *value *= 2;
}
```

`values` is a `&mut [i32]`, so the loop hands you a `&mut i32` each turn and `*value *= 2` writes through it into the caller's buffer. Nothing is returned and nothing is allocated.

The index version also works:

```rust
for index in 0..values.len() {
    values[index] *= 2;
}
```

It's the same thing with a bounds check per turn, and `cargo clippy` will tell you so with `needless_range_loop`.

The test that matters is the second one:

```rust
double_in_place(&mut values[1..4]);
assert_eq!(values, vec![1, 4, 6, 8, 5]);
```

The function was handed a window of three elements and it changed exactly those three. It had no way to reach the other two, because a slice knows only where it starts and how many elements it has.

## What this lesson was really about

- **A slice is a borrow with a length.** Two words: an address and a count.
- **`&[T]` is the parameter type.** It accepts arrays, Vecs, and pieces of either, and costs the caller nothing.
- **A view returned from a function is still a view.** `middle` gives back the caller's own numbers, which is why nothing needed to be copied.
- **The bounds check happens when the slice is made.** You get a precise panic for free; what you don't get is a compile error.
- **`&str` is this, over bytes.** Everything here is the machinery you've been using for text since the first lesson — see [1.4.1](../../../04-text-and-strings/01-string-vs-str/README.md).
