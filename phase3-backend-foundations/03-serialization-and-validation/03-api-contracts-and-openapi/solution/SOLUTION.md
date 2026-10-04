# Solution — 3.3.3 API contracts and OpenAPI (`utoipa`)

The full code is `solution/src/lib.rs`; it passes every test in `solution/tests/`, including `build_test.rs` for the Build rung.

## The four readers

All four work on the document as a plain `serde_json::Value`, and all four lean on one fact: indexing a `Value` with a missing key gives `Value::Null` instead of panicking, so a missing path, method or status falls through to "nothing".

```rust
pub fn schema_names(doc: &Value) -> Vec<String> {
    let mut names: Vec<String> = match doc["components"]["schemas"].as_object() {
        Some(schemas) => schemas.keys().cloned().collect(),
        None => Vec::new(),
    };
    names.sort();
    names
}
```

`as_object()` is `None` for `Null`, which covers "no `components`" and "no `schemas`" in one arm. Sorting is explicit because `serde_json::Value` keeps its object keys in a `BTreeMap` only by default; sorting here does not depend on that.

```rust
pub fn operation_statuses(doc: &Value, method: &str, path: &str) -> Vec<String> {
    let responses = &doc["paths"][path][method.to_lowercase()]["responses"];
    /* same keys-then-sort as above */
}
```

OpenAPI spells methods in lower case (`"get"`), so `method.to_lowercase()` makes `"GET"`, `"Get"` and `"get"` the same lookup. `response_content_types` is the same walk one level deeper (`["responses"][status]["content"]`); a `204` has no `content`, so it ends in `Null` and gives an empty `Vec`.

```rust
pub fn undocumented(doc: &Value, routes: &[(&str, &str)]) -> Vec<String> {
    routes.iter()
        .filter(|(method, path)| doc["paths"][*path][method.to_lowercase()].is_null())
        .map(|(method, path)| format!("{method} {path}"))
        .collect()
}
```

`filter` keeps the routes with no operation and `map` formats them with the caller's spelling of the method (`"Post /a"`, not `"post /a"`). Iterating `routes` in order is what keeps the output in the order given.

## The `content(...)` spelling

In `utoipa` 5 a multi-media-type response is `content((ApiError = "application/json"), (String = "text/plain"))`: schema first, media type after the `=`. The `utoipa` 4 order (`("application/json" = ApiError)`) is a macro parse error, `expected ,`. The single-type form `body = String, content_type = "text/plain"` is also valid and is used for `400` and `415`.

## The Build rung

```rust
#[utoipa::path(put, path = "/anime/{id}", request_body = CreateAnime,
    params(("id" = u64, Path, description = "the anime's id")),
    responses((status = 200, body = Anime), (status = 404, body = ApiError),
              (status = 422, content((ApiError = "application/json"), (String = "text/plain")))))]
pub async fn replace_anime(/* State, Path(id), Json(input) */) -> ApiResult<Json<Anime>>
```

Four edits make a new endpoint complete: the handler with its attribute, `paths(..., replace_anime)`, `.put(replace_anime)` on the `/anime/{id}` route, and `("PUT", "/anime/{id}")` in `ROUTES`. Forget the third and the route answers `405`; forget the first or second and `undocumented` lists the route; forget the fourth and nothing in the readers notices, because `ROUTES` is the list they trust (`build_test.rs` checks that entry directly). The rating is checked before the id, as for `POST`, and `replace_anime` overwrites the three fields in place, so the id never changes and a repeated identical `PUT` gives the same response (`put_replaces_the_whole_anime_and_keeps_the_id` sends it twice).
