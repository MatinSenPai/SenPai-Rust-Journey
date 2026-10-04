# Solution — 3.2.5 CORS and frontend integration

## `dev_cors`

```rust
CorsLayer::new()
    .allow_origin(Any)
    .allow_methods(Any)
    .allow_headers(Any)
```

`Any` is a unit struct from `tower_http::cors`. Each `allow_*` method accepts anything that converts into its header type, and `Any` becomes a literal `*` in the response, which is why the dev test asserts `*` exactly. `CorsLayer::permissive()` is this same layer prebuilt (it also exposes every response header). No `allow_credentials`, so no credentials header is sent.

## `prod_cors`

```rust
let origin = allowed_origin.parse::<HeaderValue>().expect("invalid allowed origin");
CorsLayer::new()
    .allow_origin(AllowOrigin::list([origin]))
    .allow_methods([Method::GET, Method::POST])
    .allow_headers([header::CONTENT_TYPE])
```

Three decisions worth defending:

- **`AllowOrigin::list`, not a bare `HeaderValue`.** With a single `HeaderValue`, `tower-http` stamps that fixed value on *every* response, even one to `https://evil.example.com`, and leans on the browser to reject it. With `list`, the layer compares the request's `Origin` against the list and sends `access-control-allow-origin` only on a match. That is what the "unknown origin is not vouched for" test pins down.
- **`.expect` on the parse.** A bad origin is a deployment mistake. Panicking at startup beats a healthy-looking process whose every browser client fails silently.
- **Only `GET`/`POST` and `content-type`.** Grant what the frontend uses: a future `DELETE` route then fails closed until someone adds `Method::DELETE` on purpose. The preflight answer lists these constants and never mirrors what was asked, which is why asking for `authorization` still gets `content-type` back.

## `prod_cors_from_list` (the "Build" exercise)

```rust
let origins = allowed_origins
    .split(',')
    .map(str::trim)
    .filter(|entry| !entry.is_empty())
    .map(|entry| {
        assert!(!entry.ends_with('/'), "allowed origin {entry:?} ends with a slash and can never match");
        entry.parse::<HeaderValue>().expect("invalid allowed origin")
    })
    .collect();
```

The same `AllowOrigin::list`, `allow_methods` and `allow_headers` as `prod_cors` then apply to `origins` (the solution shares them in a private `prod_layer`, and `prod_cors` calls it with one origin). `{entry:?}` prints the entry in quotes, and the test only looks for the entry text inside the message. An empty list gives `AllowOrigin::list([])`, which vouches for nobody.

## On the challenge

```rust
CorsLayer::new().expose_headers([HeaderName::from_static("x-total-count")])
```

`expose_headers` adds `access-control-expose-headers` to **non-preflight** responses only; the preflight does not carry it. Without it, JavaScript sees the response but `response.headers.get("x-total-count")` is `null`.
