# Solution — 3.2.3 Anime catalog CRUD (in-memory)

The full code is `solution/src/lib.rs`; it passes every test in `solution/tests/`, including `build_test.rs` for the Build rung.

## `AnimeError::into_response`

```rust
fn into_response(self) -> Response {
    let (status, message) = match self {
        AnimeError::NotFound => (StatusCode::NOT_FOUND, "anime not found".to_string()),
        AnimeError::InvalidRating(r) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("rating must be between 1 and 10, got {r}"),
        ),
    };
    (status, Json(serde_json::json!({ "error": message }))).into_response()
}
```

The `match` picks a `(status, message)` pair per variant, and the last line builds a `(StatusCode, Json<Value>)` tuple, which already implements `IntoResponse`. `Json` also sets `Content-Type: application/json`. This is the only place in the program where the words "not found" and the number `404` meet. A bad rating is `422`, not `400`, because the request parsed and the content broke a rule (3.1.3).

## The store

```rust
pub fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
    validate_rating(input.rating)?;
    let mut inner = self.inner.lock().unwrap();
    inner.next_id += 1;
    let anime = Anime { id: inner.next_id, title: input.title,
                        status: input.status, rating: input.rating };
    inner.items.insert(anime.id, anime.clone());
    Ok(anime)
}
```

Validation comes before the lock, for two reasons: a rejected request never waits on the mutex, and it never uses up an id (`a_rejected_create_does_not_use_up_an_id`). The id is incremented *before* it is read, so ids start at `1`. They also never repeat after a delete, because `next_id` only goes up and is not derived from the map's size (`ids_are_never_reused_after_a_delete`).

```rust
pub fn update(&self, id: u64, input: UpdateAnime) -> Result<Anime, AnimeError> {
    validate_rating(input.rating)?;
    let mut inner = self.inner.lock().unwrap();
    let anime = inner.items.get_mut(&id).ok_or(AnimeError::NotFound)?;
    if let Some(title) = input.title { anime.title = title; }
    if let Some(status) = input.status { anime.status = status; }
    if input.rating.is_some() { anime.rating = input.rating; }
    Ok(anime.clone())
}
```

The spec says the rating is checked first, so `update(999, rating: 20)` is `InvalidRating`, not `NotFound` (`update_checks_the_rating_before_the_id`). `get_mut` gives a `&mut Anime` inside the map, so the fields are overwritten in place. Nothing is modified until both checks have passed, so every `Err` leaves the store untouched.

`get` is `.get(&id).cloned().ok_or(NotFound)`, `delete` is `.remove(&id).ok_or(NotFound)`, and `list` collects the values, then `sort_by_key` on the id. A `HashMap` gives no order, and the 20-item test would catch an unsorted list.

Every method holds the `MutexGuard` only until it returns. The store is synchronous, so a guard can never be alive at an `.await`.

## Handlers and the route table

```rust
pub async fn create_anime(State(store): State<Arc<AnimeStore>>, Json(input): Json<CreateAnime>)
    -> Result<(StatusCode, [(HeaderName, String); 1], Json<Anime>), AnimeError>
{
    let anime = store.create(input)?;
    let location = format!("/anime/{}", anime.id);
    Ok((StatusCode::CREATED, [(header::LOCATION, location)], Json(anime)))
}
```

`?` converts nothing here: the store's error type is already the handler's. The three-part tuple is `(StatusCode, headers, body)`, and `axum` implements `IntoResponse` for it. The other four handlers are one line each: `Json(store.list())`, `store.get(id).map(Json)` (or `?` and `Ok(Json(..))`), the same for update, and `store.delete(id)?; Ok(StatusCode::NO_CONTENT)`.

```rust
Router::new()
    .route("/anime", get(list_anime).post(create_anime))
    .route("/anime/{id}", get(get_anime).patch(update_anime).delete(delete_anime))
    .with_state(store)
```

## Build: `PUT`

`AnimeStore::replace(id, CreateAnime)` validates the rating, then looks the id up with `get_mut(...).ok_or(NotFound)?` and overwrites the whole entry with `*anime = Anime { id, .. }`. The handler is the same shape as `update_anime`, with `Json<CreateAnime>`, and `.put(replace_anime)` is chained into the `/anime/{id}` route. A `PUT` to a missing id is `404` and creates nothing, because ids belong to the server. A body missing a required field never reaches the handler: `Json` rejects it with `422`. Sending the same `PUT` twice gives the same status, the same body and the same state, and `the_same_put_twice_gives_the_same_state_and_the_same_answer` asserts all three. That is the idempotency 3.1.3's table promises for `PUT`.

## Challenge: `If-Match`

One workable shape: add `version: u64` to `Anime` (set to `1` on create, `+= 1` on every successful update or replace); add `AnimeError::VersionMismatch` and a `412 Precondition Failed` arm; give `update_anime` a `HeaderMap` extractor, parse `If-Match` as a `u64`, and pass it to `update`, which compares it with the stored version *inside the same lock* before changing anything. The check and the write must happen under one lock acquisition. If they were two separate calls, another request could slip in between them, and you would be back to the lost-update problem.

## On the warm-ups and the errors

- **Why is `Result<Json<Anime>, AnimeError>` a legal return?** Because `Result<T, E>: IntoResponse` when both halves are. Remove your `impl IntoResponse for AnimeError` and you get exactly the `E0277` from `examples/03-error-without-into-response-broken.rs`.
- **The `MutexGuard` repairs.** Example 02 and 04: wrap the lock in a block that returns the value you need (`let now = { let mut g = ...; *g += 1; *g };`), as `examples/06-guard-dropped-before-await-fix.rs` does. `drop(guard)` before the `.await` also works, but the block form cannot be forgotten. Using `tokio::sync::Mutex` would also compile, and is the right tool only when the lock really must be held across an `.await`.
- **Example 03.** Add `impl IntoResponse for ShowError` with `(StatusCode::NOT_FOUND, "no such show").into_response()`.
- **Example 05.** Return `Ok((StatusCode::CREATED, Json(..)))`. The handler's return type has to change with it, so `use axum::http::StatusCode;` is the import you need.

## What this lesson was really about

None of the code is long. What matters is where each decision lives: rules in the store, the status-code vocabulary in one `IntoResponse` impl, and the method semantics (`201`, `204`, `PATCH` vs `PUT`) in the handlers and the route table. When module 5 swaps the `HashMap` for Postgres, only the store changes.
