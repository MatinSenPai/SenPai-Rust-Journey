# Checkpoint

1. `rejects_a_missing_required_field_as_invalid_json` and
   `rejects_an_out_of_range_rating` both fail, but as different
   `ReviewError` variants. Walk through exactly which line of `parse_review`
   each one fails at, and why a `rating` of `11` can only be caught by
   `validator`, never by `serde`, even though both are "wrong data."
2. `comment` has `#[serde(default, skip_serializing_if = "Option::is_none")]`
   on the `serde` side and `#[validate(length(max = 1000, ...))]` on the
   `validator` side, with no `required = false`-style annotation anywhere
   telling `validator` the field is optional. How does `validator` know not
   to run the `length` rule when `comment` is `None`?
3. `validation_summary` sorts its output before returning it. What's the
   actual, concrete failure mode you'd hit in
   `reports_every_broken_rule_at_once_sorted_by_field` if that `.sort()`
   call were deleted? (What does `HashMap` promise about iteration order,
   and what does that mean for a test asserting on `messages[0]` and
   `messages[1]`?)
4. Compare this lesson's two-pass design (`serde` then `validator`) to a
   single DRF `Serializer` subclass with both field-level constraints
   (`max_length=200`) and `.is_valid()` in one place. What's a concrete
   advantage of keeping "does this parse" and "is this acceptable" as two
   separate, independently testable steps, and what's a concrete downside
   (something DRF's single-pass design gets you for free that this
   two-pass design doesn't)?
5. If you wanted to add a rule that isn't expressible as `length` or
   `range` — say, "the title must not be identical to the comment" (a
   cross-field rule, not a single-field one) — is that something
   `#[validate(...)]` attributes on individual fields can express at all?
   Skim the `validator` crate's docs for `#[validate(custom(...))]` or
   struct-level validation and describe, in a sentence or two, how you'd
   approach it.
