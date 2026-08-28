# Solution — 1.3.1 Shared and mutable references

```rust
pub fn longest_length(lines: &Vec<String>) -> usize {
    let mut longest = 0;
    for line in lines {
        if line.len() > longest {
            longest = line.len();
        }
    }
    longest
}

pub fn count_above(values: &Vec<i32>, limit: &i32) -> usize {
    let mut found = 0;
    for value in values {
        if *value > *limit {
            found += 1;
        }
    }
    found
}

pub fn append_all(target: &mut String, extras: &Vec<String>) {
    for extra in extras {
        target.push_str(extra);
    }
}

pub fn clamp_all(values: &mut Vec<i32>, ceiling: &i32) {
    for value in values {
        if *value > *ceiling {
            *value = *ceiling;
        }
    }
}

pub fn transfer(from: &mut i32, to: &mut i32, amount: &i32) {
    *from -= *amount;
    *to += *amount;
}
```

Five functions, five signatures, and not one `.clone()` between them. Two of them only look and three of them write, and you can tell which is which without reading a single line of the bodies — that is what a reference in a signature buys you.

## `longest_length` — the one with no stars at all

```rust
for line in lines {
    if line.len() > longest {
        longest = line.len();
    }
}
```

`lines` is a `&Vec<String>`, so each `line` is a `&String`. And yet there is no `*` anywhere: `line.len()` is a method call, and the method dot dereferences for you. That is auto-deref doing its job, and it is why real Rust code has so few stars in it.

`longest` is a plain `usize` — a number you own, living on this function's stack frame. Nothing here borrows it and nothing needs to.

The empty case falls out for free. `longest` starts at `0`, the loop never runs, and `0` is the answer the specification asked for. No guard clause needed, which is worth noticing: a starting value chosen well often removes a special case.

## `count_above` — where the stars come back

```rust
if *value > *limit {
    found += 1;
}
```

Both `value` and `limit` are `&i32`. Writing `*value > *limit` compares the two numbers.

Here is the thing that trips people up: `value > limit` also compiles, and gives the same answer, because the standard library implements comparison between references too. So does `value > &5`. Three spellings, one result.

Which should you write? `*value > *limit`, at least while you're learning. It says out loud what is being compared, and the day the types stop being `Copy` numbers it is the version that keeps working.

`limit` being a `&i32` is a slightly artificial signature — in real code you would take a plain `i32`, since copying four bytes is cheaper than following an arrow to them. It is written this way here so that a shared reference to a `Copy` type shows up at least once. That distinction gets its proper treatment when you meet generic code in Phase 2.

The test checks something beyond the count:

```rust
assert_eq!(count_above(&values, &limit), 2);
assert_eq!(values.len(), 3);
assert_eq!(limit, -1);
```

Both arguments survive the call intact. With `&`, they always would — and that is precisely the thing the last lesson had to fake with a tuple.

## `append_all` — one `&mut` and one `&` in the same signature

```rust
for extra in extras {
    target.push_str(extra);
}
```

Two references with different jobs, right next to each other. `target` is borrowed mutably because it is being changed; `extras` is borrowed shared because it is only being read. The signature is the documentation.

`push_str` wants a text slice and `extra` is a `&String`, and it works anyway — Rust converts one to the other at the call site automatically. That conversion has a name and a lesson of its own ([1.4.1](../../../04-text-and-strings/01-string-vs-str/README.md)); for now, notice that it happened and that you didn't have to ask.

No return value. There is nothing to return: `target` is the caller's `String` and the caller can see it change. Compare that with what this function would have had to look like one lesson ago:

```rust
fn append_all(target: String, extras: Vec<String>) -> (String, Vec<String>)
```

That is the pattern this whole module exists to delete.

## `clamp_all` — writing through the loop variable

```rust
for value in values {
    if *value > *ceiling {
        *value = *ceiling;
    }
}
```

`values` is a `&mut Vec<i32>`, so `for value in values` hands you a `&mut i32` each turn. That single fact explains every star in the body: `value` is an arrow, so reading it needs `*value` and writing to it needs `*value = ...`.

Drop the star on the last line and you get the `E0308` from the lesson's error section, in a different costume:

```text
expected `&mut i32`, found `i32`
```

Same mistake, same fix. Assignment never guesses.

`values.len()` is untouched — clamping changes values, not how many there are. The test asserts it by comparing the whole `Vec`, which catches both "wrong numbers" and "wrong length" at once.

## `transfer` — three references, two of them mutable

```rust
*from -= *amount;
*to += *amount;
```

Two `&mut i32` alive at once, and the compiler doesn't object, because they point at two different variables. Try calling it as `transfer(&mut a, &mut a, &1)` and it *will* object — that is `E0499`, and it is [1.3.2](../../02-borrow-checker-rules/README.md).

Every one of those stars is compulsory. Drop the first and you get a third error code for your collection:

```text
error[E0368]: binary assignment operation `-=` cannot be applied to type `&mut i32`
help: `-=` can be used on `i32` if you dereference the left-hand side
```

Different code, same lesson: an arrow is not the thing it points at.

The specification deliberately refuses to be clever. Nothing is checked, a negative `amount` moves the other way, and `from` may go below zero. That is not laziness — it is so that the doc comment fully determines the behaviour, and so that no test can surprise you with a rule you were never told.

## What this lesson was really about

- **A reference is an arrow to somebody else's value.** It owns nothing, it copies nothing, and when it ends nothing is freed.
- **`&T` reads, `&mut T` writes, `T` takes.** Choose the weakest one that does the job, and the signature becomes documentation nobody can let go stale.
- **`*` is for assignment; the method dot handles the rest.** Almost every star you write will be on the left of an `=` or inside a loop over `&mut`.
- **`&mut` needs the owner to be `mut`.** You cannot lend permission you do not hold, and `E0596` says so in one line.
- **The give-it-back tuple is gone.** From here on, a function returning a value only to hand it back is a missing `&`.

Two questions this lesson deliberately left open: what happens when a shared and a mutable borrow of the *same* value overlap, and exactly when a borrow stops counting as alive. That's [1.3.2](../../02-borrow-checker-rules/README.md) and [1.3.3](../../03-borrow-scopes-and-nll/README.md), in that order.
