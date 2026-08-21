# Solution — 1.3.2 The rules of the borrow checker

```rust
pub fn duplicate_in_place(values: &mut Vec<i32>) {
    let original = values.len();
    for index in 0..original {
        let value = values[index];
        values.push(value);
    }
}

pub fn move_first_to_last(values: &mut Vec<i32>) {
    if values.len() < 2 {
        return;
    }
    let front = values.remove(0);
    values.push(front);
}

pub fn append_longest(lines: &mut Vec<String>) {
    if lines.is_empty() {
        return;
    }
    let mut best = 0;
    for index in 1..lines.len() {
        if lines[index].len() > lines[best].len() {
            best = index;
        }
    }
    let copy = lines[best].clone();
    lines.push(copy);
}

pub fn drop_duplicates(values: &mut Vec<i32>) {
    let mut kept: Vec<i32> = Vec::new();
    for value in values.iter() {
        if !kept.contains(value) {
            kept.push(*value);
        }
    }
    *values = kept;
}

pub fn apply_bonus(scores: &mut Vec<i32>, bonus: i32) -> i32 {
    if scores.is_empty() {
        return 0;
    }
    let mut total = 0;
    for score in scores.iter_mut() {
        *score += bonus;
        total += *score;
    }
    scores.push(total);
    total
}
```

Five functions, one `.clone()`. If you wrote more than that, somewhere you bought your way out of an error instead of restructuring.

## `duplicate_in_place` — the classic trap wearing an exercise costume

The natural first attempt is this:

```rust
for value in values.iter() {
    values.push(*value);
}
```

```text
error[E0502]: cannot borrow `*values` as mutable because it is also borrowed as immutable
 --> src\main.rs:3:9
  |
2 |     for value in values.iter() {
  |                  -------------
  |                  |
  |                  immutable borrow occurs here
  |                  immutable borrow later used here
3 |         values.push(*value);
  |         ^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

The same `E0502` as example 04, with a `Vec<i32>` instead of a `Vec<String>`. The danger is even more concrete here: if this were allowed, `push` could move the buffer while the loop is still reading out of the old one.

The answer is an index walk, with one detail that matters:

```rust
let original = values.len();
```

The length is read **before** the loop. Write `0..values.len()` instead and every `push` pushes the finish line further away, so the loop never ends — which is precisely Python's infinite loop, this time with the compiler's blessing. **The borrow checker prevents invalid memory, not wrong logic.** That one is still your job.

Third detail: `let value = values[index];` is deliberate. `i32` is `Copy`, so that is a number rather than a borrow, and the short borrow that produced it is over immediately.

Written as one line, `values.push(values[index]);` also compiles — the argument is evaluated before the mutable borrow activates. That special case is called a *two-phase borrow*, and [1.3.3](../../03-borrow-scopes-and-nll/README.md) is where it belongs. Two lines cost nothing and lean on no special case.

## `move_first_to_last` — take it out, then put it back

```rust
let front = values.remove(0);
values.push(front);
```

Two calls, and no borrow is alive between them. `remove` hands you ownership of the element and leaves the `Vec` one shorter; `push` puts that same value back.

The "let me just point at the element" version fails:

```rust
let front = &mut values[0];
values.push(*front);
```

```text
error[E0499]: cannot borrow `values` as mutable more than once at a time
 --> src\main.rs:4:5
  |
3 |     let front = &mut values[0];
  |                      ------ first mutable borrow occurs here
4 |     values.push(*front);
  |     ^^^^^^      ------ first borrow later used here
  |     |
  |     second mutable borrow occurs here
