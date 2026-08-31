# Solution

```rust
pub fn build_report(label: &str, samples: &[i64]) -> Option<Report> {
    let min = samples.iter().copied().min()?;
    let max = samples.iter().copied().max()?;
    let sum: i64 = samples.iter().sum();

    Some(Report {
        label: label.to_string(),
        count: samples.len(),
        mean: sum as f64 / samples.len() as f64,
        min,
        max,
    })
}
```

No explicit `is_empty` check anywhere — `Iterator::min` returns `Option<i64>`
(`None` on an empty iterator), and `?` on an `Option` inside a function that
itself returns `Option` early-returns the `None`. By the time execution
reaches the division, `min()?` has already proven the slice is non-empty, so
`samples.len()` can't be zero. One subtle ordering point: this only works
because the emptiness signal comes *first* — computing `mean` before the `?`
lines would divide by zero on an empty slice (well, produce `NaN`, since it's
float division, which is arguably worse: no crash, just a poisoned value
that keeps propagating).

```rust
#[cfg(feature = "json-export")]
pub fn to_json(report: &Report) -> String {
    serde_json::to_string(report)
        .expect("a plain struct of strings and numbers cannot fail to serialize")
}
```

Why `.expect` instead of returning `serde_json::Result<String>`? `to_string`
can genuinely fail — but only for types whose `Serialize` impl can error
(maps with non-string keys, custom impls that bail). `Report` is five plain
fields, so failure is unreachable in practice, and making every caller
handle a `Result` that can't happen is noise. The `.expect` message documents
the reasoning at the crash site.

```rust
#[cfg(feature = "csv-export")]
pub fn to_csv(report: &Report) -> String {
    format!(
        "{},{},{},{},{}",
        report.label, report.count, report.mean, report.min, report.max
    )
}
```

The Build rung's own feature — mirrors `json-export`'s shape (`[]`, no
`dep:` list because there's nothing optional to turn on) with zero new
dependencies. `format!` and each field's own `Display` impl are enough; Rust
prints a whole-number `f64` like `30.0` as `30`, which is why the test below
asserts `"30"`, not `"30.0"`.

```rust
#[cfg(feature = "pretty")]
pub fn to_json_pretty(report: &Report) -> String {
    serde_json::to_string_pretty(report)
        .expect("a plain struct of strings and numbers cannot fail to serialize")
}
```

The Challenge rung. `pretty = ["json-export"]` in `Cargo.toml` is the whole
trick — turning `pretty` on turns `json-export` on for you, transitively, the
same shape as `derive = ["serde/derive"]` would turn on a *dependency's*
feature instead of this crate's own. `pretty_tests` only compiles at all
because that worked: it calls `to_json`, a `json-export`-gated function,
while only ever asking Cargo for `pretty`.

## The feature wiring, which is the real lesson

- `Cargo.toml`: `serde`/`serde_json` are `optional = true`, and
  `json-export = ["dep:serde", "dep:serde_json"]` is the only thing that
  turns them on. `csv-export = []` and `pretty = ["json-export"]` sit right
  next to it — same table, same shape, one with nothing to turn on but
  itself, one that turns on another feature instead of a dependency.
  `default = []` keeps the base crate dependency-free.
- The struct uses `#[cfg_attr(feature = "json-export",
  derive(serde::Serialize))]` — a conditional *attribute*, so `Report`
  exists in both builds. A plain `#[cfg]` on the struct would delete it from
  the default build and break `build_report`, whose return type names it
  unconditionally.
- `to_json`, `to_csv`, and `to_json_pretty` each use a plain
  `#[cfg(feature = "...")]` on the function itself — the whole function
  should not exist without its feature. With `to_json` gone, the default
  build never references `serde_json` at all, which is what lets the
  dependency be optional in the first place.
- Every feature-specific test module sits behind
  `#[cfg(all(test, feature = "..."))]`, so a plain `cargo test` never even
  tries (and fails) to compile it.

Verify every world, from this `solution/` directory's manifest:

```bash
cargo test --manifest-path solution/Cargo.toml
cargo test --manifest-path solution/Cargo.toml --features json-export
cargo test --manifest-path solution/Cargo.toml --features csv-export
cargo test --manifest-path solution/Cargo.toml --features pretty
cargo test --manifest-path solution/Cargo.toml --all-features
```

If you only ever run the first command, none of `to_json`, `to_csv`, or
`to_json_pretty` — nor their tests — are ever even compiled. A green default
build says nothing about any of the feature builds. Real crates automate
this in CI with one job per supported combination (or `cargo hack
--each-feature` once combinations multiply); the discipline of "every
claimed combination stays green" is the additivity rule from the README,
turned into practice.
