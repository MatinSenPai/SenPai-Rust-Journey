# Solution — 3.2.2 Writing your own extractor

## Repair

1. `05`: `type Rejection = (StatusCode, &'static str);`. A tuple of a status code and text already implements `IntoResponse`; the enum had no such impl. (Map `Missing` and `Unreadable` to a status and a message.)
2. `06`: take `Result<UserAgent, (StatusCode, &'static str)>` and match on it, as `examples/04-optional-with-result.rs` does. `Option<UserAgent>` would need `OptionalFromRequestParts`.
3. `07`: `impl<S: Send + Sync> FromRequestParts<S> for UserAgent`.
4. `08`: `async fn upload(method: Method, body: String)`. The body extractor goes last.

## `ApiKey`

```rust
let key = parts
    .headers
    .get("x-api-key")
    .and_then(|value| value.to_str().ok())
    .map(str::trim)
    .filter(|key| !key.is_empty())
    .ok_or((StatusCode::UNAUTHORIZED, "missing x-api-key header"))?;
Ok(ApiKey(key.to_string()))
```

Three failures (absent, not text, blank) collapse into one `None`, so one `ok_or` gives them all the same `401`. The `?` returns the rejection, and `axum` turns it into the response.

## `Pagination`

```rust
let Query(raw) = Query::<RawPagination>::from_request_parts(parts, state)
    .await
    .map_err(|_| (StatusCode::BAD_REQUEST, "page and per_page must be whole numbers"))?;
let page = raw.page.unwrap_or(1);
let per_page = raw.per_page.unwrap_or(DEFAULT_PER_PAGE);
if page == 0 || per_page == 0 { /* 400, at least 1 */ }
Ok(Pagination { page, per_page: per_page.min(MAX_PER_PAGE) })
```

`RawPagination` has `Option<u32>` fields, so a missing key is `None` and `Query` itself rejects `abc`, `-1` (not a `u32`) and a repeated key. Zero is a valid `u32`, so the extractor checks it. The cap is `min`, not an error.

## `ClientVersion`

A missing header gives the first `400`. Everything else goes through one `bad` rejection: `to_str`, then `split_once('.')`, then `parse::<u32>()` on each half. `1` has no dot; `1.2.3` leaves `2.3`, which is not a `u32`; `1.`, `.4`, `1.-2` and the empty string fail to parse. Both halves must parse, so "exactly two whole numbers" falls out of three small steps.

## Challenge

```rust
impl<S> FromRequestParts<S> for Authorized
where
    S: Send + Sync,
    KeyStore: FromRef<S>,
{
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let ApiKey(key) = ApiKey::from_request_parts(parts, state).await?;
        let store = KeyStore::from_ref(state);
        /* 403 when the key is not in the set */
    }
}
```

`ApiKey` already rejects a missing key with `401`, and `?` passes that through unchanged. `KeyStore: FromRef<S>` means the state only has to be able to produce a `KeyStore`; a router whose state is exactly a `KeyStore` works because `FromRef<T> for T` exists for `Clone` types. The full version is in `src/lib.rs` under `challenge`.
