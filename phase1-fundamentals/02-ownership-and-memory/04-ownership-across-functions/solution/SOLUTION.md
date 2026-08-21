# Solution — 1.2.4 Ownership across functions

```rust
pub fn appended(mut base: String, extra: String) -> String {
    base.push_str(&extra);
    base
}

pub fn longer_of(left: String, right: String) -> String {
    if right.len() > left.len() {
        right
    } else {
        left
    }
}

pub fn measure_and_return(text: String) -> (usize, String) {
    let length = text.len();
    (length, text)
}

pub fn length_of(text: &str) -> usize {
    text.len()
}

pub fn split_off_first(mut values: Vec<i32>) -> (i32, Vec<i32>) {
    let first = values.remove(0);
    (first, values)
}
```

## `appended` — reusing the buffer you were given

```rust
base.push_str(&extra);
base
```

`base` came in owned, so it is yours: you may grow it, and whatever spare capacity it already had gets used before anything is allocated. Then it goes back out, moved, with no copy.

The version that looks equivalent and is not:

```rust
let mut out = String::new();
out.push_str(&base);
out.push_str(&extra);
out
```

That throws away a perfectly good buffer and builds a new one. Same answer, one extra allocation and one extra copy of `base`. **When a function is handed an owned value, growing that value is nearly always cheaper than building a fresh one.**

`&extra` rather than `extra` because `push_str` only needs to read the bytes it is copying in. `extra` is dropped at the end of the function either way.

## `longer_of` — the argument you do not return

```rust
if right.len() > left.len() {
    right
} else {
    left
}
```

Both arguments are owned by this function. Exactly one of them leaves; the other reaches the closing brace still owned, and is dropped there. That is not a leak and not a waste — it is the language doing precisely what [1.2.1](../../01-stack-and-heap/README.md) described, at the moment it described.

Notice this compiles without a `.clone()` anywhere. In a language where the caller might still hold both strings, returning one of them would be a shared reference and dropping the other would be someone else's problem. Here the ownership question has one answer and the compiler already knows it.

The `>` rather than `>=` is the specification: `longer_of("ab", "cd")` must give `"ab"`, so a tie keeps the left one. And the test with `"a"` against `"س"` is there to keep you honest — `.len()` is bytes, so one Persian letter outweighs one Latin one. If you reached for `.chars().count()` you would fail that assertion, and the doc comment did say bytes.

## `measure_and_return` versus `length_of` — the whole lesson, side by side

```rust
pub fn measure_and_return(text: String) -> (usize, String) {
    let length = text.len();
    (length, text)
}

pub fn length_of(text: &str) -> usize {
    text.len()
}
```

Two functions doing the same job. Look at what the tests have to do differently.

```rust
let (length, text) = measure_and_return("hello".to_string());
```

The caller destructures a tuple, and has to rebind `text` to keep using it. Call it twice and you write that dance twice.

```rust
let owned = String::from("hello");
assert_eq!(length_of(&owned), 5);
assert_eq!(length_of(&owned), 5);
assert_eq!(owned, "hello");
```

Called twice with no ceremony, and `owned` is still `owned` at the end. **And it takes a literal directly** — `length_of("سلام")` needs no `String` to be built at all, so there is no allocation anywhere in that call.

`measure_and_return` is not bad code. It is what you would have to write if borrowing did not exist, and writing it once is the point: [module 1.3](../../../03-borrowing-and-references/README.md) opens with the problem this function is a workaround for.

Note also what `length_of` did *not* need: no `mut`, no return of the argument, no tuple. A signature that asks for less is easier to call, easier to read, and harder to get wrong.

## `split_off_first` — `mut` on a parameter, again

```rust
pub fn split_off_first(mut values: Vec<i32>) -> (i32, Vec<i32>) {
    let first = values.remove(0);
    (first, values)
}
```

`values[0]` would not compile for a `Vec<String>` — that is `E0507` from [1.2.2](../../02-move-semantics/README.md). It happens to compile here because `i32` is `Copy`, and it would be wrong anyway: the spec asks for the first element **and the rest**, so the element has to actually come out.

`remove(0)` does both jobs: it hands you the element and closes the gap. And `mut` is required because it modifies the `Vec` — which you are entitled to do, because the `Vec` is yours now. The caller cannot tell; `mut` on a parameter is not part of the signature.

`remove(0)` shifts every remaining element down one, which is `O(n)`. For a three-element test that is nothing. For a hot loop over a large `Vec` it is the wrong tool, and the right ones are `swap_remove` when order does not matter or `VecDeque` when it does.

## What this lesson was really about

- **Passing a value to a function is a move**, with exactly the same rules as an assignment. There is no separate "argument passing" concept to learn.
- **A function's signature is a demand.** `String` says "give this up"; `&str` says "let me look". Ask for the smallest thing that does the job.
- **Returning gives ownership away**, which is where new values come from and why `build() -> String` needs no ceremony.
- **A value that comes in and does not go out is dropped at the closing brace**, and that is the whole cleanup story.
- **The "give it back" tuple pattern works and nobody writes it.** Having written it once, you will recognise what borrowing is for the moment it arrives.
