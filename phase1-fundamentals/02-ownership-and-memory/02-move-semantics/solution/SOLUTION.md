# Solution

```rust
pub fn total_length(strings: Vec<String>) -> usize {
    let mut total = 0;
    for s in strings {
        total += s.len();
    }
    total
}
```

The interesting line is `for s in strings`. `strings` is `Vec<String>`
(owned, not borrowed), and `for x in some_vec` (not `&some_vec`) calls
`IntoIterator::into_iter` by value — it **consumes** the vec, handing you
each owned `String` in turn as `s`. That's exactly the "total_length takes
full ownership" contract the function signature promises: by the time this
loop finishes, every `String` that was in `strings` has either been counted
and then dropped (at the end of each loop iteration, once `s` goes out of
scope), and `strings` itself is gone too.

On recall question 1: uncommenting `moved_value_demo` and running
`cargo check` gives roughly:

```
error[E0382]: borrow of moved value: `s`
 --> src/lib.rs:XX
  |
  | let s = String::from("hello");
  | - move occurs because `s` has type `String`, which does not implement the `Copy` trait
  | let s2 = s;
  |          - value moved here
  | println!("{s}");
  |           ^ value borrowed here after move
```

`E0382` specifically means "you tried to use a value after it moved."
The error message even tells you *why* it moved (`String` isn't `Copy`) —
that's the subject of the very next lesson.
