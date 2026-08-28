# Solution

`wrap` and `unwrap_box` are the two directions of the same move — onto the heap, then back off it:

```rust
pub fn wrap(value: i32) -> Box<i32> {
    Box::new(value)
}

pub fn unwrap_box(boxed: Box<String>) -> String {
    *boxed
}
```

`*boxed` here isn't reading through a borrow — `unwrap_box` owns `boxed` outright, so `*boxed` moves the `String` out of the heap allocation and hands it back, consuming the box in the process. That's the same move the `E0507` example in the lesson body shows you *can't* do through a shared reference: owning the `Box` is exactly what makes it legal.

`increment_boxed` needs two dereferences, not one:

```rust
pub fn increment_boxed(boxed: &mut Box<i32>) -> i32 {
    **boxed += 1;
    **boxed
}
```

`boxed` is `&mut Box<i32>` — a mutable reference to the box, not to the `i32` directly. The first `*` follows that reference to the `Box<i32>` itself; the second follows the `Box` to the `i32` it owns. Miss one and you're either incrementing the wrong thing or the types don't line up at all.

The size measurement is the heart of today's lesson, and it's the shortest function in the file:

```rust
pub fn box_size_report() -> (usize, usize, usize) {
    (
        size_of::<Box<i32>>(),
        size_of::<Box<[u8; 4096]>>(),
        size_of::<Box<Box<i32>>>(),
    )
}
```

Three wildly different `T`s — four bytes, four thousand ninety-six bytes, and a whole other `Box` — and three identical answers. Nothing about the code has to branch or special-case any of them; `Box<T>`'s size simply doesn't read `T`'s size at all when `T` is `Sized`.

`make_playable_list` is ordinary trait-object construction, the same shape [2.3.7](../../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.md) already taught — this lesson's only job was to explain why `Box<dyn Playable>` can hold a `Song` and a `Podcast` side by side at all:

```rust
pub fn make_playable_list() -> Vec<Box<dyn Playable>> {
    vec![
        Box::new(Song {
            title: "Intro".to_string(),
        }),
        Box::new(Podcast {
            title: "Deep Dive".to_string(),
            duration_minutes: 42,
        }),
    ]
}
```

`Song` and `Podcast` are different sizes on the stack, but every element of this `Vec` is the same `Box<dyn Playable>` — one fat pointer, sixteen bytes, no matter which struct is on the other end of it.
