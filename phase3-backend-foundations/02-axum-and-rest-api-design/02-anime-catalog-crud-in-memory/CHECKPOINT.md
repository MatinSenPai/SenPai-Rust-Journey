# Checkpoint

1. `tests/store_test.rs` never imports `axum` or builds a `Request` at all,
   yet it covers every validation rule and not-found case the API has. What
   does that tell you about where the *real* logic of this lesson lives —
   the store or the handlers? What would change about your testing
   strategy if `AnimeStore::create` lived entirely inside the
   `create_anime` handler instead of being its own type?
2. `update_overwrites_only_the_fields_that_were_set` checks that leaving
   `title: None` in an `UpdateAnime` leaves the stored title untouched.
   `UpdateAnime` has no way to represent "explicitly clear the rating back
   to unrated" — `rating: None` only ever means "don't touch it." How would
   you change `UpdateAnime`'s shape to support both "leave alone" and
   "explicitly clear" for an `Option<u8>` field? (You don't have to
   implement it — describe the type.)
3. `create_anime`'s return type is `Result<(StatusCode, Json<Anime>),
   AnimeError>`. Why does the success case need an explicit
   `StatusCode::CREATED` at all — what status code would `axum` use if you
   just returned `Json<Anime>` directly, and why is that the wrong choice
   for a `POST` that creates something?
4. Every error response in this lesson has the shape `{"error": "..."}"`,
   coming from one `impl IntoResponse for AnimeError`. What would you have
   to change in this file for every *successful* response to *also* share
   one consistent envelope shape (e.g. `{"data": ...}`) — is that a bigger
   or smaller change than the error case, and why?
5. `AnimeStore` uses `Mutex<StoreInner>` — a single lock guarding both
   `next_id` and `items` together, rather than two separate `Mutex`es (one
   per field). `create` needs to both increment `next_id` *and* insert into
   `items` as one operation. What could go wrong if those were two
   independently-locked fields instead, with another request's `create`
   able to run between your `next_id` increment and your `items.insert`?
