# 3.3.3 — API contracts and OpenAPI (`utoipa`)

## At a glance

After this lesson you can:

- Generate an OpenAPI 3.1 document from your own types and handlers with `#[derive(ToSchema)]`, `#[utoipa::path]` and `#[derive(OpenApi)]`, and serve it from an `axum` route.
- Test the contract: assert that the document lists the paths, schemas, status codes and content types you promise, and that the router really answers them.
- Read the three compile errors `utoipa` gives you, and spot the two mistakes it gives you no error for (a route nobody documented, a status the handler never returns).

**Time:** ~90 minutes · **Prerequisites:**
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md),
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md),
[3.3.1 — Serde in depth](../01-serde-depth/README.md),
[3.3.2 — Validation](../02-validation/README.md)

---

## Why this matters

3.2.3 built a catalog API and chose a status code for every outcome. Those choices live in the code, and a client developer cannot read your code. They need a *contract*: a document that says "`POST /anime` answers `201` with this JSON, or `422` with that JSON". The format everyone agreed on is **OpenAPI**. Frontend code generators, API gateways, Postman, and mock servers all read it.

You can write that document by hand. It is correct on the day you write it. A week later someone adds a field, or a new route, or changes `201` to `200`, and nobody opens the YAML. A hand-written spec drifts, and a wrong contract is worse than none, because people trust it.

In Django you may know `drf-spectacular`: it reads your serializers and views and generates the schema, so the document follows the code. `utoipa` is the same idea for Rust, with one big difference in *how*. Python can look at your classes while the program runs. Rust cannot, so `utoipa` works at compile time: derive macros on your types and an attribute on your handlers write the document for you. In exchange you must list what belongs in it. This lesson shows exactly what that buys you, and the two places where it still lets the document lie.

---

## The concept

### The contract is a JSON document

```senpai-visual
{"kind":"concept","labels":["Rust types: ToSchema","Handlers: utoipa::path","ApiDoc: OpenApi derive","ApiDoc::openapi()","JSON route: /api-docs/openapi.json","clients, code generators, gateways"]}
```

An OpenAPI document is one JSON object with a fixed shape. This outline is the part of it this lesson uses (an outline, not program output):

```text
openapi: "3.1.0"
info:        title, version
paths
  "/anime/{id}"
    get -> responses -> "200" -> content -> "application/json" -> schema
components
  schemas -> Anime, ApiError, CreateAnime, WatchStatus
```

Every operation (a method on a path) lists its `responses` by status code. Every response lists the media types it can come back as, and each media type points at a schema. Schemas that several operations share live once under `components`, and are referred to by `$ref`.

### Types become schemas: `ToSchema`

```rust
#[derive(Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum WatchStatus { Watching, Completed, PlanToWatch, Dropped }

/// One catalog entry.
#[derive(Serialize, Deserialize, ToSchema)]
pub struct Anime {
    #[schema(example = 1)]
    pub id: u64,
    pub title: String,
    pub status: WatchStatus,
    /// `1..=10`, or `null` when the show is not rated yet.
    #[schema(minimum = 1, maximum = 10)]
    pub rating: Option<u8>,
}
```

`examples/08-print-a-schema.rs` prints what the derive wrote for these two:

```sh
cargo run -p p3-03-03-api-contracts-and-openapi --example 08-print-a-schema
```

```text
--- WatchStatus
{
  "description": "Where you are with a show. On the wire it is snake_case:\n`\"watching\"`, `\"completed\"`, `\"plan_to_watch\"`, `\"dropped\"`.",
  "enum": [
    "watching",
    "completed",
    "plan_to_watch",
    "dropped"
  ],
  "type": "string"
}
--- Anime
{
  "description": "One catalog entry.",
  "properties": {
    "id": {
      "example": 1,
      "format": "int64",
      "minimum": 0,
      "type": "integer"
    },
    "rating": {
      "description": "`1..=10`, or `null` when the show is not rated yet.",
      "format": "int32",
      "maximum": 10,
      "minimum": 1,
      "type": [
        "integer",
        "null"
      ]
    },
    "status": {
      "$ref": "#/components/schemas/WatchStatus"
    },
    "title": {
      "example": "Frieren",
      "type": "string"
    }
  },
  "required": [
    "id",
    "title",
    "status"
  ],
  "type": "object"
}
```

Read it against the Rust. The derive honours your `serde` attributes (3.3.1): `rename_all = "snake_case"` is why the enum values are `plan_to_watch`. `Option<u8>` is `["integer", "null"]` and is missing from `required`. A `u64` is an `int64` with a minimum of 0. Doc comments become `description`. `#[schema(...)]` adds what Rust's types cannot say: an example, a range.

