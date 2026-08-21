# Solution — 1.2.2 Move semantics

```rust
pub fn extended(mut text: String, suffix: char) -> String {
    text.push(suffix);
    text
}

pub fn reversed(values: Vec<i32>) -> Vec<i32> {
    let mut out = Vec::with_capacity(values.len());
    for offset in 0..values.len() {
        out.push(values[values.len() - 1 - offset]);
    }
    out
}

pub fn total_bytes_of(values: Vec<String>) -> usize {
    let mut bytes = 0;
    for value in &values {
        bytes += value.len();
    }
    bytes
}

pub fn take_first(mut values: Vec<String>) -> String {
    values.remove(0)
}

pub fn merged(left: Vec<i32>, right: Vec<i32>) -> Vec<i32> {
    let mut out = left;
    for value in right {
        out.push(value);
    }
    out
}
```

## `extended` — the `mut` that appeared in the signature

The exercise gave you `text: String`. The solution has `mut text: String`. That looks like changing the signature, and it isn't:

```rust
pub fn extended(mut text: String, suffix: char) -> String
```

**`mut` on a parameter is not part of the function's type.** The caller can't see it, doesn't care, and nothing about how they call you changes. It says one thing to the compiler: *inside this body, I intend to modify my own copy of this binding.* Which you may, because it's yours now — the caller gave it to you.

If you wrote this instead, it's identical:

```rust
pub fn extended(text: String, suffix: char) -> String {
    let mut text = text;
    text.push(suffix);
    text
}
```

That's shadowing from [1.1.1](../../../01-foundations/01-variables-mutability-shadowing/README.md) doing the same job in two lines. Both are fine; `mut` in the signature is what you'll see in real code.

And this is what a move buys you. In a language where the caller might still be holding that string, mutating it would be a rude surprise. Here it can't be: they gave it away.

## `reversed` — why `with_capacity` and not `new`

```rust
let mut out = Vec::with_capacity(values.len());
```

You know exactly how many are coming. Saying so means one allocation instead of a series of doubling reallocations, for four extra words of typing. It's the free version of [1.2.1](../../01-stack-and-heap/README.md)'s lesson.

The index arithmetic is where mistakes live. `values.len() - 1 - offset` walks from the end: at `offset` 0 that's the last element, and at `offset == len - 1` it's element zero. If you wrote `values.len() - offset` you'd index one past the end on the very first turn and panic.

`values[...]` on a `Vec<i32>` gives you a copy rather than a move, which is why this compiles at all — for a `Vec<String>` the same line is `E0507`, the error from `05-move-out-of-a-vec.rs`.

There is of course `values.reverse()`, which does it in place with no second `Vec`, and `.into_iter().rev().collect()`, which is what you'd actually write once you have [Phase 2](../../../../phase2-intermediate/02-iterators-and-closures/README.md). Building it by hand once is how you get to read those and know what they cost.

## `total_bytes_of` — the one that borrows

```rust
for value in &values {
```

Every other exercise here consumes. This one has a `&`, and the reason is worth stating: `.len()` only needs to *look* at each string. Taking ownership of each one just to ask its length, and then dropping it, would be work for nothing.

Drop the `&` and it still compiles and still passes, because `values` isn't wanted afterwards. The habit is the point. **Take ownership when you need the thing; borrow when you need to look at it.** That sentence is most of [module 1.3](../../../03-borrowing-and-references/README.md) in advance.

## `take_first` — the legal way to move out of a `Vec`

```rust
values.remove(0)
```

`values[0]` doesn't compile:

```text
error[E0507]: cannot move out of index of `Vec<String>`
```

The reason is worth having straight. If taking element zero out were allowed, the `Vec` would have a hole in it — a slot whose `String` had been given away — and there is no such thing as a `Vec` with a hole. Every position from 0 to `len` must hold a valid value.

`remove(0)` is legal because it fixes that: it takes the element out **and** shifts everything else down, so the `Vec` is one shorter and still has no holes. It hands you the element itself, moved out, yours.

That shifting costs something. Removing from the front of a thousand-element `Vec` moves 999 elements. Two things worth knowing:

- **`swap_remove(0)`** is instant: it moves the last element into the hole instead of shifting. Use it when the order doesn't matter.
- **`VecDeque`** is the collection built for taking from both ends, and it arrives in [Phase 2](../../../../phase2-intermediate/01-collections/README.md).

## `merged` — reusing an allocation instead of making one

```rust
let mut out = left;
for value in right {
    out.push(value);
}
out
```

`let mut out = left;` is a move, and it's doing real work: `out` takes over the buffer `left` already had, and everything pushed into it may fit without allocating at all. Starting from `Vec::new()` instead would throw away a perfectly good buffer and rebuild it.

`for value in right` consumes `right`, which is exactly right — the values are wanted, not looked at.

The standard library way is `out.append(&mut right)`, which moves the elements across in one memory operation instead of one at a time. It needs `&mut`, which is [1.3.1](../../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md).

## What this lesson was really about

- **A move is a transfer of responsibility, not a copy of data.** Three words are copied; the buffer stays exactly where it was.
- **A move costs the same whatever the size.** Ten bytes or ten megabytes, it is three words.
- **The source becomes unusable, and that is the entire point** — two owners would mean two frees, and a double free is one of the three bugs from [1.2.1](../../01-stack-and-heap/README.md).
- **Numbers don't move because they own nothing.** The dividing line isn't "small versus large"; it's "is anyone responsible for a buffer".
- **Take ownership when you need the value; borrow when you need to look.** Everything in module 1.3 follows from that sentence.
