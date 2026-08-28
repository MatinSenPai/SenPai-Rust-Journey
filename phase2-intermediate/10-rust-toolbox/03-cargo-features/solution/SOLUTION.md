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

No explicit `is_empty` check anywhere — `Iterator::min` returns
`Option<i64>` (`None` on an empty iterator), and `?` on an `Option`
inside an `Option`-returning function early-returns the `None`. By the
time we reach the division, `min()?` has already proven the slice is
non-empty, so `samples.len()` can't be zero. One subtle ordering point:
this only works because the emptiness signal comes *first*; computing
`mean` before the `?` lines would divide by zero on an empty slice
(well — produce `NaN`, since it's float division, which is arguably
worse: no crash, just a poisoned value).

```rust
#[cfg(feature = "json-export")]
pub fn to_json(report: &Report) -> String {
    serde_json::to_string(report).expect("a plain struct of strings and numbers cannot fail to serialize")
}
```

Why `.expect` instead of returning `serde_json::Result<String>`?
`to_string` can genuinely fail — but only for types whose `Serialize`
impl can error (maps with non-string keys, custom impls that bail).
`Report` is five plain fields, so failure is unreachable in practice,
and making every caller handle a `Result` that can't happen is noise.
The `.expect` message documents the reasoning at the crash site. (If
`Report` ever grows a field with a fallible `Serialize`, this is the
line to revisit — which is exactly what an `expect` message is for.)

## The feature wiring, which is the real lesson

- `Cargo.toml`: `serde`/`serde_json` are `optional = true`, and
  `json-export = ["dep:serde", "dep:serde_json"]` is the only thing that
  turns them on. `default = []` keeps the base crate dependency-free.
- The struct uses `#[cfg_attr(feature = "json-export",
  derive(serde::Serialize))]` — conditional *attribute*, so `Report`
  exists in both builds. Plain `#[cfg]` on the struct would delete it
  from the default build and break `build_report`.
- `to_json` uses plain `#[cfg(feature = "json-export")]` — the whole
  function should not exist without the feature, and with it gone, the
  default build never references `serde_json`, which is what lets the
  dependency be optional at all.
- The JSON tests sit behind `#[cfg(all(test, feature = "json-export"))]`
  so the default `cargo test` doesn't try (and fail) to compile them.

Verify both worlds, from this `solution/` directory's manifest:

```bash
cargo test --manifest-path solution/Cargo.toml
cargo test --manifest-path solution/Cargo.toml --features json-export
```

If you only ever run the first command, `to_json` and its test are never
even compiled — a green default build says nothing about the feature
build. Real crates automate this in CI with one job per supported
feature combination (or `cargo hack --each-feature` once combinations
multiply); the discipline of "every claimed combination stays green" is
the additivity rule from the README turned into practice.
