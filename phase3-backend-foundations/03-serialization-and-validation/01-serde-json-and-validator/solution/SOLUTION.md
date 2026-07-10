# Solution

```rust
pub fn validation_summary(errors: &ValidationErrors) -> Vec<String> {
    let mut messages: Vec<String> = errors
        .field_errors()
        .iter()
        .flat_map(|(field, errs)| {
            errs.iter().map(move |e| {
                let message = e.message.clone().unwrap_or_else(|| e.code.clone()).to_string();
                format!("{field}: {message}")
            })
        })
        .collect();
    messages.sort();
    messages
}
```

`.field_errors()` returns a `HashMap<&str, &Vec<ValidationError>>` — one
field can have *multiple* broken rules at once (unlikely here, but the
type allows it), hence the nested iteration: `.flat_map` over fields,
`.map` over each field's `Vec<ValidationError>`. `e.message` is
`Option<Cow<'static, str>>` — `Some(_)` when the `#[validate(...)]`
attribute supplied a custom `message = "..."` (every rule in this lesson
does), `None` otherwise, in which case `e.code` (the rule's short name,
e.g. `"length"`, `"range"`) is the fallback.

```rust
pub fn parse_review(json: &str) -> Result<ReviewSubmission, ReviewError> {
    let submission: ReviewSubmission =
        serde_json::from_str(json).map_err(|e| ReviewError::InvalidJson(e.to_string()))?;
    submission
        .validate()
        .map_err(|errors| ReviewError::Invalid(validation_summary(&errors)))?;
    Ok(submission)
}
```

Two sequential `?`s, each converting a different underlying error type
(`serde_json::Error`, then `validator::ValidationErrors`) into the same
`ReviewError` — the same "map every failure mode into your own error
type at the point it can occur" pattern as `HttpParseError` in module 1.
Note `submission.validate()` runs strictly *after* deserialization
succeeds — there is no way to call `.validate()` on data that failed to
even parse, which is exactly the two-pass design the README describes,
made concrete.

## On the checkpoint questions

**Q1 (which line each failure hits):** A missing `title` key fails inside
`serde_json::from_str::<ReviewSubmission>(json)` — `Deserialize`'s derived
code requires every non-`#[serde(default)]` field to be present in the
JSON object, so this never gets far enough to construct a
`ReviewSubmission` at all, and returns before `.validate()` is ever
called. `rating: 11` deserializes *successfully* — `11` is a perfectly
valid `u8` (`u8` covers `0..=255`), so `serde` has no complaint — the
failure only happens on the next line, inside `submission.validate()`,
where the `#[validate(range(min = 1, max = 10))]` rule runs against an
already-fully-constructed struct. `serde` can only ever reject *shape*
(wrong type, missing field); numeric range is a semantic rule no built-in
Rust numeric type expresses on its own.

**Q2 (how `validator` skips `None`):** This is built into `validator`'s
derive macro itself — when it sees a field of type `Option<T>` with a
`#[validate(...)]` attribute, the generated code checks `if let Some(inner)
= &self.field` before running the rule against `inner`, and does nothing
at all when the field is `None`. No extra annotation is needed because the
field's *type* already tells the macro it's optional — this is another
instance of the "the type system expresses the thing directly, no separate
flag needed" pattern you've seen with `Option<T>` throughout this
curriculum.

**Q3 (deleting `.sort()`):** `HashMap` makes zero guarantees about
iteration order — it can (and does, depending on hashing/insertion
details) come back in either order across runs, or even across
compilations. Without the sort, `messages[0]` and `messages[1]` in
`reports_every_broken_rule_at_once_sorted_by_field` would be genuinely
unpredictable: sometimes `"rating: ..."` first, sometimes `"title: ..."`
first, making the test flaky — it might pass locally and fail in CI, or
pass one run and fail the next, for no code change at all. Sorting is what
turns "probably fine" into "actually deterministic."

**Q4 (advantage/downside of two passes vs. one DRF pass):** Advantage: each
pass is independently testable and has a single, narrow responsibility —
you can unit-test `validation_summary` against a hand-built
`ValidationErrors` with zero JSON involved at all, and reason about
"is this JSON well-formed" completely separately from "are these values
acceptable." Downside: DRF's single-pass `Serializer` can produce *one*
combined error response describing *every* problem (missing fields,
wrong types, *and* failed validators) in one dict, in one pass. This
two-pass design can't do that as smoothly — a structurally malformed
request (missing `title`) never even reaches the validation pass, so you
get "the JSON was bad" with no visibility into whether `rating` would
*also* have failed range validation, whereas DRF might be able to surface
both problems in the same `.errors` dict in one shot.

**Q5 (cross-field rules):** `validator` supports exactly this via
struct-level validation: `#[validate(schema(function =
"check_title_and_comment_differ"))]` (or, in newer versions, a
custom-validation attribute on the struct itself) that names a function
`fn check_title_and_comment_differ(review: &ReviewSubmission) ->
Result<(), ValidationError>` — it receives the *whole* struct rather than
one field, so it can compare `title` against `comment` directly, and its
error contributes to the same `ValidationErrors` that field-level rules
populate. This is the same escape hatch as `#[validate(custom(...))]` on a
single field, just scoped to the whole struct instead of one field, for
exactly the "this rule needs more than one field to evaluate" case.
