# Checkpoint

1. In the `[features]` table, what does `json-export = ["dep:serde",
   "dep:serde_json"]` mean, piece by piece? What role does
   `optional = true` on the dependency lines play — what would happen if
   you removed it (and the `dep:` entries)?
2. Your library is used by two crates in the same workspace: one enables
   `json-export`, the other doesn't. How many times does Cargo compile
   your library, and with which features? What is this behavior called?
3. Explain, using your answer to question 2, why a feature that *removes*
   or *changes* existing behavior (rather than only adding) is a design
   bug. Invent a concrete two-dependents scenario where it breaks.
4. `Report` uses `#[cfg_attr(feature = "json-export",
   derive(serde::Serialize))]` while `to_json` uses `#[cfg(feature =
   "json-export")]`. Why does the struct need `cfg_attr` instead of
   plain `cfg` — what would `#[cfg(feature = "json-export")]` on the
   struct itself do to the no-feature build?
5. The JSON tests live in `#[cfg(all(test, feature = "json-export"))]`.
   What two commands do you run to know the crate is green in both
   worlds, and what would silently go untested if you only ever ran plain
   `cargo test`?
