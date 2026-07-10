# 03.1 — `serde_json` and `validator`

## Two separate jobs DRF bundles into one

A DRF `Serializer` does two things at once: it turns JSON into Python
objects (and back), *and* it validates the data while doing so
(`.is_valid()`, `.errors`). Rust deliberately keeps these as two separate
libraries with two separate jobs:

- **`serde`** (+ `serde_json`) only cares about *shape*: does this JSON
  parse into a `ReviewSubmission` struct at all — right field names, right
  types? A `rating` field containing `"nine"` instead of `9` fails at this
  stage; deserialization itself is the check.
- **`validator`** only cares about *rules*, once you already have a
  correctly-shaped `ReviewSubmission`: is `rating` actually between 1 and
  10? Is `title` non-empty and under some length? These aren't things a
  type alone can express (Rust's type system doesn't have "a `u8` between 1
  and 10" as a distinct type from `u8`), so a second, explicit pass handles
  them.

## Deriving `Deserialize`/`Serialize`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewSubmission {
    pub title: String,
    pub rating: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}
```

`#[derive(Deserialize)]` generates the JSON-parsing code for you (this is
what `serde_json::from_str::<ReviewSubmission>(json)` calls under the
hood) — no hand-written `parse_request`-style byte-splitting like module 1,
because JSON's grammar is standardized and `serde_json` already implements
it correctly and fast. Two attributes worth knowing:

- **`#[serde(default)]`** — if `comment` is absent from the incoming JSON
  entirely, use `Option::default()` (`None`) instead of failing to
  deserialize. Without this, a JSON object missing the `comment` key
  entirely would be a hard error, even though `Option<String>` "should"
  mean optional.
- **`#[serde(skip_serializing_if = "Option::is_none")]`** — the *opposite*
  direction: when *serializing* a `ReviewSubmission` back to JSON, omit the
  `comment` key entirely if it's `None`, rather than writing
  `"comment": null`. Whether you want `null` or an omitted key is a real
  API design choice — DRF's default is closer to always including the key
  with `null`; this attribute is how you opt into the other convention.

## Deriving `Validate`

```rust
#[derive(Debug, Validate)]
pub struct ReviewSubmission {
    #[validate(length(min = 1, max = 200, message = "title must be 1-200 characters"))]
    pub title: String,

    #[validate(range(min = 1, max = 10, message = "rating must be between 1 and 10"))]
    pub rating: u8,

    #[validate(length(max = 1000, message = "comment must be at most 1000 characters"))]
    pub comment: Option<String>,
}
```

`#[derive(Validate)]` generates a `.validate(&self) -> Result<(),
ValidationErrors>` method. Each `#[validate(...)]` attribute is a rule —
`length` for strings/collections, `range` for numbers, plus others
(`email`, `url`, `must_match`, `custom`) not needed here. Notice
`comment: Option<String>` gets a `length` rule directly on the `Option` —
`validator` only runs the rule when the `Option` is `Some`, skipping `None`
automatically, exactly the "only validate a field that's actually
present" behavior DRF's `required=False` fields give you.

`ValidationErrors` (what `.validate()` returns in the `Err` case) is a
structured, per-field map — `errors.field_errors()` gives you a
`HashMap<&str, &Vec<ValidationError>>`, the same shape as DRF's
`serializer.errors` dict (`{"title": [...], "rating": [...]}`), just
strongly typed instead of a loosely-typed dict.

## Your task

Implement the `todo!()`s in `src/lib.rs`:

- `validation_summary` — flatten a `ValidationErrors` into a sorted
  `Vec<String>` of `"field: message"` strings, easy to assert against in
  tests (and easy to hand back in an HTTP error body, once module 7 wires
  this into `axum` for real).
- `parse_review` — deserialize a JSON string into `ReviewSubmission`, then
  validate it, producing a `ReviewError` that distinguishes "the JSON
  itself was malformed" from "the JSON parsed fine but broke a validation
  rule."

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
