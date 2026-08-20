# Solution — 06 Tooling

Every change below keeps behaviour identical. `cargo test` stays green throughout; only the style moves.

| Was | Became | Lint |
|---|---|---|
| `name: &String` | `name: &str` | `ptr_arg` |
| `name.len() == 0` | `name.is_empty()` | `len_zero` |
| `return x * 2;` | `x * 2` | `needless_return` |
| `if flag == true { true } else { false }` | `flag` | `bool_comparison`, `needless_bool` |
| `let counted = ...; counted` | return it directly | `let_and_return` |
| `for i in 0..nums.len() { total += nums[i] }` | `nums.iter().sum()` | `needless_range_loop` |
| `match opt { Some(x) => x, None => 0 }` | `opt.unwrap_or_default()` | `manual_unwrap_or_default` |

## The two worth dwelling on

### `&String` → `&str` is not cosmetic

```rust
pub fn is_empty_name(name: &str) -> bool
```

This is the only change that alters what callers can do — and it *widens* what they can do, so nothing breaks. Before the change, this was a compile error:

```rust
is_empty_name("");                       // a &str literal, not a &String
is_empty_name(&some_string[0..0]);       // a slice, not a &String
```

After it, all of them work, and none of them allocate. The lint is about your API's shape, not about micro-optimisation.

Anything that is a `&String` can become a `&str` for free — Rust does that conversion automatically at call sites. The reverse needs an allocation. So `&str` is strictly the more useful parameter type, always.

### `for i in 0..nums.len()` → `.iter().sum()`

Two separate improvements folded together:

1. **Iterating instead of indexing.** `nums[i]` performs a bounds check every time round. `nums.iter()` can't go out of bounds by construction, so there's nothing to check. Safer *and* faster — the rare case where those agree.
2. **`sum()` instead of an accumulator.** Says what it means in one word.

If you wrote `nums.iter().fold(0, |acc, x| acc + x)`, that's also correct, and it's exactly what `sum()` does underneath. Prefer the named one.

## On `cargo clippy --fix`

It would have applied five of the seven automatically. Using it in real work is fine.

Using it *here* would have skipped the point. The exercise wasn't the seven edits — it was reading each warning and being able to say why it's right. That skill is what makes clippy a tutor rather than a nag, and it only develops if you read.

## When to disagree with clippy

Real answer: rarely, in your first year. Almost every warning you get is right, and the ones that aren't tend to be `pedantic` lints you opted into yourself.

But the mechanism matters when you do:

```rust
// The lesson keeps this loop un-idiomatic on purpose: showing what clippy
// dislikes is the whole point of the example.
#[allow(clippy::needless_range_loop)]
fn sum_verbose(nums: &[i32]) -> i32 {
```

The comment is not decoration. Six months later it's the only thing that tells the next reader — likely you — whether the `allow` is still earning its place.
