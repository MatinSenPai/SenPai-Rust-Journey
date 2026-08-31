# 2.10.3 — Cargo features and conditional compilation

## At a glance

After this lesson you can:

- Say what happens to code behind a disabled `#[cfg(feature = "...")]` in the final binary — does it run slower, get replaced by a stub, or not exist at all — and contrast that with what `cfg!(feature = "...")` actually does.
- Wire up an optional dependency: `optional = true` on the dependency itself, `dep:name` inside a feature's list, and `default = []` so the base crate compiles with no extra dependencies.
- Say why Cargo compiles a shared dependency exactly once per build — even when different crates ask for different features of it — and use that to explain why a feature may only add capability, never remove or change it.

**Time:** ~50 minutes · **Prerequisites:** [2.7.2 — Unit, integration, and doc tests](../../07-project-structure-and-testing/02-unit-integration-doc-tests/README.md)

## Why this matters

Open the `Cargo.toml` of almost any real crate you've touched and it probably has a `[features]` table: `tokio` has close to 30 features, `serde` has `derive`, and this repo's own manifest turns on feature lists for half its dependencies. That's exactly what features are: compile-time, opt-in switches — an extra function, an extra trait impl, an extra dependency that only makes it into the binary if the consumer actually asked for it.

The closest Python has is `pip install requests[socks]` — an "extra" that installs an optional dependency. But the analogy breaks right there: that extra only changes what gets *installed*; the code path is still chosen at run time with something like `try: import socks` — both branches live in the `.py` file either way, whether `socks` is installed or not. In Cargo, a feature changes what gets *compiled*: code behind a disabled feature isn't slow-pathed, isn't stubbed — it simply isn't in the output. There's no Django equivalent either — `INSTALLED_APPS` toggles apps at process startup; features do their toggling before the compiler even starts.

## The concept

### Something you already know, with a different condition

[2.7.2](../../07-project-structure-and-testing/02-unit-integration-doc-tests/README.md) taught you what `#[cfg(test)]` means: "only compile this module when building for `cargo test`" — not slower, not stubbed, not compiled at all, so it isn't in your final binary either. This lesson shows you the exact same mechanism, just with a different condition: instead of "are we testing right now?", it's now "did the consumer ask for this?" — and you're the one naming the feature in `Cargo.toml`, not Cargo.

### Declaring a feature and an optional dependency

```toml
[features]
default = []
json-export = ["dep:serde", "dep:serde_json"]

[dependencies]
serde = { workspace = true, optional = true }
serde_json = { workspace = true, optional = true }
```

Three pieces. `optional = true` means "don't compile this dependency unless some feature turns it on." `dep:serde` inside a feature's list means "turning on `json-export` also turns on the `serde` dependency." And `default` is whatever feature set a consumer gets when they say nothing — ours is empty, so the base crate stays dependency-free. (A feature can also turn on *another* feature — of this same crate, or of a dependency — spelled `derive = ["serde/derive"]`-style; ours only needs `dep:`. You build this crate's own version of that in "Challenge," below.)

### `#[cfg(...)]` removes an item entirely

```rust
#[cfg(feature = "json-export")]
fn describe() -> &'static str {
    "compiled in"
}

#[cfg(not(feature = "json-export"))]
fn describe() -> &'static str {
    "compiled out"
}
```

In any one build, only one of these two `describe`s exists — not "dead and unreachable," but its tokens are removed from the compile tree before the compiler even checks types. Add `println!("{}", describe())` after these two definitions:

```text
describe(): compiled out
```

That's the default build's output (`examples/01-cfg-vs-cfg-bang.rs`, no features). With `--features json-export`:

```text
describe(): compiled in
```

### `cfg!(...)` is a run-time boolean, not a remover

The same file has one more line:

```rust
println!("cfg!(feature = \"json-export\"): {}", cfg!(feature = "json-export"));
```

```text
cfg!(feature = "json-export"): false
```

With `--features json-export`, this same line prints `true`. That's the difference from `#[cfg(...)]`: `cfg!(...)` never removes anything from the tree — it just drops in a `bool` literal, exactly where you wrote it. So if instead of the two `describe`s above you wrote one `describe` that branches with `if cfg!(feature = "json-export") { ... } else { ... }`, and the first branch called `to_json`, *both* branches — run or not — still have to compile in every build. With the feature off, `to_json` doesn't exist, so that branch fails to resolve — the exact error you'll meet below, in "Errors you will meet." Short version: `#[cfg(...)]` decides before compilation. `cfg!(...)` also decides before compilation — but only what a `bool` is worth, never what code exists.

