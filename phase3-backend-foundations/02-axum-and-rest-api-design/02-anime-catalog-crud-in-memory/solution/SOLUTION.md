# Solution

```rust
pub fn create(&self, input: CreateAnime) -> Result<Anime, AnimeError> {
    validate_rating(input.rating)?;

    let mut inner = self.inner.lock().unwrap();
    inner.next_id += 1;
    let id = inner.next_id;
    let anime = Anime { id, title: input.title, status: input.status, rating: input.rating };
    inner.items.insert(id, anime.clone());
    Ok(anime)
}
```

Validation happens **before** the lock is even taken — there's no point
blocking every other request on the mutex just to reject input that was
never going to be valid. `next_id += 1` then `let id = inner.next_id`
(rather than using the *old* value and incrementing after) means IDs start
at 1, not 0, matching `create_assigns_an_id_starting_at_one`.

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

`get_mut` (not `get`) is what makes in-place field overwriting possible —
`anime` here is a `&mut Anime` borrowed from inside the `HashMap`, so
`anime.title = title` mutates the stored value directly, no re-insertion
needed. `input.rating.is_some()` (rather than `if let Some(rating) =
input.rating`) is deliberate: `Option<u8>` is `Copy`, so
`anime.rating = input.rating` can just move the whole `Option` across in
one assignment rather than unwrapping and rewrapping it.

```rust
pub async fn create_anime(
    State(store): State<Arc<AnimeStore>>,
    Json(input): Json<CreateAnime>,
) -> Result<(StatusCode, Json<Anime>), AnimeError> {
    let anime = store.create(input)?;
    Ok((StatusCode::CREATED, Json(anime)))
}
```

The `?` here works because `AnimeError` is the handler's own error type —
`store.create` returning `Result<Anime, AnimeError>` propagates directly.
`(StatusCode, Json<Anime>)` is a tuple that itself implements
`IntoResponse` (axum provides this for `(StatusCode, T)` where `T:
IntoResponse`) — that's what lets you override the default `200 OK` a bare
`Json<Anime>` would produce.

```rust
pub fn app(store: Arc<AnimeStore>) -> Router {
    Router::new()
        .route("/anime", get(list_anime).post(create_anime))
        .route("/anime/{id}", get(get_anime).patch(update_anime).delete(delete_anime))
        .with_state(store)
}
```

## On the checkpoint questions

**Q1 (where the real logic lives):** All of it lives in `AnimeStore` — the
handlers are thin adapters that unwrap an extractor, call one store method,
and wrap the result. If `create`'s logic lived directly inside
`create_anime`, every test would have to go through `tower::ServiceExt::
oneshot` and a real (or in-memory) HTTP request just to check "does
creating with rating 11 get rejected?" — testing a validation rule would
require standing up the entire HTTP stack for what's actually pure,
synchronous logic. Keeping it in `AnimeStore` is exactly what makes
`tests/store_test.rs` possible at all: fast, no `#[tokio::test]`,
no request/response plumbing, one function call per assertion.

**Q2 (explicit "clear" vs. "leave alone"):** The standard trick is
double-`Option`: `rating: Option<Option<u8>>`, or a small enum with the same
shape, e.g. `enum Patch<T> { Unset, SetTo(T), Clear }`. With
`Option<Option<u8>>`: outer `None` = field absent from the JSON body at
all = "leave alone"; outer `Some(None)` = field present with a JSON `null`
= "clear it"; outer `Some(Some(9))` = field present with a value = "set
it." Getting `serde` to actually distinguish "field absent" from "field
present but null" for this shape needs
`#[serde(default, with = "::serde_with::rust::double_option")]` (the
`serde_with` crate) or a hand-rolled `Deserialize` impl — plain `#[derive
(Deserialize)]` on `Option<Option<u8>>` collapses both cases to `None` by
default, which is the trap worth knowing about if you ever reach for this.

**Q3 (why not just return `Json<Anime>`):** `axum` defaults every
successful response to `200 OK` unless you say otherwise. `200` reads as
"here's the current state of something that already existed" — for a
`POST` that just brought a *new* resource into existence, HTTP convention
(and any API consumer's expectations, DRF's `CreateAPIView` included)
is `201 Created`. `Json<Anime>` alone would compile and technically work,
but every client (and any tooling that inspects status codes to decide
whether to retry, cache, or treat a response as an error) would get a
subtly wrong signal.

**Q4 (a shared success envelope):** This is a **bigger** change than the
error case. The error case only needed one `impl IntoResponse for
AnimeError` because every error already funnels through that single type.
Successes don't share a type at all — `list_anime` returns
`Json<Vec<Anime>>`, `get_anime` returns `Json<Anime>`, `delete_anime`
returns `StatusCode` with no body. Wrapping all of them in one envelope
would mean either introducing a generic `ApiResponse<T> { data: T }`
wrapper type and changing every handler's return type to use it, or (module
7's actual approach) applying a `tower::Layer`/middleware that rewrites
every outgoing response body uniformly, which needs no per-handler changes
at all but is a materially more advanced tool than anything used in this
lesson.

**Q5 (why one `Mutex` instead of two):** With two separate locks, another
`create` request could interleave between "read/increment `next_id`" and
"insert into `items`" — for example, two concurrent `create` calls could
both read `next_id = 0`, both increment to `1`, and both insert at key `1`,
with the second insert silently overwriting the first (`HashMap::insert`
returns and discards the old value if the key already existed). That's a
genuine lost-write race condition. One `Mutex` guarding both fields
together makes "increment `next_id`, then insert" one atomic unit as far as
every other thread/task is concerned — nobody can observe or interleave
with the state in between those two steps.
