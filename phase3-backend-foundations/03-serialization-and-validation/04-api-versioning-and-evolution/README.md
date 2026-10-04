# 3.3.4 — API versioning and evolution

## At a glance

After this lesson you can:

- Sort any change to an API into "breaks a client that worked yesterday" or "does not", and defend each call, including the quiet ones (a tightened validation rule, a new enum value).
- Serve two versions of one resource from the same domain type: a `From` conversion per wire shape, a `Router` that nests `/v1` and `/v2`, and the `Accept` header as a second way to choose.
- Retire a version in public: `Deprecation`, `Sunset` and `Link` headers while it still works, `410 Gone` after.
- Write a client that survives additive changes (a tolerant reader), and read the two failures you get when it does not: a `missing field` panic and `E0004`.

**Time:** ~100 minutes · **Prerequisites:**
[3.3.1 — Serde in depth](../01-serde-depth/README.md),
[3.3.2 — Validation](../02-validation/README.md),
[3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md),
[3.2.4 — `tower::Service` / `Layer`: middleware by hand](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md),
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md)

---

## Why this matters

Everything in this module so far was about the shape of the data: how `serde` writes it (3.3.1), which inputs the server accepts (3.3.2), how the shape is written down for other people (3.3.3, the OpenAPI lesson). This lesson is about the fourth thing: the shape *changes*. A web page you can redeploy and every user gets the new one. An API has callers you cannot redeploy: a phone app a user has not updated in a year, a script a friend wrote, a partner's integration nobody remembers owning. For them the shape is a promise, and the day you rename `status` to `watch_status` is the day their app stops showing anything.

In Django you may know the two halves of this. DRF ships `URLPathVersioning` (`/v1/...`) and `AcceptHeaderVersioning` (a media type), and fills in `request.version` for you. What it cannot do is tell you *when* a change needs a new version at all. That is a judgement about the shape, and it is what this lesson trains. In `axum` there is no versioning feature to switch on: a version is a route prefix, or a header you read, and the rest is plain Rust you can test.

The lesson has the whole life of a change: decide if it breaks, ship it behind a version, announce the old one's end, switch it off. Your service stays in memory, as in 3.2.3. Module 5 is where the *stored* data meets the same problem (a database schema also changes under live traffic), and it is linked at the end.

---

## The concept

### What actually breaks a client