One side benefit: typo the feature name — `examples/03-cfg-typo-warning.rs` prints `cfg!(feature = "made-up-feature")`, for a feature that was never declared in `Cargo.toml` — and the compiler doesn't quietly let it through. A standard lint (`unexpected_cfgs`) warns:

```text
warning: unexpected `cfg` condition value: `made-up-feature`
  --> phase2-intermediate\10-rust-toolbox\03-cargo-features\examples\03-cfg-typo-warning.rs:11:14
   |
11 |         cfg!(feature = "made-up-feature")
   |              ^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: expected values for `feature` are: `broken`, `default`, and `json-export`
   = help: consider adding `made-up-feature` as a feature in `Cargo.toml`
   = note: see <https://doc.rust-lang.org/nightly/rustc/check-cfg/cargo-specifics.html> for more information about checking conditional configuration
```

### `#[cfg_attr(...)]`: the conditional form of an attribute

```rust
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "json-export", derive(serde::Serialize))]
pub struct Report {
    pub label: String,
    pub count: usize,
    pub mean: f64,
    pub min: i64,
    pub max: i64,
}
```

`Report` itself exists in *both* builds — only which attributes are `derive`d on it changes. `#[cfg_attr(cond, attr)]` means "apply `attr` only when `cond` holds" — it makes the *attribute* conditional, not the item. Swap that line for a plain `#[cfg(feature = "json-export")]` on the whole `struct Report`, and the default build would have no `Report` at all — `build_report`, which is always available and returns `Option<Report>`, would stop compiling, its return type pointing at something that no longer exists.

### Features must be additive: unification

Imagine two other crates depending on this one:

```toml
# crate A
p2-10-03-cargo-features = { version = "0.1", features = ["json-export"] }

# crate B, in the same build
p2-10-03-cargo-features = { version = "0.1" }
```

Cargo compiles this crate **once** — not one copy for A and another for B — with the **union** of every feature anyone asked for: since A wants `json-export`, B gets a copy where `to_json` exists too, whether B asked for it or not. This is **unification**, and it forces one design rule onto whoever writes the feature: turning a feature on may only **add** something, never remove or change existing behavior. A feature like `no-std-panic-handler` that *removes* something breaks any dependent who never asked for it — because that dependent has no way to keep it off; it can only turn on features of its own, never turn off a feature some other crate already turned on.

```senpai-visual
{"kind":"concept","labels":["crate A asks for json-export","crate B asks for nothing","Cargo builds one shared copy","the copy gets the union of both requests","B gets to_json too, whether it asked or not"]}
```

The consequence: your crate has to build, and its tests have to pass, in **every** feature combination you claim to support — exactly what "Hands on" and "Exercises," below, ask of you.

### The same trick for tests

```rust
#[cfg(all(test, feature = "json-export"))]
mod json_tests {
```

`all(...)` ANDs together as many conditions as you like. This module only compiles when you're **both** running tests **and** `json-export` is on — so a plain `cargo test` never even tries to compile it, and the base crate never has to see `serde_json`.

## Hands on

```sh
cargo run -p p2-10-03-cargo-features --example 01-cfg-vs-cfg-bang
cargo run -p p2-10-03-cargo-features --example 01-cfg-vs-cfg-bang --features json-export
```