```

Three labels over two lines, and `first borrow later used here` sits exactly on `*front` — meaning "this use is what kept the first borrow alive". Copy it into a variable beforehand and the error is gone.

The `values.len() < 2` guard is load-bearing too: the specification says a shorter `Vec` is left untouched, and without the guard `remove(0)` panics on an empty one.

## `append_longest` — hold an index, not a reference

```rust
let mut best = 0;
for index in 1..lines.len() {
    if lines[index].len() > lines[best].len() {
        best = index;
    }
}
let copy = lines[best].clone();
lines.push(copy);
```

`best` is a `usize`. It borrows nothing, so whatever you do to `lines` afterwards is fine.

The version that makes `best` a `&String` happens to compile here, because its last use is inside the `push` call itself. But the day you add a line after the push that reads `best`, you get `E0502`. **The index removes the question entirely**, and it is what you want in real code.

This is the only function with a `.clone()` in it, and the specification is where the justification lives: an independent `String` has to end up in the `Vec`. The test `the_appended_line_owns_its_own_buffer` checks exactly that — if you had found a way to share the buffer, that assertion would have caught it (and sharing isn't even expressible without [Phase 2](../../../../phase2-intermediate/05-smart-pointers/02-rc-and-arc/README.md)).

"The earlier one wins" comes from `>` rather than `>=`. With `>=`, the last of the equally long strings would win and `the_earlier_of_two_equal_lines_wins` would go red.

## `drop_duplicates` — build, then replace

```rust
let mut kept: Vec<i32> = Vec::new();
for value in values.iter() {
    if !kept.contains(value) {
        kept.push(*value);
    }
}
*values = kept;
```

Removing while iterating is the same error as always:

```text
error[E0502]: cannot borrow `*values` as mutable because it is also borrowed as immutable
 --> src\main.rs:4:13
  |
2 |     for value in values.iter() {
  |                  -------------
  |                  |
  |                  immutable borrow occurs here
  |                  immutable borrow later used here
3 |         if *value == 3 {
4 |             values.remove(0);
  |             ^^^^^^^^^^^^^^^^ mutable borrow occurs here
```

So a fresh `Vec` gets built and installed at the end: `*values = kept;`. That `*` is the dereference from [1.3.1](../../01-shared-and-mutable-refs/README.md) — "assign to the thing this reference points at", not "repoint the reference". The old `Vec` is dropped and freed right there.

The `values.iter()` borrow lives until the loop ends and `*values = kept;` comes after it, so there is no conflict. Move that line inside the loop and you get `E0502`.

Cost: `contains` scans the whole of `kept` for every element, so this implementation is quadratic. For small lists that is exactly right; the faster answer is a `HashSet`, which arrives in [Phase 2](../../../../phase2-intermediate/01-collections/01-vec-and-hashmap/README.md).

## `apply_bonus` — one writing pass, then one push

```rust
for score in scores.iter_mut() {
    *score += bonus;
    total += *score;
}
scores.push(total);
```

`iter_mut` hands you a `&mut i32` at a time, and each one is finished with before the next exists. That is why "one writer" isn't violated even though you are writing over the whole `Vec`.

The `push` has to be **outside** the loop:

```text
error[E0499]: cannot borrow `scores` as mutable more than once at a time
 --> src\main.rs:7:9
  |
4 |     for score in scores.iter_mut() {
  |                  -----------------
  |                  |
  |                  first mutable borrow occurs here
  |                  first borrow later used here
...
7 |         scores.push(total);
  |         ^^^^^^ second mutable borrow occurs here
```

`iter_mut()` is itself a mutable borrow of the whole `Vec`, alive until the loop ends, and `push` wants a second one. This time the rule has also caught a real bug: pushing inside a loop that walks the same `Vec` is that infinite loop again.

The `is_empty` guard keeps the specification: an empty `Vec` gains nothing and the answer is `0`. Without it, empty would turn into `[0]` and `an_empty_score_list_gains_nothing` would go red.

## The pattern all five share

None of these fought the checker. Every one of them made one of the same few moves:

| Move | Where |
|---|---|
| copy the value out so the borrow ends | `duplicate_in_place`, `move_first_to_last` |
| hold an index instead of a reference | `append_longest` |
| build somewhere else and replace at the end | `drop_duplicates` |
| separate the reading pass from the writing | `apply_bonus` |

And in none of them was `.clone()` the escape hatch. The lesson's only clone is the one the specification asked for.
