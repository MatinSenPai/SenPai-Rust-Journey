# Solution — 1.2.3 `Clone` and `Copy`

```rust
pub fn duplicated(text: String) -> (String, String) {
    let copy = text.clone();
    (text, copy)
}

pub fn array_survives(values: [i32; 4]) -> ([i32; 4], i32) {
    let mut total = 0;
    for value in values {
        total += value;
    }
    (values, total)
}

pub fn shrunk(values: Vec<i32>) -> Vec<i32> {
    let mut values = values;
    values.shrink_to_fit();
    values
}

pub fn doubled_up(values: Vec<String>) -> Vec<String> {
    let mut out = Vec::with_capacity(values.len() * 2);
    for value in values {
        out.push(value.clone());
        out.push(value);
    }
    out
}

pub fn repeated(text: String, times: usize) -> Vec<String> {
    if times == 0 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(times);
    for _ in 1..times {
        out.push(text.clone());
    }
    out.push(text);
    out
}
```

Two of these needed a clone. Two didn't. One needed exactly `n - 1` of them.

## `duplicated` — the clone that's actually justified

```rust
let copy = text.clone();
(text, copy)
```

This is what `.clone()` is for. The caller asked for two independent strings, and two independent strings means two buffers, and two buffers means an allocation. There is no cleverness available.

The test is written to prove independence rather than equality:

```rust
left.push('!');
assert_eq!(left, "hi!");
assert_eq!(right, "hi", "the two must not share a buffer");
```

If you'd found some way to hand back two names for one buffer, that assertion is what would catch it. In Python this test would fail for a list, because `second = first` shares. Here, sharing isn't even expressible without the tools from [Phase 2](../../../../phase2-intermediate/05-smart-pointers/02-rc-and-arc/README.md).

Order matters slightly: clone first, then build the tuple. Write `(text, text.clone())` and you've moved `text` into the tuple before the clone runs — `E0382`.

## `array_survives` — the one that needs no clone at all

```rust
for value in values {
    total += value;
}
(values, total)
```

Look at that carefully. The loop consumes `values`… and then the next line returns `values`. In [1.2.2](../../02-move-semantics/README.md) that was `E0382`.

It compiles because **`[i32; 4]` is `Copy`**. The loop didn't take the array; it took a copy of it, and the original is untouched. Change the type to `[String; 4]` and the same code stops compiling immediately.

That's the whole distinction in one function. `Copy` isn't "a cheaper clone" — it's *"a move of this type doesn't invalidate the source, because there's nothing to be responsible for."*

Sixteen bytes get copied here. Which is real work — but it's a stack copy with no allocator involved, and that's a different order of cost from `.clone()` on a `String`.

## `shrunk` — and why `.clone()` would also have worked

```rust
values.shrink_to_fit();
values
```

`shrink_to_fit` asks the allocator for a block of exactly the right size and moves the contents. It's the direct expression of what was asked for.

But so is this:

```rust
values.clone()
```

Because **cloning a `Vec` allocates exactly `len`, not `capacity`.** You can see it in the example output:

```text
roomy len/cap:  3/100
clone len/cap:  3/3
```

Both work. `shrink_to_fit` says what you mean, and there's a real difference besides: `.clone()` briefly holds *both* buffers, so the peak memory is the sum of the two. For a Vec you were shrinking because it was too big, that's the wrong direction.

Neither `capacity() == len()` is guaranteed by the language — it's what the standard library does. The test asserts it because that's the observable behaviour, the same way [1.2.1](../../01-stack-and-heap/README.md)'s test asserted the doubling.

## `doubled_up` — clone once, then move

```rust
for value in values {
    out.push(value.clone());
    out.push(value);
}
```

Two pushes, one clone. The first copy has to be a clone because the second push still needs the original; the second push is a move because nothing needs `value` afterwards.

Writing `out.push(value.clone()); out.push(value.clone());` also passes the tests and does twice the allocation. **The last use of a value should be a move.** That habit is worth building now: look at where a value is used for the last time, and don't clone there.

`Vec::with_capacity(values.len() * 2)` is free to write and saves the doubling reallocations, since the final length is known exactly.

## `repeated` — `n - 1` clones, not `n`

```rust
for _ in 1..times {
    out.push(text.clone());
}
out.push(text);
```

The obvious version writes `for _ in 0..times { out.push(text.clone()) }` and then `text` is dropped, unused, at the end of the function. That's one allocation made and one thrown away.

The range starting at 1 is doing the counting: `1..times` runs `times - 1` times. Then the original goes in last, moved rather than cloned. For `times == 1` the loop doesn't run at all and there are zero clones.

`for _ in` rather than `for n in` because the number isn't wanted. That's the same `_` from [1.1.3](../../../01-foundations/03-compound-types-and-destructuring/README.md), doing the same job in a different place: a position acknowledged and deliberately not named.

The `times == 0` guard is needed because `1..0` is an empty range that would leave the final `push` to run anyway, giving one element instead of none. A guard clause, from [1.1.5](../../../01-foundations/05-control-flow/README.md).

## What this lesson was really about

- **`.clone()` allocates.** For a `Vec<String>` it allocates once per element plus once for the Vec.
- **`Copy` is a promise, not an optimisation**: copying the bytes copies the whole value and there's nothing to clean up.
- **Every `Copy` type is `Clone`; the reverse isn't true.** `String` is `Clone` and can never be `Copy`.
- **The last use of a value should be a move.** Most of the wasteful clones in real code are on a value nobody needed afterwards.
- **When you reach for `.clone()`, ask what you'd need instead.** Usually a `&` — and that's [module 1.3](../../../03-borrowing-and-references/README.md).