A client is code someone else wrote against the shape you promised. A change breaks it when something the client relied on is no longer true. Here is every kind of change this lesson names, sorted. The rule behind the table: clients are unknown, so assume they read what you send and ignore nothing they did not expect to see, and the server ignores request fields it does not know (`serde`'s default).

| Change | Breaks a client? | Why |
|---|---|---|
| add a response field | no | a client that ignores unknown fields never looks at it |
| remove a response field | **yes** | the client reads it and it is gone |
| rename a response field | **yes** | that is a remove plus an add |
| change a response field's type | **yes** | `9` and `"9"` are different things to a parser |
| add a response enum value | **yes** | an exhaustive client has no arm for `"on_hold"` |
| remove a response enum value | no | the client just never sees it |
| add an optional request field | no | old clients simply do not send it |
| add a required request field | **yes** | every old request is now invalid |
| remove a request field | no | the server ignores what old clients still send |
| tighten request validation | **yes** | yesterday's valid request is rejected today |
| loosen request validation | no | everything that was valid still is |
| add an endpoint | no | nobody calls what they do not know |
| remove an endpoint | **yes** | callers get `404` |

```senpai-visual
{"kind":"concept","labels":["add: old clients never look at it","remove / rename / retype: old clients read it and fail","tighten: valid requests become 422","a new enum value is an addition that still breaks"]}
```

Two rows are the ones people get wrong. **Adding an enum value** is an addition, but a client that matched on every value the server used to send now meets one it has no arm for. In Rust that is a compile error the next time the client is rebuilt; in a language without exhaustive matching it is a wrong or missing behaviour at run time. **Tightening validation** (3.3.2's `length(max = ...)`) changes no type and no field. The shape is identical and the contract is not. "Does it break?" is a question about everything a client could have relied on, and the JSON shape is only part of it.

### One domain type, two wire shapes

The way to keep two versions cheap is to keep neither of them as your real type. The domain type (`Anime`) has no opinion about JSON. Each wire version is its own struct with its own `serde` attributes, and a `From` conversion is the only place they meet:

```rust
pub struct AnimeV1 { // frozen: never edited again
    pub id: u64, pub title: String,
    pub status: WatchStatus, pub rating: Option<u8>,
}

pub struct AnimeV2 {
    pub id: u64, pub title: String,
    pub watch_status: WatchStatus, // renamed: this is why v2 exists
    pub rating: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub episodes: Option<u32>, // added: not breaking
}
```

`skip_serializing_if` and `default` are the attributes from 3.3.1, doing the additive part: an unknown episode count leaves the key out instead of writing `null`. `examples/01-same-anime-two-shapes.rs` converts two shows into both shapes:

```text
v1 {"id":1,"title":"Frieren","status":"watching","rating":9}
v2 {"id":1,"title":"Frieren","watch_status":"watching","rating":9,"episodes":28}
v1 {"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null}
v2 {"id":2,"title":"Dandadan","watch_status":"plan_to_watch","rating":null}
```

Notice that v2's `rating` stays `null` when unknown, because changing that would be a breaking change of its own, while `episodes` is new, so it can follow the new, tidier rule. Each `From<&Anime>` impl copies fields and renames one; there is no logic in it to get wrong. Once a version ships, its struct is frozen: you add `AnimeV3`, you do not edit `AnimeV1`. This is the same split as 3.2.3's store and its HTTP edge, one level up: the domain is stable and the representation is the part that varies.

### Strategy 1: a prefix in the URL

The simplest version selector is the path: `/v1/anime`, `/v2/anime`. In `axum` it is `Router::nest`, which mounts a whole sub-router under a prefix:

```rust
let v1_routes = Router::new()
    .route("/anime", get(v1))
    .layer(middleware::from_fn(deprecated));
let app = Router::new()
    .nest("/v1", v1_routes)
    .nest("/v2", Router::new().route("/anime", get(v2)))
    .route("/anime", get(negotiated));
```

The `.layer(...)` sits on the v1 sub-router only (3.2.4's `from_fn`), so every v1 response, even a `404`, passes through it and v2 never does. `examples/02-nest-and-negotiate.rs` sends four requests through this router with `oneshot` and prints what came back:

```text
GET /v1/anime accept=None
  deprecation=@1767225600 sunset=Thu, 31 Dec 2026 23:59:59 GMT vary=-
  {"id":1,"status":"watching"}
GET /v2/anime accept=None
  deprecation=- sunset=- vary=-
  {"episodes":28,"id":1,"watch_status":"watching"}
GET /anime accept=None
  deprecation=- sunset=- vary=accept
  {"id":1,"status":"watching"}
GET /anime accept=Some("application/vnd.anime.v2+json")
  deprecation=- sunset=- vary=accept
  {"episodes":28,"id":1,"watch_status":"watching"}
```

(The keys come out sorted because `json!` builds a map; only the example does this, the real types keep field order.) The URL version is visible in logs, in `curl`, in a browser bar, and a cache or a gateway can route on it without understanding the request. The cost: a URL is supposed to name a *resource*, and `/v1/anime/1` and `/v2/anime/1` are the same anime with two names. For a public API with many unknown callers that cost is usually worth it, which is why most of the ones you have used do this.

```senpai-visual
{"kind":"network","labels":["GET /v1/anime","nest /v1: V1 shape + Deprecation headers","GET /v2/anime","nest /v2: V2 shape","GET /anime + Accept","version_from_accept picks the shape + Vary"]}
```

### Strategy 2: the version in `Accept`

3.1.3 taught content negotiation: the client says what it can read in `Accept`, the server picks. Versioning can use the same channel with a *vendor media type*: `Accept: application/vnd.anime.v2+json` means "I want the v2 shape, as JSON". The URL stays `/anime` and names the resource once. Two rules make it correct:

- **Say what you decided.** The same URL now answers differently by header, so a cache has to know. The response carries `Vary: accept`, which tells every cache "the body depends on the `Accept` request header" (the middle line of the output above, `vary=accept`).
- **Pick a default.** A client that sends `*/*` or nothing must get something. This lesson's default is v1, the oldest contract, so nobody is moved without asking. A client that asks for an unknown version (`...v3+json`) also gets v1 here. The alternative, `406 Not Acceptable`, is stricter and is also defensible.

The cost: the version is invisible in a URL, so it is harder to try in a browser and easy to forget in a cache rule. DRF's `AcceptHeaderVersioning` is this strategy. You write the parsing yourself in the Implement rung.

### Strategy 3: additive only, and a tolerant reader

The cheapest version is the one you never create. If every change you make is in the "no" column of the table, there is nothing to version. Two things have to hold: the server only ever *adds*, and the client is a **tolerant reader** (a name from Postel's old rule, "be conservative in what you send, liberal in what you accept"): it ignores fields it does not know and defaults fields that are missing. `serde` is a tolerant reader unless you tell it not to:

```rust
#[derive(Deserialize)]
struct Tolerant {
    id: u64,
    title: String,
    #[serde(default)]
    episodes: Option<u32>,
}
```

`examples/03-tolerant-reader.rs` reads the v2 body, and a body from before `episodes` existed, with three clients. The second, `Strict`, is the same struct with `#[serde(deny_unknown_fields)]`, the attribute for readers who think strictness is a virtue:

```text
tolerant, v2 body:  Ok(Tolerant { id: 1, title: "Frieren", episodes: Some(28) })
tolerant, old body: Ok(Tolerant { id: 2, title: "Dandadan", episodes: None })
strict,   v2 body:  Err(Error("unknown field `watch_status`, expected `id` or `title`", line: 1, column: 40))
strict,   old body: Ok(Strict { id: 2, title: "Dandadan" })
reader, old name:   Ok(Reader { status: Watching })
reader, new name:   Ok(Reader { status: Watching })
reader, new value:  Ok(Reader { status: Unknown })
```

The last three lines are the third client. Its `status` field has `#[serde(alias = "watch_status")]`, so it reads either name, and its enum has a `#[serde(other)]` variant that catches any value it has never heard of. Those two attributes make a client survive a rename and a new enum value, which is the point of "Errors you will meet" below. Be honest about the limit: tolerance is a promise the *client* keeps, and you control only your server. A public API cannot assume it, which is why the first two strategies exist.

### Retiring a version: say it before you do it

A version you keep forever is a version you maintain forever. Getting rid of one is a three-step conversation, and the first two steps are headers on a response that still works:

```senpai-visual
{"kind":"concept","labels":["1. Deprecation header: it still works, plan to leave","2. Sunset header: the date it stops, Link: where to go","3. after the date: 410 Gone, with the same Link","a client that logs headers sees it coming"]}
```

- **`Deprecation: @1767225600`** (RFC 9745). The `@` followed by Unix seconds is that RFC's date format. It says "this was marked deprecated at that moment". 1767225600 is 2026-01-01T00:00:00Z.
- **`Sunset: Thu, 31 Dec 2026 23:59:59 GMT`** (RFC 8594). An HTTP-date: the moment the resource is expected to stop answering.
- **`Link: </v2/anime>; rel="successor-version"`** (RFC 5829). Where the replacement lives.

After the sunset moment the answer is `410 Gone`, not `404`. `404` means "I do not know it, maybe it never existed or may come back"; `410` means "it was here, on purpose, and is gone for good". It still carries the `Link` so a stuck client has a next step. These are *machine-readable* notices: a well-behaved client or an API gateway can alert on them long before anybody reads an email. Which, being honest, is the other half: headers do not replace telling the owners.

### Which strategy when

| | URL prefix | `Accept` media type | Additive only |
|---|---|---|---|
| visible in logs and `curl` | yes | no, a header | n/a |
| one URL per resource | no | yes | yes |
| cache needs | none | `Vary: accept` | none |
| works for unknown callers | yes | yes | only if they are tolerant |
| cost per version | a second route tree | a second route tree + parsing | none, until a change breaks |

They combine. A common shape, and this lesson's router, is additive-only as the default, a URL version when something has to break, and `Accept` available as the second door. Whichever you pick, the decision "does this change need a new version?" is the same one: the table at the top.

---

## Hands on

Run the four examples that compile. (`04` and `05` are broken on purpose and sit behind the `broken` feature; "Errors you will meet" shows them. `06` compiles and is simply wrong.)

```sh
cargo run -p p3-03-04-api-versioning-and-evolution --example 01-same-anime-two-shapes
cargo run -p p3-03-04-api-versioning-and-evolution --example 02-nest-and-negotiate
cargo run -p p3-03-04-api-versioning-and-evolution --example 03-tolerant-reader
cargo run -p p3-03-04-api-versioning-and-evolution --example 06-tightened-validation-silent
```

```text
61 chars
v1 accepts: true
v2 accepts: false
```

(The first three print the blocks in "The concept"; this is the last.)

The tests start out red. `src/lib.rs` has the whole skeleton, and every function you implement is a `todo!()` with a doc comment that is its full specification:

```sh
cargo test -p p3-03-04-api-versioning-and-evolution --no-fail-fast 2>&1 | grep 'test result: FAILED'
```

```text
test result: FAILED. 0 passed; 7 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

When everything is green, run the real server (it listens on `127.0.0.1:3130`). These transcripts were captured against the solution; `grep -v '^date'` drops the one line that changes every second. First a v1 request, then v2:

```sh
cargo run -p p3-03-04-api-versioning-and-evolution &
curl -si http://127.0.0.1:3130/v1/anime/1 | grep -v '^date'
curl -si http://127.0.0.1:3130/v2/anime/1 | grep -v '^date'
```

```text
HTTP/1.1 200 OK
content-type: application/json
deprecation: @1767225600
sunset: Thu, 31 Dec 2026 23:59:59 GMT
link: </v2/anime>; rel="successor-version"
content-length: 57

{"id":1,"title":"Frieren","status":"watching","rating":9}
HTTP/1.1 200 OK
content-type: application/json
content-length: 77

{"id":1,"title":"Frieren","watch_status":"watching","rating":9,"episodes":28}
```

The v1 body is the frozen shape and its headers announce the end of v1; v2 has neither. Now the unversioned route, once without and once with the media type, and a missing id on v1:

```sh
curl -si -H 'accept: application/vnd.anime.v2+json' http://127.0.0.1:3130/anime/2 | grep -v '^date'
curl -si http://127.0.0.1:3130/anime/2 | grep -v '^date'
curl -si http://127.0.0.1:3130/v1/anime/99 | grep -v '^date'
```

```text
HTTP/1.1 200 OK
content-type: application/json
vary: accept
content-length: 72

{"id":2,"title":"Dandadan","watch_status":"plan_to_watch","rating":null}
HTTP/1.1 200 OK
content-type: application/json
vary: accept
content-length: 66

{"id":2,"title":"Dandadan","status":"plan_to_watch","rating":null}
HTTP/1.1 404 Not Found
content-type: application/json
deprecation: @1767225600
sunset: Thu, 31 Dec 2026 23:59:59 GMT
link: </v2/anime>; rel="successor-version"
content-length: 27

{"error":"anime not found"}
```

Both `/anime/2` answers say `vary: accept`. The `404` still carries the three deprecation headers, because the middleware wraps the whole v1 sub-router, not just the handlers. Stop the server when you are done (`kill %1` in the same shell). Then try these:

1. Ask `/anime/1` with `Accept: application/vnd.anime.v3+json`. Which shape do you get, and which line of `version_from_accept`'s spec says so?
2. Add a `TightenRequestValidation` to `requires_new_version`'s input in a test. Does the answer change? What does that say about "the JSON shape is identical"?
3. In `examples/06-tightened-validation-silent.rs`, change the v2 limit to `200`. Which line of the output changes?

---

## Errors you will meet

### A run-time panic: `missing field`

This is the break the table called "rename a response field", seen from the client. `examples/04-renamed-field-breaks-client-broken.rs` is a client written against v1 reading a v2 body:

```rust
#[derive(Debug, Deserialize)]
struct AnimeV1 {
    id: u64,
    status: String,
}
// body: {"id":1,"watch_status":"watching","episodes":28}
let anime: AnimeV1 = serde_json::from_str(body).unwrap();
```

```text
thread 'main' (41792) panicked at phase3-backend-foundations\03-serialization-and-validation\04-api-versioning-and-evolution\examples\04-renamed-field-breaks-client-broken.rs:20:53:
called `Result::unwrap()` on an `Err` value: Error("missing field `status`", line: 1, column: 48)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses is the thread id and changes on every run.)

**What the compiler is objecting to:** nothing. There is no compile error: the client compiles, and the break happens when the data arrives. `serde` looked for `status`, found `watch_status`, and a required field is not optional. Nothing in this client changed; the contract under it did.

**The fix:** two halves, one for each side. On the server, never rename inside a shipped version: that is the reason `/v2` exists. On the client, be tolerant: `#[serde(alias = "watch_status")]` on `status` reads either name, as the third client in `examples/03-tolerant-reader.rs` does.

**Why this is the fix:** a deserializer fails on a missing required field because that is the safe default, so the only way to stop it failing is to change what it was told to expect (`alias`, or `Option` with `default`) or change what it is sent (a new version). A `.unwrap()` on a network response turns a contract change into a crash; in a service, handle the `Err`.

### `E0004` — non-exhaustive patterns

The enum row of the table, from a Rust client's side. `examples/05-new-enum-value-broken.rs` has a client that matched every `WatchStatus` the server used to send, after the type grew an `OnHold`:

```text
error[E0004]: non-exhaustive patterns: `&WatchStatus::OnHold` not covered
  --> phase3-backend-foundations\03-serialization-and-validation\04-api-versioning-and-evolution\examples\05-new-enum-value-broken.rs:20:11
   |
20 |     match s {
   |           ^ pattern `&WatchStatus::OnHold` not covered
   |
note: `WatchStatus` defined here
  --> phase3-backend-foundations\03-serialization-and-validation\04-api-versioning-and-evolution\examples\05-new-enum-value-broken.rs:11:6
   |
11 | enum WatchStatus {
   |      ^^^^^^^^^^^
...
16 |     OnHold, // added in a "minor" release
   |     ------ not covered
   = note: the matched value is of type `&WatchStatus`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern or an explicit pattern as shown
   |
24 ~         WatchStatus::Dropped => "dropped",
25 ~         &WatchStatus::OnHold => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
error: could not compile `p3-03-04-api-versioning-and-evolution` (example "05-new-enum-value-broken") due to 1 previous error
```

**What the compiler is objecting to:** `match` must cover every value of the type, and the type now has one more. The error names the missing case and shows the line that defines it. This is the good outcome: a Rust client finds out at build time. A client in a language without exhaustive matching finds out when the first `"on_hold"` arrives in production.

**The fix:** decide what the client does with a value it does not know, and say it in the match. A wildcard arm (`_ => "unknown"`) is the tolerant answer. When the enum is *read from JSON*, the same decision goes on the type: a `#[serde(other)]` variant, as in the third client of `03`, so an unknown string becomes `Unknown` instead of a parse error.

**Why this is the fix:** the table said that adding an enum value breaks exhaustive clients. A catch-all arm makes the client non-exhaustive on purpose, so a future addition is something it handles instead of something that breaks it. You trade the compiler's reminder for resilience, and that is a decision, not an accident. The server-side lesson: put new enum values behind a version, or document "clients must tolerate unknown values" from day one.

### No error at all: yesterday's request is rejected

```text
61 chars
v1 accepts: true
v2 accepts: false
```

**What's actually broken:** `examples/06-tightened-validation-silent.rs` compiles, runs, and the types are identical. The only change is `max = 200` to `max = 50` on `title`. A 61-character title was a valid request before and is a `422` now. No struct changed, so no diff of the types, no OpenAPI field list, and no compiler catches it. The table's "tighten request validation" row is the one a code review misses most, because it looks like a cleanup.

**The fix:** a tightened rule is a breaking change, so it ships behind a new version (`CreateAnimeV2` with `max = 50`, `CreateAnimeV1` keeps `200`), or you leave the limit alone.

**Why this is the fix:** the contract is every request the server accepts, not only the field names. Only a test that replays an old, valid request against the new server catches this class, which is why `classify` has `TightenRequestValidation` as its own variant: it is a change you can make with no type changing at all.

---

## Exercises

### Warm up

<details>
<summary>The server starts sending a new field, <code>"studio"</code>, in every response. Does that break a client?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No, if the client is a tolerant reader: it never looks at `studio`. It does break a client that used `deny_unknown_fields`, which is why "additive changes are safe" is a statement about clients you have to be able to assume. For an API with unknown callers, you assume it and say so in the docs.

</details>

<details>
<summary>The server starts sending <code>"on_hold"</code> as a new <code>status</code> value. Additive, so safe?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. It is an addition to a field's set of values, and a client with an exhaustive `match` (or a typed enum with no catch-all) has no case for it. It is a breaking change for those clients, which is why the table lists "add a response enum value" as breaking.

</details>

<details>
<summary>A <code>PATCH</code> endpoint now requires a <code>reason</code> field. Which row of the table, and what do old clients see?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

"Add a required request field": breaking. An old client's `PATCH` has no `reason`, so every request it sends is now invalid and answers `422`. The compatible way is `reason` as optional, with a default.

</details>

<details>
<summary>Why does an <code>Accept</code>-versioned route need <code>Vary: accept</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

A cache stores a response under the URL. `/anime/1` answers with two different bodies depending on a header, and without `Vary: accept` a cache can hand the v2 body to a v1 client that asked for the same URL. `Vary` tells the cache which request headers are part of the key.

</details>

<details>
<summary>Why <code>410 Gone</code> after the sunset date and not <code>404</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`404` says the server does not know the resource, which could be a typo or a temporary state. `410` says it existed and was removed on purpose and permanently. A client or a gateway can treat the two very differently: retry a `404` later, stop calling a `410`.

</details>

### Repair

Fix both broken examples, and the one that is silently wrong:

1. `examples/04-renamed-field-breaks-client-broken.rs` compiles and panics. Make the client read the v2 body without panicking and without changing the body string. Run it with `--features broken`.
2. `examples/05-new-enum-value-broken.rs` does not compile. Make it compile and print something sensible for `OnHold` too. Do not just add `todo!()`, which is what the compiler's help suggests.
3. `examples/06-tightened-validation-silent.rs` prints `v2 accepts: false`. Without touching `CreateAnimeV1`, make both lines say `true` for the 61-character title, and say in a comment which kind of change you just undid.

### Implement

Everything in `src/lib.rs` that is a `todo!()` except the last two functions: the two `From` impls, `classify`, `requires_new_version`, `deprecation_headers` and `version_from_accept`. Each doc comment is the complete specification (the exact JSON, the whole classification table, the header names and their order, how `Accept` is parsed), so you should never need to open the tests. The router `app` and its handlers are given.

```sh
cargo test -p p3-03-04-api-versioning-and-evolution
```

`tests/model_test.rs` (8 tests) checks the pure logic with plain calls. `tests/api_test.rs` (7 tests) checks the router you were given, through `tower::ServiceExt::oneshot`, and passes only when your conversions, headers and negotiation are right.

### Build

`app_after_sunset`: the router for the day after v1's sunset date. Every `/v1/...` path, including ones that never existed, answers `410 Gone` with the `link` header and the body `{"error":"v1 was retired; use /v2"}`. `/v2` and the unversioned routes stay as in `app`. Its doc comment has the full spec. The hint is in the lesson: a sub-router can have a `fallback`, and you have already seen a `Router` nested under a prefix. Write your own tests in a new `tests/sunset_test.rs`, including one for a path that was never a v1 route.

### Challenge (optional)

`diff_shapes(old, new)`: compare two example JSON responses and list the changes, using the same `Change` enum, so `requires_new_version(&diff_shapes(&old, &new))` is a release gate. The doc comment has the exact rules: removed keys, changed JSON kinds, added keys, and what a rename looks like. This is where the lesson points next. 3.3.3's generated OpenAPI document is exactly the kind of machine-readable shape this check could run on, instead of two hand-pasted samples. A tool that diffs two such documents in CI is the same idea with real schemas. There are no provided tests: write three, including a rename.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Breaking change | a change after which a client that worked before may fail | every release of a public API |
| Additive change | a change that only adds (a field, an endpoint, an optional input) | the default way to evolve |
| URL versioning | the version is a path prefix (`/v1/...`), mounted with `Router::nest` | public APIs with unknown callers |
| Media-type versioning | the version is in `Accept` (`application/vnd.anime.v2+json`) | one URL per resource; needs `Vary: accept` |
| Tolerant reader | a client that ignores unknown fields and defaults missing ones | every client of an API you do not control |
| `Deprecation` / `Sunset` / `Link` | headers that announce a version's end, its date and its successor | before every retirement |
| `410 Gone` | the resource existed and was removed on purpose | after a sunset date |

### What you now know

- A change breaks a client when something the client relied on stops being true. That includes request rules (`tighten validation`) and enum sets (`add a value`), not only field names.
- Keep one domain type and one frozen struct per wire version, with a `From` conversion between them.
- Three ways to version: a URL prefix with `nest`, a vendor media type in `Accept` (with `Vary: accept` and a default), or additive-only changes with tolerant clients. They combine.
- Retire a version in two stages: `Deprecation`, `Sunset` and `Link` while it still answers, `410 Gone` after.
- A client can protect itself with `serde(default)`, `alias` and `other`, and the server can protect clients by never changing a shipped version.

### What comes back later

- **A schema that changes under live data, not just on the wire**: [3.5.2 — Migrations](../../05-postgres-and-sqlx/02-migrations/README.md)
- **The same catalog on a database, where a "version" also means a table shape**: [3.5.3 — Anime catalog, Postgres-backed](../../05-postgres-and-sqlx/03-anime-catalog-postgres-backed/README.md)
- **Configuration that picks the sunset date and the default version without a rebuild**: [3.4.1 — 12-factor config and secrets](../../04-configuration-and-app-structure/01-config-and-secrets/README.md)
- **Where the shared store and the version constants live in a real application**: [3.4.2 — Application state and dependency wiring](../../04-configuration-and-app-structure/02-app-state-and-dependency-wiring/README.md)
- **The shape of a contract, written down for other people**: [3.3.3 — API contracts and OpenAPI (`utoipa`)](../03-api-contracts-and-openapi/README.md)
- **One error format for the `422`s and `404`s a retired or tightened API produces**: [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)

### Can you explain?

- Why is adding an enum value a breaking change when adding a field is not?
- Why is tightening a validation rule breaking when no type changed?
- Why keep a `From` conversion between a domain type and each wire version instead of one struct that serves both?
- When is `Accept`-header versioning a better fit than a URL prefix, and what does it force you to add to the response?
- What does a tolerant reader do, and why can your server not rely on it?
- What are `Deprecation`, `Sunset` and `Link` for, and why is the final answer `410` and not `404`?
- Across 3.3.1 to 3.3.4: which of the four lessons decides what the server writes, which decides what it accepts, which describes it for others, and which decides when it is allowed to change?

### Looking back at module 3.3, and ahead

Four lessons, one thread. 3.3.1 gave you the attributes that decide what a type looks like on the wire. 3.3.2 added the separate pass that decides which inputs are acceptable at all. 3.3.3 turned both into a document other people can read, so the shape is written down instead of remembered. This lesson is what happens to all of it over time: the shape stops being yours to change freely the moment someone else depends on it. Module 3.4 takes the service you now have and gives it what a real deployment needs before it touches a database: configuration, shared application state, and graceful shutdown with health and readiness checks. Module 3.5 then replaces the in-memory store with PostgreSQL, where the same question, "can I change this without breaking what is already there?", returns for stored data as migrations.

---

## Going further

- [RFC 9745 — The Deprecation HTTP Response Header Field](https://www.rfc-editor.org/rfc/rfc9745.html): the `@<seconds>` date format and what `Deprecation` promises.
- [RFC 8594 — The Sunset HTTP Header Field](https://www.rfc-editor.org/rfc/rfc8594.html): the `Sunset` header and its HTTP-date value.
- [RFC 5829 — Link Relations for Simple Version Navigation](https://www.rfc-editor.org/rfc/rfc5829.html): `successor-version` and its siblings.
- [RFC 9110 — HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110.html): §12 (content negotiation, `Accept`), §15.5.11 (`410 Gone`), §12.5.5 (`Vary`).
- [Serde field attributes](https://serde.rs/field-attrs.html) and [container attributes](https://serde.rs/container-attrs.html): `default`, `alias`, `skip_serializing_if`, and `deny_unknown_fields`.
- [`Router::nest` in the `axum` docs](https://docs.rs/axum/0.8.9/axum/struct.Router.html#method.nest): how a prefix is stripped before the inner router sees the path.
- [DRF — Versioning](https://www.django-rest-framework.org/api-guide/versioning/): the same strategies, as classes.