One honest limit: `minimum = 1, maximum = 10` is *documentation*. Nothing in `utoipa` rejects a rating of 15. The check is a `if` in the handler (3.3.2 is where such rules get a proper home). If you change one and forget the other, you have drift again, just inside a single file.

### Handlers become operations: `#[utoipa::path]`

```rust
/// `GET /anime/{id}`: `200 OK` with the anime, or `404`.
#[utoipa::path(
    get,
    path = "/anime/{id}",
    params(("id" = u64, Path, description = "the anime's id")),
    responses(
        (status = 200, description = "found", body = Anime),
        (status = 404, description = "no anime has this id", body = ApiError),
    ),
    tag = "anime"
)]
pub async fn get_anime(/* extractors */) -> ApiResult<Json<Anime>> { /* ... */ }
```

The attribute does not touch the function. It sits next to it and records what *you* write in it: the method, the path (with `{id}` spelled as in `axum` 0.8), the parameters, and one entry per status code. The first line of the doc comment becomes the operation's `summary`, the rest its `description`.

Several media types for one status use `content(...)`. In `utoipa` 5 each entry is `(Schema = "media/type")`:

```rust
(status = 422, description = "rating out of range, or wrong JSON shape",
    content((ApiError = "application/json"), (String = "text/plain"))),
```

That is how `POST /anime` tells the truth about 3.2.3: *your* `422` is JSON, but the `400`, `415` and the other `422` come from `axum`'s `Json` extractor before your function runs, as `text/plain`. A contract that listed only JSON errors would be wrong. (3.8.1 makes every error one shape; until then the document says what is real.)

Where the DRF analogy stops: `drf-spectacular` introspects the view and guesses status codes and serializers, so it can be wrong silently. `utoipa` guesses nothing. The `responses(...)` list is a *promise you type*. It is never compared with the function body. Keep that sentence in mind for "Errors you will meet".

### Gathering: `#[derive(OpenApi)]` and a route

```rust
#[derive(OpenApi)]
#[openapi(
    info(title = "Anime catalog", version = "1.0.0"),
    paths(list_anime, create_anime, get_anime, delete_anime),
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
```

`paths(...)` is the list of annotated handlers; a schema reachable from them (`Anime`, `ApiError`, ...) is collected into `components` automatically. `ApiDoc::openapi()` builds the document in memory, and since it implements `Serialize`, `Json(...)` can serve it. `examples/01-print-the-document.rs` shows slices of the result:

```sh
cargo run -p p3-03-03-api-contracts-and-openapi --example 01-print-the-document
```

```text
openapi version: "3.1.0"
paths: ["/anime", "/anime/{id}"]
schemas: ["Anime", "ApiError", "CreateAnime", "WatchStatus"]
--- GET /anime/{id} responses
{
  "200": {
    "content": {
      "application/json": {
        "schema": {
          "$ref": "#/components/schemas/Anime"
        }
      }
    },
    "description": "found"
  },
  "404": {
    "content": {
      "application/json": {
        "schema": {
          "$ref": "#/components/schemas/ApiError"
        }
      }
    },
    "description": "no anime has this id"
  }
}
```

(The keys come out alphabetically because the document went through `serde_json::Value`.)

### Testing the contract

Because the document is plain JSON, a test can ask it questions: no network, no browser. The "Implement" rung has you write four small readers over a `serde_json::Value`, and the tests use them like this:

| Question | Reader | Answer for this API |
|---|---|---|
| which schemas exist? | `schema_names` | `Anime`, `ApiError`, `CreateAnime`, `WatchStatus` |
| which statuses does `POST /anime` promise? | `operation_statuses` | `201`, `400`, `415`, `422` |
| which media types can that `422` be? | `response_content_types` | `application/json`, `text/plain` |
| which routes did nobody document? | `undocumented` | none |

