# Checkpoint

1. `ApiError` implements both `thiserror::Error` (giving it `Display`) and
   `axum::response::IntoResponse`. What job does each trait actually do, and
   why isn't `Display` alone enough for axum to turn an `Err(ApiError)` into
   an HTTP response? What would you have to change if you wanted `ApiError`
   to also work as the error type for a `tonic` gRPC service (Phase 4) at
   the same time?
2. The error envelope has both a `code` (`"not_found"`, `"validation_failed"`)
   and a `message` (a free-text sentence). Give a concrete example of a
   frontend bug that a `code` field prevents but a `message`-only API would
   be vulnerable to. Why is it *not* enough to just document "the message
   for a 404 is always `\"X not found\"`" instead of adding a separate
   `code` field?
3. `WidgetStore::create` and `WidgetStore::get` both return
   `Result<Widget, ApiError>` directly — there's no separate
   `WidgetStoreError` that later gets converted at the HTTP boundary (unlike
   `taskforge-api`'s `JobError` → `ApiError` conversion). What's the
   tradeoff? Under what circumstances would you introduce a separate domain
   error type for a crate this size, and when would that just be extra
   ceremony?
4. `every_error_response_shares_the_same_envelope_shape` in `tests/
   api_test.rs` checks that both a 404 and a 400 response have exactly the
   keys `["code", "message"]` under `"error"`. What kind of regression would
   this specific test catch that `create_with_empty_name_returns_400_...`
   and `get_missing_widget_returns_404_...` — which each test one error case
   in isolation — would *not* catch on their own?
5. Suppose you need to add a new failure mode: creating a widget whose name
   already exists should return `409 Conflict` instead of succeeding. Walk
   through every place in `src/lib.rs` you'd need to touch to add this
   cleanly (new variant, matching arm, code string, store logic) — and name
   one place you would *not* need to touch, and why not.
6. `From<validator::ValidationErrors> for ApiError` sorts its collected
   error messages before joining them. `HashMap` iteration order is
   unspecified per-run. What test failure mode does the `.sort()` call
   prevent, and why would that failure mode be especially annoying to
   debug if it showed up (hint: think about how often it would actually
   reproduce)?