Now the broken one — just to see the error, not to fix it yet (that's "Repair"):

```sh
cargo run -p p2-10-03-cargo-features --example 02-to-json-without-feature --features broken
```

Then try these:

1. Run `01-cfg-vs-cfg-bang` once more, this time with `--no-default-features`. Does the output differ from the plain, no-features run? Why or why not?
2. On a local copy of `01-cfg-vs-cfg-bang.rs`, add a new `println!` that prints `cfg!(feature = "made-up-feature")` — a name never declared anywhere in this crate's `Cargo.toml`. Does it fail to compile, print `false`, or something else?

## Errors you will meet

### `E0432` — unresolved import

`examples/02-to-json-without-feature.rs` starts like this:

```rust
use p2_10_03_cargo_features::{to_json, Report};
```

With `--features broken` (and no `json-export`), it fails like this:

```text
error[E0432]: unresolved import `p2_10_03_cargo_features::to_json`
  --> phase2-intermediate\10-rust-toolbox\03-cargo-features\examples\02-to-json-without-feature.rs:8:31
   |
 8 | use p2_10_03_cargo_features::{to_json, Report};
   |                               ^^^^^^^ no `to_json` in the root
   |
note: found an item that was configured out
  --> phase2-intermediate\10-rust-toolbox\03-cargo-features\src\lib.rs:37:8
   |
36 | #[cfg(feature = "json-export")]
   |       ----------------------- the item is gated behind the `json-export` feature
37 | pub fn to_json(report: &Report) -> String {
   |        ^^^^^^^

For more information about this error, try `rustc --explain E0432`.
```

**What the compiler is actually objecting to:** `to_json` cannot be found at the crate root. This isn't a logic bug in your code — `to_json` genuinely doesn't exist in this build, because `json-export` is off. The compiler even says so directly: it found an item that was "configured out" — some `#[cfg(...)]` left it out of this particular build.

**The fix:** turn on the feature `to_json` actually needs:

```sh
cargo build -p p2-10-03-cargo-features --example 02-to-json-without-feature --features broken,json-export
```

**Why this is the fix:** no line of code was wrong. `to_json` has always lived behind `json-export` — exactly what "Declaring a feature and an optional dependency" told you above. The problem was in the build command, not in the program's logic.

## Exercises

### Warm up

<details>
<summary>When you compile this crate with no features on, is <code>to_json</code>'s body still somewhere in the final binary — just unreachable — or does it not exist at all?</summary>

Not at all. `#[cfg(...)]` runs before the compiler even checks types; when the condition doesn't hold, the whole item — signature, body, everything — is removed from the syntax tree before any later stage ever sees it. That's different from `if false { ... }`, which *does* compile — the optimizer just throws it away; here there's nothing left to compile in the first place.

</details>

<details>
<summary>Crate A depends on your crate (<code>you</code>) with the <code>json-export</code> feature on. Crate B, in the same build, depends on <code>you</code> with no features on. How many copies of <code>you</code> does Cargo compile?</summary>

One. Cargo builds a shared dependency exactly once per build, with the union of every feature anyone asked for — here, `json-export` on, because A wanted it. B gets a `you` with `to_json` available too, even though it never asked for it. That's unification, and it's exactly why a feature may only add — never remove, never change existing behavior.

</details>

<details>
<summary>This compiles whether the feature is on or off:</summary>

```rust
println!("{}", cfg!(feature = "json-export"));
```

Now imagine changing the body to `if cfg!(feature = "json-export") { to_json(&report) } else { String::new() }`, with nothing else changed. Does this compile with the feature off?

</details>

<details>
<summary>Answer</summary>

No. `cfg!(...)` is just a run-time boolean, not a code remover. Both arms of the `if` still have to name real, existing items and type-check, in every build, whether or not that arm ever actually runs. Since `to_json` doesn't exist without `json-export`, the first arm fails to resolve — the same `E0432` you met in "Errors you will meet," not a value of `false`.

</details>

<details>
<summary>Can crate B keep <code>json-export</code> off for the whole build, even if crate A depends on the same shared dependency with that feature on?</summary>

No. There's no way to "veto" or force off a feature that some other crate in the same build already turned on. A dependent can only ask for *more* features (or decline default ones for itself), never cancel a feature another dependent already requested. That's the flip side of unification: if turning a feature on could change or remove behavior, B would be silently broken by A's unrelated choice — exactly what the additivity rule exists to prevent.

</details>

### Repair

`examples/02-to-json-without-feature.rs` produces exactly the `E0432` above. Do two things:

1. Without touching the code, make it build just by changing the command:

```sh
cargo build -p p2-10-03-cargo-features --example 02-to-json-without-feature --features broken,json-export
```

It compiles now — nothing was wrong with the code; the feature it needed simply wasn't on.

2. Now, on a local copy of that file, make it compile in **either** configuration: put the call to `to_json` behind `#[cfg(feature = "json-export")]`, and print something else when the feature is off. Both of these should now succeed with no error:

```sh
cargo build -p p2-10-03-cargo-features --example 02-to-json-without-feature --features broken
cargo build -p p2-10-03-cargo-features --example 02-to-json-without-feature --features broken,json-export
```

### Implement

Two functions remain in `src/lib.rs` — the exact spec for each is above the function itself, in its doc comment:

- `build_report` — always available.
- `to_json` — only exists under the `json-export` feature.

```sh
cargo test -p p2-10-03-cargo-features
cargo test -p p2-10-03-cargo-features --features json-export
```

Run both. The second one also compiles and runs `json_tests` — something the first command never even compiles. Green on only one of the two means the job is half done.

### Build

Add a new feature: `csv-export`, shaped exactly like `json-export` but with no new dependency at all.

1. In `Cargo.toml`, under `[features]`: `csv-export = []`.
2. Write `pub fn to_csv(report: &Report) -> String`, behind `#[cfg(feature = "csv-export")]`, returning one CSV line: `label,count,mean,min,max`, in that exact order, comma-separated, no header row, no trailing newline — each field with its own default `{}` formatting.
3. Write a test module, behind `#[cfg(all(test, feature = "csv-export"))]`, shaped like `json_tests`.

No new dependency needed — `format!` is enough for one CSV line.

### Challenge (optional)

Add a third feature: `pretty`, which carries no code of its own — it just turns on `json-export`:

```toml
pretty = ["json-export"]
```

Then write `pub fn to_json_pretty(report: &Report) -> String`, behind `#[cfg(feature = "pretty")]`, using `serde_json::to_string_pretty`. Confirm `cargo build --features pretty` compiles even though you never separately asked for `json-export` anywhere — the same pattern you saw in "Declaring a feature and an optional dependency" (`derive = ["serde/derive"]`), this time on this crate's own features.

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Cargo feature | An opt-in, compile-time switch declared in `[features]` | Anywhere a capability or dependency needs to stay optional |
| `optional = true` / `dep:name` | A dependency that doesn't compile unless some feature turns it on | Keeping a heavy dependency out of the base crate |
| `#[cfg(feature = "...")]` | Removes an item entirely, before type-checking | A function/struct/module that should only exist in one build |
| `cfg!(feature = "...")` | A run-time boolean; removes no code | Branching on whether a feature is on, in a function compiled in every build |
| `#[cfg_attr(cond, attr)]` | The conditional form of an attribute; the item itself stays | A `derive` (or similar) that only some builds need |
| Feature unification | One shared dependency, compiled once, with the union of every requested feature | Why features must only ever add |

### What you now know

- Code behind a disabled feature is removed before type-checking — not slower, not stubbed, not there at all.
- `optional = true` plus `dep:name` inside a feature is the only way to turn on an optional dependency; `default = []` means the base crate wants none of them by itself.
- `cfg!(...)` is a run-time boolean, not a remover — both branches you condition on it still have to compile in every build.
- `#[cfg_attr(cond, attr)]` makes the attribute conditional, not the item itself.
- Cargo builds a shared dependency exactly once, with the union of every dependent's features — which is exactly why a feature may only add.

### What comes back later

- **Full JSON serialization with `derive(Serialize)`, plus validation** — [Phase 3 — `serde_json` and `validator`](../../../phase3-backend-foundations/03-serialization-and-validation/01-serde-json-and-validator/README.md)

### Can you explain?

- Why is code behind a disabled feature not "slower" or "stubbed," but simply not there?
- Show, with an example, the difference between `#[cfg(feature = "...")]` and `cfg!(feature = "...")` — one that only one of the two actually compiles with the feature off.
- How does Cargo guarantee that two different crates asking for different features of the same shared dependency still get exactly one compiled copy of it? What rule does that guarantee place on whoever writes the feature?
- Why does `Report` use `#[cfg_attr(...)]` instead of a plain `#[cfg(feature = "json-export")]` on the whole struct?

## Going further

- [The Cargo Book — Features](https://doc.rust-lang.org/cargo/reference/features.html) — the official reference, with the full `dep:` syntax.
- [docs.rs — `tokio`, Feature flags section](https://docs.rs/tokio/latest/tokio/#feature-flags) — a real crate with dozens of them, to see this pattern at scale.
- [`cargo-hack`](https://github.com/taiki-e/cargo-hack) — tests every feature combination separately, the same discipline "features must be additive" asked of you, turned into a tool.