`tests/api_test.rs` is the other half: it sends the real requests through `oneshot` (3.2.1) and checks that the router answers exactly those statuses and content types. A contract test checks *both*: what the document says, and that the server does it.

### Two ways the contract still lies

```senpai-visual
{"kind":"result","labels":["route added to the router","not in paths(...)","document silent about it","attribute says 201","handler answers 200","document promises the wrong thing"]}
```

- **Missing from the document.** The router has a route; nobody put it in `paths(...)`. Nothing fails. `undocumented` catches it, if you give it an accurate list of routes. `axum` 0.8 gives you no way to list a router's routes, so that list (`ROUTES` here) is written by hand: a second thing to keep in step. Writing it is still much cheaper than a whole spec.
- **Promising what the body does not do.** The attribute says `201`, the function returns a bare `Json`, which is `200` (3.2.3's trap). Only a test that sends the request notices.

---

## Hands on

Run the two examples that show a lie. Neither is an error; both compile.

```sh
cargo run -p p3-03-03-api-contracts-and-openapi --example 03-forgotten-route-trap
cargo run -p p3-03-03-api-contracts-and-openapi --example 06-documented-201-answers-200-trap
```

```text
GET /ping   -> 200 OK   documented: true
GET /health -> 200 OK   documented: false
contract says POST /anime answers ["201"]
the handler answers 200 OK
```

The tests start out partly red. `src/lib.rs` has the whole API, already documented, and four `todo!()` readers:

```sh
cargo test -p p3-03-03-api-contracts-and-openapi 2>&1 | grep 'test result'
```

```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The 7 in `api_test` pass already: they test the given API against the given document. The 12 in `contract_test` are yours.

The real server (it listens on `127.0.0.1:3120`) serves its own contract. Start it, then ask it:

```sh
cargo run -p p3-03-03-api-contracts-and-openapi --example 02-serve-the-contract &
curl -si http://127.0.0.1:3120/api-docs/openapi.json | head -n 2
curl -s http://127.0.0.1:3120/api-docs/openapi.json | head -c 120
```

```text
listening on http://127.0.0.1:3120
HTTP/1.1 200 OK
content-type: application/json
{"openapi":"3.1.0","info":{"title":"Anime catalog","description":"A tiny catalog API, documented from its own code.","li
```

(The first line comes from the server; `head -c` cuts the body off without a newline.) Now check the contract's claims about `POST /anime` and `GET /anime/{id}`, each printed with its status and content type:

```sh
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"title":"Frieren","status":"watching","rating":9}' http://127.0.0.1:3120/anime
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{"title":"Bad","status":"watching","rating":15}' http://127.0.0.1:3120/anime
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -H 'content-type: application/json' -d '{' http://127.0.0.1:3120/anime
curl -s -w '\n%{http_code} %{content_type}\n' -X POST -d '{"title":"x","status":"watching"}' http://127.0.0.1:3120/anime
curl -s -w '\n%{http_code} %{content_type}\n' http://127.0.0.1:3120/anime/9
```

```text
{"id":1,"title":"Frieren","status":"watching","rating":9}
201 application/json
{"error":"rating must be between 1 and 10, got 15"}
422 application/json
Failed to parse the request body as JSON: EOF while parsing an object at line 1 column 1
400 text/plain; charset=utf-8
Expected request with `Content-Type: application/json`
415 text/plain; charset=utf-8
{"error":"anime not found"}
404 application/json
```

Every status and media type here is in the document. Stop the server (`kill %1` in the same shell). Then try:

1. Open the JSON and find `422` under `POST /anime`. Why does it list two media types?
2. Change `status = 404` to `status = 410` in `get_anime`'s attribute and run `api_test`. Does anything fail? What does that tell you about what `utoipa` checks?
3. Run `examples/01-print-the-document.rs` and look at `DELETE /anime/{id}`'s `204`. Why is there no `content`?

---

## Errors you will meet

Each broken example sits behind the `broken` feature. In the transcripts below, `cargo` first prints `unused variable` warnings for the four `todo!()` readers when you build against the skeleton. They are left out here and disappear once you implement the readers.

### `E0277` — a body type that does not derive `ToSchema`

```rust
#[derive(Serialize)]
struct Genre { name: String }

#[utoipa::path(get, path = "/genre", responses((status = 200, body = Genre)))]
async fn genre() -> Json<Genre> { /* ... */ }
```

`examples/04-missing-to-schema-broken.rs`:

```text
error[E0277]: the trait bound `Genre: ToSchema` is not satisfied
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\04-missing-to-schema-broken.rs:14:70
   |
14 | #[utoipa::path(get, path = "/genre", responses((status = 200, body = Genre)))]
   |                                                                      ^^^^^ unsatisfied trait bound
   |
help: the trait `ToSchema` is not implemented for `Genre`
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\04-missing-to-schema-broken.rs:10:1
   |
10 | struct Genre {
   | ^^^^^^^^^^^^
   = help: the following other types implement trait `ToSchema`:
             &'t [T]
             &'t mut [T]
             &str
             ()
             BTreeMap<K, T>
             BTreeSet<K>
             Box<T>
             Cow<'a, T>
           and 26 others

error[E0277]: the trait bound `Genre: PartialSchema` is not satisfied
   --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\04-missing-to-schema-broken.rs:14:70
    |
 14 | #[utoipa::path(get, path = "/genre", responses((status = 200, body = Genre)))]
    |                                                                      ^^^^^ unsatisfied trait bound
    |
help: the trait `utoipa::__dev::ComposeSchema` is not implemented for `Genre`
   --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\04-missing-to-schema-broken.rs:10:1
    |
 10 | struct Genre {
    | ^^^^^^^^^^^^
    = help: the following other types implement trait `utoipa::__dev::ComposeSchema`:
              &[T]
              &mut [T]
              &str
              BTreeMap<K, T>
              BTreeSet<K>
              Box<T>
              Cow<'a, T>
              HashMap<K, T, S>
            and 24 others
    = note: required for `Genre` to implement `PartialSchema`
note: required by a bound in `name`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\utoipa-5.5.0\src\lib.rs:374:21
    |
374 | pub trait ToSchema: PartialSchema {
    |                     ^^^^^^^^^^^^^ required by this bound in `ToSchema::name`
...
405 |     fn name() -> Cow<'static, str> {
    |        ---- required by a bound in this associated function

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-03-03-api-contracts-and-openapi` (example "04-missing-to-schema-broken") due to 4 previous errors
```

**What the compiler is objecting to:** `body = Genre` makes the macro ask `Genre` for its schema. `Genre` has `Serialize`, which tells `serde` how to write it, but nothing tells `utoipa` what it looks like in a document. The second error is the same missing impl seen through the trait `ToSchema` builds on.

**The fix:** derive it.

```rust
#[derive(Serialize, ToSchema)]
struct Genre { name: String }
```

**Why this is the fix:** `ToSchema` is the one place a type describes itself to the document. Every type you name in `body = ...`, or that one of them contains, needs it.

### `E0425` — `paths(...)` names a function with no attribute

```rust
async fn health() -> &'static str { "ok" }

#[derive(OpenApi)]
#[openapi(paths(health))]
struct Doc;
```

`examples/05-path-without-attribute-broken.rs`:

```text
error[E0425]: cannot find type `__path_health` in this scope
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\05-path-without-attribute-broken.rs:11:10
   |
11 | #[derive(OpenApi)]
   |          ^^^^^^^ not found in this scope
   |
   = note: this error originates in the derive macro `OpenApi` (in Nightly builds, run with -Z macro-backtrace for more info)

error[E0433]: cannot find module or crate `__path_health` in this scope
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\05-path-without-attribute-broken.rs:11:10
   |
11 | #[derive(OpenApi)]
   |          ^^^^^^^ use of unresolved module or unlinked crate `__path_health`
   |
   = help: if you wanted to use a crate named `__path_health`, use `cargo add __path_health` to add it to your `Cargo.toml`
   = note: this error originates in the derive macro `OpenApi` (in Nightly builds, run with -Z macro-backtrace for more info)

Some errors have detailed explanations: E0425, E0433.
For more information about an error, try `rustc --explain E0425`.
error: could not compile `p3-03-03-api-contracts-and-openapi` (example "05-path-without-attribute-broken") due to 2 previous errors
```

**What the compiler is objecting to:** `#[utoipa::path]` generates a hidden helper type named `__path_<function>` next to the handler. `paths(health)` looks for that helper. The name `__path_health` means nothing to you, but it is the clue: `health` never got the attribute.

**The fix:** put `#[utoipa::path(get, path = "/health", responses((status = 200, description = "ok")))]` on `health`.

**Why this is the fix:** `paths(...)` does not read your function, only the helper the attribute leaves behind. No attribute, no helper, no entry.

### A macro parse error — the `utoipa` 4 spelling of `content(...)`

```rust
responses((status = 200, content(("text/plain" = String))))
```

There is no error code here, because the macro rejects its own input before `rustc` has a type to complain about. `examples/07-v4-content-syntax-broken.rs`:

```text
error: expected `,`
  --> phase3-backend-foundations\03-serialization-and-validation\03-api-contracts-and-openapi\examples\07-v4-content-syntax-broken.rs:10:52
   |
10 |     responses((status = 200, content(("text/plain" = String))))
   |                                                    ^

error: could not compile `p3-03-03-api-contracts-and-openapi` (example "07-v4-content-syntax-broken") due to 1 previous error
```

**What the compiler is objecting to:** the caret is under the `=`. The macro read `"text/plain"` as the schema, expected the end of the entry, and found `=`. Examples and blog posts written for `utoipa` 4 put the media type first. In version 5 (this lesson checked 5.5.0, the one in `Cargo.lock`) the order is `(Schema = "media/type")`.

**The fix:**

```rust
responses((status = 200, content((String = "text/plain"))))
```

**Why this is the fix:** the macro's grammar for an entry is "schema, then optionally `= media type`". When an attribute fails to parse, check the crate version of whatever you copied from.

### No error at all: a route the document never mentions

```text
GET /ping   -> 200 OK   documented: true
GET /health -> 200 OK   documented: false
```

**What's actually broken:** `examples/03-forgotten-route-trap.rs` works. The route answers `200`. The document just does not know it exists, so a client generated from the document has no way to call it.

**The fix:** annotate the handler and add it to `paths(...)`, then make a test fail if that is forgotten:

```rust
assert!(undocumented(&doc, ROUTES).is_empty());
```

**Why this is the fix:** a missing entry cannot be a compile error, because the router and the document are two lists. The test compares them. It is only as good as `ROUTES`, which you keep by hand.

### No error at all: documented `201`, answered `200`

```text
contract says POST /anime answers ["201"]
the handler answers 200 OK
```

**What's actually broken:** `examples/06-documented-201-answers-200-trap.rs` has `status = 201` in the attribute and a handler that returns a bare `Json`, which is `200`. The document describes the code you *meant*.

**The fix:** make the handler match, and let a test send the request:

```rust
Ok((StatusCode::CREATED, Json(anime)))
```

**Why this is the fix:** the tuple's first element decides the status (3.2.3). `utoipa` will never read it, so `created_answers_201_with_the_new_anime` in `tests/api_test.rs` is what ties the promise to the behaviour.

---

## Exercises

### Warm up

<details>
<summary>The schema says <code>#[schema(minimum = 1, maximum = 10)]</code> on <code>rating</code>. A client sends <code>15</code>. Does <code>utoipa</code> reject it?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. The attribute only writes `minimum` and `maximum` into the document. The rejection (`422`) is the `if` in the handler. The two must be kept in step by you and by a test.

</details>

<details>
<summary>Which media types can the <code>422</code> of <code>POST /anime</code> have, and why more than one?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`application/json` and `text/plain`. The handler's own rating check answers a JSON `ApiError`. A JSON body of the wrong shape is rejected by `axum`'s `Json` extractor first, as plain text (3.2.3).

</details>

<details>
<summary>You add <code>.route("/health", get(health))</code> and forget the attribute. Compile error? What does the document say?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No compile error, and the document says nothing about `/health`. The router and `paths(...)` are separate lists.

</details>

<details>
<summary>Is <code>rating: Option&lt;u8&gt;</code> in <code>required</code>? What is its <code>type</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Not required. The type is `["integer", "null"]`, which is how OpenAPI 3.1 says "an integer or null".

</details>

<details>
<summary>Why does <code>DELETE /anime/{id}</code>'s <code>204</code> have no <code>content</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

A `204 No Content` has no body, and the attribute names no `body`, so there is nothing to describe.

</details>

### Repair

Fix all five. Build the first three with `--features broken` to see the error, then make them compile and run:

1. `examples/04-missing-to-schema-broken.rs`.
2. `examples/05-path-without-attribute-broken.rs`: add the attribute so `/health` appears in the document.
3. `examples/07-v4-content-syntax-broken.rs`.
4. `examples/03-forgotten-route-trap.rs`: make it print `documented: true` for both routes.
5. `examples/06-documented-201-answers-200-trap.rs`: keep the attribute as it is and change the handler until it answers `201 Created`.

### Implement

The four `todo!()` readers in `src/lib.rs`: `schema_names`, `operation_statuses`, `response_content_types`, `undocumented`. Each doc comment is the whole specification (sorting, case of the method, what to return for something missing).

```sh
cargo test -p p3-03-03-api-contracts-and-openapi
```

`tests/contract_test.rs` (12 tests) checks the readers on the real document and on tiny hand-made ones. `tests/api_test.rs` (7 tests, already green) checks the API against its document.

### Build

Add `PUT /anime/{id}`: a full replacement. The body is `CreateAnime`; it answers `200` with the new `Anime` (same id), `404` for an unknown id (it never creates), and `422` for a rating outside `1..=10`. Document it with `#[utoipa::path]`, add the handler to `paths(...)`, chain `.put(...)` into the route, and add `("PUT", "/anime/{id}")` to `ROUTES`. In a new `tests/put_test.rs`, send a `PUT`, and assert that the document lists `200`, `404` and `422` for it and that `undocumented(&doc, ROUTES)` is empty.

### Challenge (optional)

Add a `?status=watching` filter to `GET /anime` and document it with `#[derive(IntoParams)]` on the query struct (with `#[into_params(parameter_in = Query)]`) and `params(YourQuery)` in the path attribute. Check the generated `parameters` in a test. One thing to look for: in a scratch check, a query struct that refers to `WatchStatus` left the `$ref` pointing at a schema missing from `components` until `WatchStatus` was reachable from a body or registered with `#[openapi(components(schemas(...)))]`. Make your document self-contained. There are no provided tests.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| OpenAPI document | a JSON description of an API's paths, responses and schemas | client generators, gateways, docs |
| contract | what the API promises: statuses, media types, shapes | tests, versioning |
| `ToSchema` | derive that describes a type to the document | every request and response type |
| `#[utoipa::path]` | attribute that records one operation | every handler |
| `#[derive(OpenApi)]` | gathers paths and schemas into `ApiDoc::openapi()` | one per API |
| contract test | a test of both the document and the real answers | any public API |

### What you now know

- A document generated from the code does not go stale the way a hand-written one does, but it still only records what you typed.
- `ToSchema` follows your `serde` attributes. `#[schema(...)]` ranges are documentation, not validation.
- Each endpoint's responses are listed per status code and media type, including `axum`'s own `text/plain` rejections.
- `utoipa` 5 spells multiple media types `(Schema = "media/type")`.
- The two silent lies (undocumented route, wrong promised status) need tests, not the compiler.

### What comes back later

- **Making every error the same JSON shape, so the document can drop the `text/plain` entries**: [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
- **Real field-level rules instead of one hand-written `if`**: [3.3.2 — Validation](../02-validation/README.md)
- **Changing the contract without breaking callers**: [3.3.4 — API versioning and evolution](../04-api-versioning-and-evolution/README.md)
- **Where the catalog's storage goes next**: [3.5.3 — Anime catalog, Postgres-backed](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md)

### Can you explain?

- Why does a hand-written spec drift, and what does generating it from types fix and not fix?
- Why can `utoipa` generate from macros where `drf-spectacular` introspects at run time?
- What does `#[schema(minimum = 1)]` do, and what does it not do?
- Why does `POST /anime`'s `422` list two media types?
- Name the two mistakes `utoipa` gives no compile error for, and the test that catches each.

---

## Going further

- [`utoipa` on docs.rs](https://docs.rs/utoipa/5.5.0/utoipa/): the attribute reference for the exact version used here.
- [OpenAPI Specification 3.1.0](https://spec.openapis.org/oas/v3.1.0): the format itself.
- [`drf-spectacular`](https://drf-spectacular.readthedocs.io/): the Django counterpart, for comparing the two approaches.
- A companion crate, `utoipa-axum`, registers a route and its documentation in one call so the two lists cannot diverge. This course does not use it; check its docs for the version that matches `axum` 0.8 before you do.
