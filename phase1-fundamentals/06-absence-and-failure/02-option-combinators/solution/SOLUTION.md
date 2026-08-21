# Solution — 1.6.2 `Option` combinators

```rust
pub fn shout(word: Option<&str>) -> Option<String> {
    word.map(|w| format!("{}!", w.to_uppercase()))
}

pub fn safe_half(n: Option<i32>) -> Option<i32> {
    n.and_then(|x| if x % 2 == 0 { Some(x / 2) } else { None })
}

pub fn positive_only(n: Option<i32>) -> Option<i32> {
    n.filter(|v| *v > 0)
}

pub fn take_and_reset(slot: &mut Option<String>) -> Option<String> {
    slot.take()
}

pub fn coordinates(x: Option<i32>, y: Option<i32>) -> Option<(i32, i32)> {
    x.zip(y)
}

pub fn first_available(primary: Option<i32>, backup: Option<i32>) -> Option<i32> {
    primary.or(backup)
}
```

Six functions, six different combinators. None of them wanted a `match`.

## `shout` — a plain `.map()`

```rust
word.map(|w| format!("{}!", w.to_uppercase()))
```

`word` is either `Some(w)` or `None`. If it's `Some`, the closure runs on it and the result goes back into `Some`; if it's `None`, the closure never runs and `None` comes straight back. Exactly the definition of `.map()` from the lesson — no branch of your own to write.

Writing it with `match` would work too:

```rust
match word {
    Some(w) => Some(format!("{}!", w.to_uppercase())),
    None => None,
}
```

Three more lines for the same result, and you have to hand-write the "get `Some`, transform, wrap back in `Some`" pattern every time. That repetition is exactly what `.map()` exists to remove.

## `safe_half` — why `.and_then()` and not `.map()`

```rust
n.and_then(|x| if x % 2 == 0 { Some(x / 2) } else { None })
```

The difference from `shout` is that the closure itself returns an `Option<i32>`, not an `i32`. If you'd written `.map(...)` here instead, the compiler would have built an `Option<Option<i32>>`, and it wouldn't have matched the declared signature — `Option<i32>`. That's the same `E0308` you met in the lesson's errors section.

The rule was the same one from "The concept": look at what the closure returns. Here it's the closure itself (not a separate function) that returns either `Some(x / 2)` or `None` — so the same rule applies: the closure returns an `Option`, so `.and_then()`.

## `positive_only` — `.filter()` and that reference

```rust
n.filter(|v| *v > 0)
```

`v` here has type `&i32`, not `i32` — that's why we wrote `*v > 0` instead of `v > 0`. Drop the `*` and the compiler says it can't compare `&i32` with `i32`; exactly what "The concept" explained: `.filter()` has to be able to hand the value back untouched inside `None` if the predicate rejects it, so it only lets you look.

The tests check `Some(0)` too: since the condition is `*v > 0`, not `*v >= 0`, zero gets rejected. Get the operator wrong and that specific test — `positive_only(Some(0))` — is the one that fails.

## `take_and_reset` — `.take()` on a `&mut Option`

```rust
slot.take()
```

The function's signature takes `&mut Option<String>`, not `Option<String>` — because `.take()` acts on the variable itself, not a copy. `.take()` pulls out whatever `slot` holds and puts `None` in its place, with no cloning at all: just an ownership move, the same thing you saw in 1.2.2.

Writing `slot.clone()` would have passed the first assertion too, but the second one — `assert_eq!(slot, None)` — would fail, because cloning leaves `slot` untouched.

## `coordinates` — `.zip()`

```rust
x.zip(y)
```

`.zip()` pairs two `Option`s into a tuple, only if both are `Some`. There's no nesting risk here because neither `x` nor `y` is a function that itself returns an `Option` — they're just two plain values being placed side by side.

## `first_available` — `.or()`

```rust
primary.or(backup)
```

If `primary` was already `Some`, `backup` was never needed. This differs from `.unwrap_or()`: there, a *value* substitutes; here, a whole *`Option`* substitutes — which is why `first_available(None, None)` is still `None`, not some default number.

## What this lesson was really about

- **`.map()` wraps the closure's result back in `Some`; `.and_then()` doesn't**, because its closure already wrapped it. The question to always ask: "does the closure I'm passing already return an `Option`?"
- **`.filter()` takes a `&T`, not a `T`** — it has to be able to hand the value back untouched if the predicate rejects it.
- **`.take()` and its relatives act on the variable itself**, not a copy — that's why their signature is `&mut Option<T>`.
- **`.zip()` can never nest**, because neither side is a function — both are values.
- **`.or()` substitutes a whole `Option`; `.unwrap_or()` substitutes a value.** Don't mix them up.

If you want to see what these same six functions would look like as `match`, go back to the "`match` or chain?" section in the [README](../README.md) — both versions sit side by side there.
