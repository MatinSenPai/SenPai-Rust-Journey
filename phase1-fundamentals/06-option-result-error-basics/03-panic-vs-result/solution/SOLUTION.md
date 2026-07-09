# Solution

```rust
pub fn required_config(vars: &HashMap<String, String>, key: &str) -> String {
    vars.get(key)
        .cloned()
        .unwrap_or_else(|| panic!("missing required config key: {key}"))
}
```
`.unwrap_or_else(|| panic!(...))` rather than a bare `.unwrap()`: the
closure only runs (and only formats the message) if the key really is
missing, and the message names exactly which key — far more useful at 3am
than a generic "called `Option::unwrap()` on a `None` value."

```rust
pub fn average_of_nonempty(nums: &[f64]) -> f64 {
    assert!(!nums.is_empty(), "average_of_nonempty called with an empty slice — caller bug");
    let sum: f64 = nums.iter().sum();
    sum / nums.len() as f64
}
```
`assert!` is `panic!`'s cousin for checking a boolean condition — reads
more clearly than an `if !cond { panic!(...) }` for a guard check like
this one. The message explicitly says "caller bug," which is the whole
point of this function's design: an empty slice reaching this function
means something upstream broke a promise, not that a user typed something
odd.

Both `required_config` and `average_of_nonempty` panic, but for genuinely
different reasons that happen to look similar in code: `required_config`'s
failure is about the **environment** the program is deployed into (still
arguably "external," but not something any *caller* inside the running
program can fix at runtime — the only fix is redeploying with the right
config) — while `average_of_nonempty`'s failure is about **another part of
your own program** violating a documented contract. Both are legitimate
"stop, something assumed-impossible happened" situations — contrast this
with `parse_user_age`, where invalid input is not a bug anywhere, it's the
ordinary, expected, majority-of-the-time behavior of talking to the
outside world, which is exactly why it returns `Result` instead.
