# Solution

```rust
pub fn increment_n_times(times: u32) -> u32 {
    let mut count = 0;
    for _ in 0..times {
        count += 1;
    }
    count
}

pub fn parse_and_double(input: &str) -> i32 {
    let input: i32 = input.parse().expect("input should be a valid integer");
    let input = input * 2;
    input
}
```

`increment_n_times` genuinely mutates the same `count` binding across
iterations — this is the textbook case for `mut`: one logical value, updated
over time.

`parse_and_double` reuses the name `input` three times, each with a
different type/value: the original `&str` parameter, then an `i32` after
`.parse()`, then the doubled `i32`. This is exactly what shadowing is for:
a short pipeline of transformations where inventing `raw_input`,
`parsed_input`, `doubled_input` would just be extra names for values you
never need side-by-side.

Note `input.parse().expect(...)`: `.parse()` returns a `Result` (Phase 1's
last module covers this properly) because parsing can fail — `.expect(msg)`
says "if this failed, crash with this message," which is fine for a lesson
exercise but not how you'd handle untrusted input in real code (you'd
propagate the error instead — see `06-option-result-error-basics`).

One honest wrinkle: if you run `cargo clippy` on `parse_and_double`, it
flags `let input = input * 2; input` with `let_and_return`, suggesting you
collapse it to a bare tail expression `input * 2`. Clippy is *right* that
it's shorter — and in real code you'd very likely take that suggestion.
This solution keeps the explicit final shadow anyway (with
`#[allow(clippy::let_and_return)]`) purely because the exercise is about
practicing shadowing itself; it's a good early example that "idiomatic" and
"what a specific lesson is trying to demonstrate" aren't always the exact
same piece of code, and that silencing a lint with `#[allow(...)]` should
always come with a reason written down next to it, like the comment above
the function.
