# 02.2 — Anime catalog CRUD (in-memory)

## Putting last lesson's pieces together into a real resource

Last lesson's routes were mostly independent demos (`/greet/{name}`,
`/echo`, `/counter`). This lesson is the shape every REST resource takes in
practice: **one struct, five operations, one route table** — exactly what
you'd scaffold with `ModelViewSet` in DRF, built here by hand so you feel
every piece of it.

| DRF (`ModelViewSet` on an `Anime` model) | This lesson |
|---|---|
| `GET /anime/` → `.list()` | `GET /anime` → `list_anime` |
| `POST /anime/` → `.create()` | `POST /anime` → `create_anime` |
| `GET /anime/{id}/` → `.retrieve()` | `GET /anime/{id}` → `get_anime` |
| `PATCH /anime/{id}/` → `.partial_update()` | `PATCH /anime/{id}` → `update_anime` |
| `DELETE /anime/{id}/` → `.destroy()` | `DELETE /anime/{id}` → `delete_anime` |
| Django ORM `Anime.objects` | `AnimeStore` (in-memory today, Postgres in module 4) |

The store is genuinely in-memory (a `HashMap` behind a `Mutex`, gone the
moment the process exits) — that's deliberate. Module 4 swaps this exact
same shape onto real Postgres, so you can see precisely which parts of a
CRUD API are about *HTTP and REST design* (this lesson) versus which parts
are about *persistence* (next module). The route table, the request/response
shapes, and the error handling barely change at all.

## Pure logic, thin HTTP edge — again

`AnimeStore` in `src/lib.rs` knows nothing about `axum`, `Json`, or HTTP
status codes — it's plain Rust: a `HashMap<u64, Anime>` behind a `Mutex`,
with `create`/`get`/`list`/`update`/`delete` methods that return
`Result<Anime, AnimeError>`. `tests/store_test.rs` exercises all of that
directly, no HTTP involved at all — the same "test the core logic without
touching I/O" discipline as every prior Phase 3 lesson, now applied to a
whole CRUD resource instead of one function.

The **handlers** are the thin translation layer: pull arguments out of the
request with extractors, call the store, translate the `Result` into an
HTTP response. `tests/api_test.rs` drives the *whole* stack — router,
extractors, handlers, store — through `tower::ServiceExt::oneshot`, the
same technique from last lesson.

## One route, multiple methods

```rust
Router::new()
    .route("/anime", get(list_anime).post(create_anime))
    .route("/anime/{id}", get(get_anime).patch(update_anime).delete(delete_anime))
```

New since last lesson: `.route(path, method_router)` takes one
`MethodRouter`, and `get(handler)` (etc.) can be **chained** —
`get(list_anime).post(create_anime)` registers both methods against the
same path, rather than needing two separate `.route(...)` calls. This
mirrors DRF's `ModelViewSet` registering multiple HTTP verbs against one
URL pattern via a router, just spelled out explicitly instead of generated
from a class.

## Turning a domain error into an HTTP response

```rust
pub enum AnimeError {
    NotFound,
    InvalidRating(u8),
}

impl IntoResponse for AnimeError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AnimeError::NotFound => (StatusCode::NOT_FOUND, "anime not found".to_string()),
            AnimeError::InvalidRating(r) => (
                StatusCode::BAD_REQUEST,
                format!("rating must be between 1 and 10, got {r}"),
            ),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}
```

Implementing `axum::response::IntoResponse` for your own error type is what
lets a handler return `Result<Json<Anime>, AnimeError>` directly — `axum`
calls `.into_response()` on whichever variant comes back, `Ok` or `Err`,
with no `match` needed in the handler itself. This is a small preview of
module 7's "consistent error envelopes" — every error in this lesson comes
back as `{"error": "..."}`, which is already the instinct DRF's
`exception_handler` encourages, just built by hand once so you see exactly
where that JSON shape comes from.

## Your task

Implement the `todo!()`s in `src/lib.rs`:

- `AnimeStore::create` / `get` / `list` / `update` / `delete` — the pure,
  no-HTTP CRUD logic.
- `create_anime` / `list_anime` / `get_anime` / `update_anime` /
  `delete_anime` — the `axum` handlers, thin wrappers over the store.
- `app` — the route table wiring both methods onto `/anime` and three
  methods onto `/anime/{id}`.

## Try it for real

```sh
cargo run -p p3-02-02-anime-catalog-crud-in-memory &
curl -X POST -H 'content-type: application/json' \
  -d '{"title":"Frieren","status":"watching","rating":9}' http://127.0.0.1:3001/anime
curl http://127.0.0.1:3001/anime
curl -X PATCH -H 'content-type: application/json' -d '{"status":"completed"}' http://127.0.0.1:3001/anime/1
curl -X DELETE http://127.0.0.1:3001/anime/1
curl http://127.0.0.1:3001/anime/1   # 404 now
```

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
