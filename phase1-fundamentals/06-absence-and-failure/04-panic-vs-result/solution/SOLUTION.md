# Solution — 1.6.4 Panic versus `Result`

```rust
pub fn parse_priority(input: &str) -> Result<u8, String> {
    let value: u8 = match input.trim().parse() {
        Ok(value) => value,
        Err(_) => return Err(format!("'{input}' is not a whole number")),
    };
    if !(1..=5).contains(&value) {
        return Err(format!("priority must be between 1 and 5, got {value}"));
    }
    Ok(value)
}

pub fn priority_label(level: u8) -> &'static str {
    match level {
        1 | 2 => "low",
        3 => "normal",
        4 | 5 => "high",
        other => unreachable!(
            "priority_label: level {other} was never validated by parse_priority (must be 1..=5)"
        ),
    }
}

pub fn checked_midpoint(sorted_ascending: &[i32]) -> i32 {
    assert!(
        !sorted_ascending.is_empty(),
        "checked_midpoint: caller must not pass an empty slice"
    );
    sorted_ascending[sorted_ascending.len() / 2]
}

pub fn last_digit_of(values: &[u32]) -> u32 {
    let last = values
        .last()
        .expect("last_digit_of: caller must not pass an empty values slice");
    last % 10
}
```

Two of these four never panic. The other two must. The difference isn't in the code — it's in where the input came from.

## `parse_priority` — input you never trust

```rust
let value: u8 = match input.trim().parse() {
    Ok(value) => value,
    Err(_) => return Err(format!("'{input}' is not a whole number")),
};
if !(1..=5).contains(&value) {
    return Err(format!("priority must be between 1 and 5, got {value}"));
}
Ok(value)
```

Two separate ways to fail, two separate messages. `.parse()` on something like `"abc"` gives back an `Err` that has nothing to do with what the number should have been — so instead of showing that internal error, this builds its own message naming `input` itself (the untrimmed original, not the trimmed copy). The second failure only happens once parsing already succeeded, so at that point `value` exists and can go straight into the message.

Notice that nothing in this function panics. Even when `input` is complete nonsense, the function ends not with a crash but with an `Err` — because this input came from outside the program, and being invalid is, by definition, ordinary.

## `priority_label` — the same value, a completely different amount of trust

```rust
match level {
    1 | 2 => "low",
    3 => "normal",
    4 | 5 => "high",
    other => unreachable!(
        "priority_label: level {other} was never validated by parse_priority (must be 1..=5)"
    ),
}
```

`level` is also a `u8`, exactly like what came out of `parse_priority` — but it's no longer raw input here; the function's contract says it's only ever called with what `parse_priority` already validated. `match` still has to cover every possible `u8` (that's the language's own rule, from 1.5.4), but the `other` arm is never actually supposed to run. `unreachable!()` says exactly that, and names the broken assumption too — not just "this shouldn't happen," but what guaranteed it and where that guarantee was bypassed.

## `checked_midpoint` — an `assert!` guarding an internal rule

```rust
assert!(
    !sorted_ascending.is_empty(),
    "checked_midpoint: caller must not pass an empty slice"
);
sorted_ascending[sorted_ascending.len() / 2]
```

`sorted_ascending` was never typed by a user; some other part of this same program built it and handed it over. If it arrives empty, that isn't a sign of "bad input" — it's a sign that some other part of this codebase already broke its word. The `assert!` message says exactly that: which function it was, and what it expected from its caller.

## `last_digit_of` — an `.expect()` that names the assumption, not the failure

```rust
let last = values
    .last()
    .expect("last_digit_of: caller must not pass an empty values slice");
last % 10
```

`values.last()` gives back an `Option<&u32>`. A bare `.unwrap()` would have worked here too — but its message would have been "called `Option::unwrap()` on a `None` value," the exact same thing `Option` itself already told you for free. This `.expect()` message says instead *why* we believed `values` would never be empty: because that's this function's own contract. Whoever sees this line in a log doesn't have to guess which `None` it was — the function's name and its assumption are right there.

## What these four functions were really about

- **The rule was never about the code, it's about where the value came from.** `parse_priority` and `priority_label` both take a `u8`; one returns `Result`, the other panics — because one arrives from a person and the other from inside this same program.
- **`assert!`/`unreachable!()`/`.expect()` are one family: "I'm asserting this cannot happen."** They only differ in the shape of what they check — a boolean condition, a `match` arm, or an `Option`.
- **A good message always names the broken promise, not the symptom.** Everywhere these four functions panic, the message names the function itself and its contract — never just "this was empty."
- **None of these four functions panics on a user's input.** Only `parse_priority` touches raw external text directly, and it's the one that never panics at all.
