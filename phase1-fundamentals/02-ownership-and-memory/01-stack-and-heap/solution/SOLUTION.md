# Solution — 1.2.1 Stack and heap

```rust
use std::mem::{size_of, size_of_val};

pub fn stack_sizes() -> (usize, usize, usize) {
    (
        size_of::<[i64; 10]>(),
        size_of::<Vec<i64>>(),
        size_of::<String>(),
    )
}

pub fn capacity_after_pushes(pushes: usize) -> usize {
    let mut values: Vec<i32> = Vec::new();
    for n in 0..pushes {
        values.push(n as i32);
    }
    values.capacity()
}

pub fn reserve_for(expected: usize) -> Vec<u8> {
    Vec::with_capacity(expected)
}

pub fn header_and_heap(text: String) -> (usize, usize) {
    (size_of_val(&text), text.len())
}

pub fn total_bytes(values: Vec<i64>) -> usize {
    size_of_val(&values) + values.capacity() * size_of::<i64>()
}
```

## `stack_sizes` — the answer is the lesson

```text
[i64; 10]:   80
Vec<i64>:    24
String:      24
```

Eighty is unsurprising: ten eight-byte numbers, sitting where you put them.

Twenty-four is the whole point. **It doesn't matter what's in the `Vec`.** Empty, three items, a million items — the value itself is always three machine words:

| word | holds |
|---|---|
| 1 | where the data is |
| 2 | how many items there are |
| 3 | how many there is room for |

And `String` is the same 24, because a `String` *is* a `Vec<u8>` with a promise attached. Same three words, same layout.

That's why the test is written as `3 * size_of::<usize>()` and not as `24`. On a 32-bit machine — a microcontroller, `wasm32` — a word is four bytes and the answer is 12. Writing the literal would have made the test lie about what it's checking.

## `capacity_after_pushes` — measuring the growth

```text
pushes:    0  1  2  3  4  5  6  7  8  9 10 ... 16 17
capacity:  0  4  4  4  4  8  8  8  8 16 16 ... 16 32
```

Not one at a time. **Doubling.** Which is why `push` is cheap on average even though it occasionally has to allocate a bigger block and copy everything into it.

Work out why doubling matters. Suppose it grew by one each time instead: pushing `n` items would mean `n` allocations and `1 + 2 + ... + n` copies, which is `n²/2` — a hundred thousand pushes becomes five billion copies. Doubling means about `log₂(n)` allocations and fewer than `2n` copies total. Each individual push is either instant or expensive, but the *average* over any run is a small constant. That's what "amortised O(1)" means, and it's the same trick behind Python's `list` and Java's `ArrayList`.

The first jump is to 4 rather than 1 because a one-element allocation is nearly always a waste; the standard library starts at 4 for small element types.

**None of this is guaranteed by the language.** It's what the current standard library does. Depend on the amortised cost; don't depend on the exact numbers.

## `reserve_for` — the point of `with_capacity`

```rust
Vec::with_capacity(expected)
```

One line, and the test is the interesting part:

```rust
assert_eq!(reserved.len(), 0);
assert!(reserved.capacity() >= 100);
```

**Length zero, capacity a hundred.** Reserving room is not filling it. `reserve_for(100)[0]` would still panic, because there is no element zero — there's just room where one could go.

This is the fix for the growth above. If you know roughly how many items are coming, `with_capacity` allocates once and no copying ever happens. In a request handler that builds a response of known length, that's a real saving for one word of typing.

`capacity() >= 100` rather than `== 100` because the allocator is allowed to hand back a slightly larger block if that suits it. Testing for exactly what you asked for would be testing an implementation detail.

## `header_and_heap` — the two halves, side by side

```rust
(size_of_val(&text), text.len())
```

```text
"hello"  ->  (24, 5)
""       ->  (24, 0)
"سلام"   ->  (24, 8)
```

The first number never moves. The second is the actual text.

`size_of_val` sounds as though it should measure the whole value, and the fact that it doesn't is worth understanding rather than memorising: **it measures the part that lives where the value lives.** A `String` on the stack is three words; what those words point at is somewhere else entirely and is not part of the `String`'s size.

Notice the Persian case landing at 8 for four letters. That's [1.1.6](../../../01-foundations/06-vec-and-string-basics/README.md) again from the memory side: `len()` is the heap byte count because that's the number the `String` is actually storing.

## `total_bytes` — capacity, not length

```rust
size_of_val(&values) + values.capacity() * size_of::<i64>()
```

The trap is writing `.len()`. It compiles, it passes a careless test, and it's wrong.

A `Vec` with three items and room for ten is **holding all ten slots**. The allocator gave it that block; nobody else can use any of it until the `Vec` gives it back. Asking "how much memory is this responsible for" means asking about the block, not about the part currently in use.

This matters in real systems. A `Vec` that grew to a million items and then had 999,999 removed still holds the million-item block — `pop` and `remove` never shrink the allocation. If you actually need the memory back, `shrink_to_fit()` is the request, and it's the sort of thing you do after loading a large file and trimming it down, not in a hot loop.

## What this lesson was really about

- **Every value has a size the compiler knows.** For a `Vec` or a `String` that size is three words, and it says nothing about how much data there is.
- **The stack is fast because it's simple**: a frame goes on when a function is called, comes off when it returns, and its size was decided at compile time.
- **The heap is flexible because it isn't simple**: a block is asked for at run time and has to be given back, exactly once.
- **"Given back exactly once" is the entire problem** that ownership exists to solve. [1.2.2](../../02-move-semantics/README.md) is where the solution starts.
