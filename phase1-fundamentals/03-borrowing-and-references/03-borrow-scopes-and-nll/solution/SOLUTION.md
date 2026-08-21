# Solution — 1.3.3 Borrow scopes and NLL

```rust
pub fn with_first_repeated(values: Vec<i32>) -> Vec<i32> {
    let mut values = values;
    if values.is_empty() {
        return values;
    }
    values.push(values[0]);
    values
}

pub fn with_total_appended(values: Vec<i32>) -> Vec<i32> {
    let mut values = values;
    let mut total = 0;
    for value in &values {
        total += value;
    }
    values.push(total);
    values
}

pub fn with_longest_repeated(names: Vec<String>) -> Vec<String> {
    let mut names = names;
    if names.is_empty() {
        return names;
    }
    let mut longest = 0;
    for index in 1..names.len() {
        if names[index].len() > names[longest].len() {
            longest = index;
        }
    }
    names.push(names[longest].clone());
    names
}

pub fn with_length_appended(text: String) -> String {
    let mut text = text;
    text.push_str(&text.len().to_string());
    text
}

pub fn doubled_then_totalled(values: Vec<i32>) -> Vec<i32> {
    let mut values = values;
    for value in &mut values {
        *value *= 2;
    }
    let mut total = 0;
    for value in &values {
        total += value;
    }
    values.push(total);
    values
}
```

There isn't one extra brace anywhere in there. If you wrote some, they're not wrong — but not one of them was needed, and that is the whole point of the lesson.

## `with_first_repeated` — one line, thanks to a two-phase borrow

```rust
values.push(values[0]);
```

That line is `Vec::push(&mut values, values[0])`: a mutable reference and a read of the same vector, in one statement. It compiles because the compiler made that `&mut`, so it starts out reserved and behaves like a shared reference right up to the call itself.

The two-line version is entirely correct too, and needs no favours from anyone:

```rust
let first = values[0];
values.push(first);
```

`i32` is `Copy`, so `first` is a copy rather than a reference and there is no borrow in play at all. Write this one if you're newer to Rust; what you need to know is **why** the one-liner is allowed as well.

The `is_empty` guard is needed because `values[0]` panics on an empty vector. That isn't a borrow error, it's an index error — two different things.

## `with_total_appended` — the loop's borrow ends with the loop

```rust
for value in &values {
    total += value;
}
values.push(total);
```

`&values` is a shared reference that stays alive to the last turn of the loop. `values.push(total)` wants a `&mut`. Those two don't go together — but they aren't together: the shared reference's last use is inside the loop, so its scope ends at the loop's closing brace, and the `push` comes after that.

No extra block around the `for` is needed. If you put one, the output is the same and nothing changed — exactly what `examples/03-ending-a-borrow-early.rs` was showing.

Now try this: move the `push` *inside* the loop. The reference is then still needed on the next turn, so you get `E0502` — and this time the third label lands on the `for` line itself, because that is where the reference gets used again.

## `with_longest_repeated` — hold an index, not a reference

This is the one that turns down your first instinct. The instinct is:

```rust
let longest = &names[0];      // a &String
names.push(longest.clone());
```

And the interesting part is that this compiles too — two-phase borrows again, because `longest.clone()` is that reference's last use and it finishes before the `&mut` activates. But add one more line that uses `longest` after the `push` and you get `E0502` immediately.

The solution version never takes on that risk:

```rust
let mut longest = 0;
for index in 1..names.len() {
    if names[index].len() > names[longest].len() {
        longest = index;
    }
}
```

`longest` is a `usize`. A number. It has borrowed nothing, so nothing can hold it up. The loop borrows `names` for reading and finishes with it on the same line each time.

Then:

```rust
names.push(names[longest].clone());
```

The `.clone()` is required, and taking it out gives you an error that is **not** a borrow error:

```text
error[E0507]: cannot move out of index of `Vec<String>`
```

That `E0507` is from [1.2.2](../../../02-ownership-and-memory/02-move-semantics/README.md): you can't pull a value out of the middle of a vector and leave the vector intact. Telling an ownership error from a borrow error at a glance is a skill of its own.

On ties: `>` rather than `>=`, so the first long name stays the winner. That's what the specification asked for.

## `with_length_appended` — the same shape, on a `String`

```rust
text.push_str(&text.len().to_string());
```

Again this is `String::push_str(&mut text, ...)` with an argument that reads `text`. Again reserved, again allowed.

And the length written on the end is the length *before* anything was added, because the argument is evaluated before the `&mut` activates. The specification said so explicitly; had it not, this code would have had two defensible answers, and that would be a defect in the specification rather than a challenge.

## `doubled_then_totalled` — three borrows, end to end

```rust
for value in &mut values {
    *value *= 2;
}
let mut total = 0;
for value in &values {
    total += value;
}
values.push(total);
```

Three different borrows of one vector, in one function:

| Borrow | From where to where |
|---|---|
| `&mut values` | inside the first loop |
| `&values` | inside the second loop |
| `&mut values` (`push`'s automatic one) | that one line only |

None of them collide, because no point exists that is inside two of them. This is the complete picture of what NLL gives you: under the lexical model all three lived to the end of the function and this function would not have compiled.

`*value *= 2` needs the `*` because `value` is a `&mut i32` and you have to go through the arrow to reach the number — the same `*` as in [1.3.1](../../../03-borrowing-and-references/01-shared-and-mutable-refs/README.md).

## What this lesson was really about

- **A borrow's scope runs from its creation to its last use**, not to the closing brace.
- **The error's third label names the ending line.** Read it first, then change the code.
- **"Later" means later in execution.** A loop can make a line above be the next use.
- **Braces are the last tool, not the first.** Look at moving the use, or not borrowing at all, before you reach for them.
- **Two-phase borrows apply only to a method's automatic `&mut`.** That's why `v.push(v.len())` compiles and its named version doesn't.
- And when you reach [slices](../../04-slices/README.md), all of this comes back — a slice is a borrow too, just of part of a collection.
