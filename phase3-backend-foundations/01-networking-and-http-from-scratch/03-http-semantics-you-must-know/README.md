# 3.1.3 — HTTP semantics you must know

## At a glance

After this lesson you can:

- Say which HTTP methods are *safe* and which are *idempotent*, and explain why those are two separate promises, not one.
- Classify any status code by its first digit, and pick the right one of the pairs people mix up (`401`/`403`, `400`/`422`, `302`/`307`, `502`/`503`).
- Pick a response format from a client's `Accept` header the way the spec says to: `q` weights, wildcards, and specificity.
- Explain why a missing `Connection` header means "keep the connection open" in HTTP/1.1 but "close it" in HTTP/1.0.
- Decode a chunked body by hand, and say why chunked encoding exists at all.

**Time:** ~70 minutes · **Prerequisites:**
[3.1.2 — Hand-rolled HTTP parser](../02-hand-rolled-http-parser/README.md)

---

## Why this matters

[3.1.2](../02-hand-rolled-http-parser/README.md) was about **syntax**: where the `\r\n` goes, what a request line looks like, how a header line splits on `:`. This lesson is about **semantics**: what the parts *mean*, and what the other side is allowed to assume because of them.

You have used all of this already, mostly without seeing it. DRF's `status.HTTP_201_CREATED` is a status-code choice. `requests` retries a timed-out `GET` on its own but never a `POST`, and that is idempotency at work. When a browser gets HTML from a URL and `curl` gets JSON from the same one, that is content negotiation. When gunicorn reuses one connection for many requests, that is keep-alive. A streaming Django response goes out in chunks. From the next module on, `axum` handles all of this for you, and you still have to choose the right status code and the right method yourself. This lesson is the checklist you choose from.

---

## The concept

### Safe and idempotent: two separate promises

The spec (RFC 9110 §9.2) gives each method two separate properties:

- **Safe**: calling it is not expected to change anything on the server. A crawler, a link prefetcher, or a cache can call a safe method whenever it likes.
- **Idempotent**: calling it N times leaves the server in the same state as calling it once. A client that never got a reply can *resend* an idempotent request without asking whether the first one arrived.

| Method | Safe | Idempotent |
|---|---|---|
| `GET`, `HEAD`, `OPTIONS`, `TRACE` | yes | yes |
| `PUT`, `DELETE` | no | yes |
| `POST`, `PATCH`, `CONNECT` | no | no |

Every safe method is also idempotent: if nothing changes, calling it twice changes nothing either. The reverse is false. `DELETE /anime/7` definitely changes the server, but doing it three times leaves the same result as doing it once: anime 7 is gone.

`examples/01-idempotency-in-practice.rs` retries a request three times, the way a client does after a timeout:

```text
after 3x PUT /balance 500:   balance = 500, deposits_applied = 0
after 3x POST /deposit 500:  balance = 1500, deposits_applied = 3
```

`PUT` *sets* the balance, so a retry costs nothing. `POST` *adds* to it, so each retry is another deposit. This is why an HTTP client retries idempotent methods automatically and leaves `POST` alone. It is also why a payment API that takes `POST` asks you for an idempotency key; you will build that in Phase 5.

```senpai-visual
{"kind":"concept","labels":["safe: the server is not changed","idempotent: N calls leave the same state as 1","every safe method is idempotent","not every idempotent method is safe: PUT, DELETE","POST and PATCH are neither"]}
```

These are promises the spec makes on your behalf. If you write a `GET /anime/7/like` handler that increments a counter, nothing stops you. But every crawler and prefetcher that reads that URL now "likes" things, because they were told `GET` is safe.

### Status codes: the first digit is the contract

Every status code falls into one of five classes, fixed by its first digit:

| Class | Meaning | Examples |
|---|---|---|
| `1xx` | informational: keep going | `101 Switching Protocols` |
| `2xx` | success | `200 OK`, `201 Created`, `204 No Content` |
| `3xx` | redirection: look elsewhere | `301`, `302`, `307`, `308` |
| `4xx` | client error: fix the request | `400`, `401`, `403`, `404`, `409`, `422`, `429` |
| `5xx` | server error: not your fault | `500`, `502`, `503` |

The class matters more than you'd think. A client that has never heard of `418` still knows it's a client error, because the spec says an unknown code is treated like the `x00` of its class. The pairs people mix up:

- **`401` vs `403`.** `401 Unauthorized` really means *unauthenticated*: "I don't know who you are." `403 Forbidden` means "I know who you are, and you still can't."
- **`400` vs `422`.** `400` means the request itself is malformed (broken JSON, say). `422` means it parsed fine but the content is invalid, like an `email` field with no `@`.
- **`301`/`302` vs `308`/`307`.** For historical reasons, clients may turn a redirected `POST` into a `GET` on `301`/`302`. `307` and `308` forbid that: same method, same body.
- **`502` vs `503`.** `502 Bad Gateway` means a proxy got a bad answer from the server behind it. `503 Service Unavailable` means the server itself is overloaded or in maintenance, and will be back.

`examples/02-status-line-and-reason-phrases.rs` prints the actual wire text for each of these. It's the same `HTTP/1.1 {code} {reason}\r\n` status line you built in 3.1.2.

### Content negotiation: `Accept` and `q` weights

A client lists the formats it will accept, and how much it wants each one:

```text
Accept: text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8
```

Each comma-separated entry is a media type, optionally followed by `;q=` and a weight from `0` to `1`. No `q` means `1`. `examples/03-parsing-accept-header-pieces.rs` splits this real Firefox header into pieces:

```text
text/html                q=1
application/xhtml+xml    q=1
application/xml          q=0.9
*/*                      q=0.8
```

Then the server picks one of the formats *it* can produce. Three rules decide which (RFC 9110 §12.5.1):

1. **Wildcards.** An entry matches a type exactly (`text/html`), by its first half (`text/*`), or matches everything (`*/*`).
2. **The most specific match wins, whatever its `q`.** If `text/html;q=0.1` and `*/*;q=0.9` both match `text/html`, its weight is `0.1`. The client named it specifically, and that's the answer it gave.
3. **`q=0` means "not acceptable."** It does not mean "lowest priority." `text/html;q=0,*/*` means "anything except HTML."

```senpai-visual
{"kind":"concept","labels":["server can produce: application/json, text/html","client sends: text/html;q=0.1, */*;q=0.9","text/html: exact match wins, weight 0.1","application/json: only */* matches, weight 0.9","server replies with application/json"]}
```

When two formats end up with the same weight, the server picks between them by its own preference. DRF does all of this when it chooses between `JSONRenderer` and `BrowsableAPIRenderer`, and you will write the same logic yourself in "Implement".

### Keep-alive: the same header, opposite defaults

Opening a TCP connection costs a round trip before a single byte of HTTP goes over it (with TLS, more). Reusing one connection for many requests saves that cost. HTTP/1.0 and HTTP/1.1 treat a missing `Connection` header in *opposite* ways (RFC 9112 §9.3):

```text
HTTP/1.1  Connection: (absent)     -> keep-alive = true
HTTP/1.1  Connection: close        -> keep-alive = false
HTTP/1.1  Connection: keep-alive   -> keep-alive = true
HTTP/1.0  Connection: (absent)     -> keep-alive = false
HTTP/1.0  Connection: keep-alive   -> keep-alive = true
HTTP/1.0  Connection: close        -> keep-alive = false
```

That's the output of `examples/04-keep-alive-defaults.rs`. In HTTP/1.1, silence means "stay open", and a client has to *opt out* with `close`. In HTTP/1.0, silence means "close", and a client has to *opt in* with `keep-alive`. A server that gets this backwards compiles and runs, and is still wrong; "Errors you will meet" shows that exact trap.

This also explains 3.1.2's rule that `Content-Length` must be exact. On a connection that stays open, "the connection closed" can no longer mark the end of a body. Something else has to.

### Chunked encoding: a body with no length up front

`Content-Length` has to go out *before* the body. So what does a server do when it doesn't know the length yet, say because it's still generating the output? It sends `Transfer-Encoding: chunked` instead, and splits the body into pieces. Each piece is its size in hexadecimal, `\r\n`, the data itself, then `\r\n`. A zero-size chunk ends the body (RFC 9112 §7.1). `examples/05-chunked-encoding-by-hand.rs` builds one and shows every `\r\n`:

```text
4\r\n
Wiki\r\n
5\r\n
pedia\r\n
E\r\n
 in\r\n
\r\n
chunks.\r\n
0\r\n
\r\n
total wire bytes: 43
```

Look at the third chunk: `E` is hexadecimal for 14, and its data (` in\r\n\r\nchunks.`) has two `\r\n` pairs *inside* it. A decoder that scanned for the next `\r\n` to find where the data ends would cut this chunk in the wrong place. The size line exists so the decoder counts bytes and doesn't have to guess.

---

## Hands on

```sh
cargo run -p p3-01-03-http-semantics-you-must-know --example 01-idempotency-in-practice
cargo run -p p3-01-03-http-semantics-you-must-know --example 02-status-line-and-reason-phrases
cargo run -p p3-01-03-http-semantics-you-must-know --example 03-parsing-accept-header-pieces
cargo run -p p3-01-03-http-semantics-you-must-know --example 04-keep-alive-defaults
cargo run -p p3-01-03-http-semantics-you-must-know --example 05-chunked-encoding-by-hand
```

Then the three broken ones. Two need the `broken` feature; the third runs normally and is simply wrong:

```sh
cargo build -p p3-01-03-http-semantics-you-must-know --example 06-status-class-non-exhaustive-broken --features broken
cargo run -p p3-01-03-http-semantics-you-must-know --example 07-accept-q-value-panic-broken --features broken
cargo run -p p3-01-03-http-semantics-you-must-know --example 08-keep-alive-default-reversed-trap
```

Then try these:

1. In `01-idempotency-in-practice`, add a `delete_account` method that sets the balance to `0`, and retry it three times. Is it idempotent?
2. In `03-parsing-accept-header-pieces`, replace the Firefox header with `curl`'s default, `*/*`. How many entries come out?
3. In `05-chunked-encoding-by-hand`, add a chunk whose data is the Persian word `"سلام"`. What size line does it get, and why isn't it `4`?

---

## Errors you will meet

### `E0004` — a status-code `match` that isn't exhaustive

```text
error[E0004]: non-exhaustive patterns: `0_u16..=99_u16` and `600_u16..=u16::MAX` not covered
  --> phase3-backend-foundations\01-networking-and-http-from-scratch\03-http-semantics-you-must-know\examples\06-status-class-non-exhaustive-broken.rs:10:11
   |
10 |     match status {
   |           ^^^^^^ patterns `0_u16..=99_u16` and `600_u16..=u16::MAX` not covered
   |
   = note: the matched value is of type `u16`
help: ensure that all possible cases are being handled by adding a match arm with a wildcard pattern, a match arm with multiple or-patterns as shown, or multiple match arms
   |
15 ~         500..=599 => "ServerError",
16 ~         0_u16..=99_u16 | 600_u16..=u16::MAX => todo!(),
   |

For more information about this error, try `rustc --explain E0004`.
```

**What the compiler is objecting to:** the five ranges `100..=599` cover every real status code, so the `match` looks complete. But `status` is a `u16`, which holds anything from `0` to `65535`, and the compiler checks the *type* rather than the spec. `0..=99` and `600..` are still uncovered.

**The fix:** don't copy the suggested `todo!()`, since that just moves the problem to run time. Make the function return `Option<&'static str>` and add a real arm for everything else:

```rust
_ => None,
```

**Why this is the fix:** a `u16` that isn't a status code is a real possibility; any client can send `HTTP/1.1 999 Nope`. `None` is how the type says so honestly. That's why your own `status_class` returns `Option<StatusClass>`.

### A run-time panic: trusting a `q` value to parse

```text
thread 'main' (16568) panicked at phase3-backend-foundations\01-networking-and-http-from-scratch\03-http-semantics-you-must-know\examples\07-accept-q-value-panic-broken.rs:12:59:
called `Result::unwrap()` on an `Err` value: ParseFloatError { kind: Invalid }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses is the thread's id and changes every run.)

**What's actually broken:** `weight_of` calls `.unwrap()` on `"high".parse::<f64>()`. A header is text a stranger sent you, the same untrusted input 3.1.2 validated before touching it. One badly configured client can now bring down the request handler.

**The fix:** decide what a bad `q` means instead of crashing on it. This lesson's rule is "treat it as `1.0`":

```rust
Some((_, weight)) => weight.trim().parse().unwrap_or(1.0),
```

**Why this is the fix:** `unwrap_or` turns the `Err` into a value you picked in advance. Being lenient here is a choice (rejecting the whole header would be another reasonable one), but a panic is never the right answer to bad input from the network.

### No error at all: keep-alive defaults reversed

```text
HTTP/1.1, no Connection header -> keep-alive = false  (should be true)
HTTP/1.0, no Connection header -> keep-alive = true  (should be false)
```

**What's actually broken:** `examples/08-keep-alive-default-reversed-trap.rs` compiles, runs, and returns a `bool` every time. The two versions' defaults are just swapped. Every HTTP/1.1 client pays for a fresh connection per request. Every old HTTP/1.0 client waits on a connection the server won't close, because it expects the server to close it first.

**The fix:** swap the two arms back. HTTP/1.1 stays open unless it sees `close`; HTTP/1.0 stays open only if it sees `keep-alive`:

```rust
"HTTP/1.1" => !says("close"),
"HTTP/1.0" => says("keep-alive"),
```

**Why this is the fix:** the defaults are in the spec, not something you choose. No compiler catches this kind of bug. Only a test that checks the *absent* header for each version does. Get used to testing what happens when nothing is sent.

---

## Exercises

### Warm up

<details>
<summary>Is <code>DELETE</code> safe? Is it idempotent?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Not safe, since it changes the server. Idempotent, because deleting the same thing a second time leaves the same result: it's gone. (The *response* can differ, `204` the first time and `404` after that. Idempotency is about the server's state, not the reply.)

</details>

<details>
<summary>A logged-in user asks for another user's private settings. <code>401</code> or <code>403</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`403 Forbidden`. The server knows who they are, and they still aren't allowed. `401` is for "I don't know who you are": no token, or an expired one.

</details>

<details>
<summary>Which format does a server that can produce <code>text/html</code> and <code>application/json</code> pick for <code>Accept: text/html;q=0, */*</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`application/json`. `q=0` makes `text/html` unacceptable, and the more general `*/*` can't bring it back. Only `application/json` is left.

</details>

<details>
<summary>An HTTP/1.0 request arrives with no <code>Connection</code> header. Does the server keep the connection open after replying?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

No. HTTP/1.0 closes by default unless the client asks for `Connection: keep-alive`. HTTP/1.1 does the opposite.

</details>

### Repair

Fix all three broken examples:

1. `examples/06-status-class-non-exhaustive-broken.rs` compiles, without a `todo!()`.
2. `examples/07-accept-q-value-panic-broken.rs` prints a weight instead of panicking.
3. `examples/08-keep-alive-default-reversed-trap.rs` prints `true` for HTTP/1.1 and `false` for HTTP/1.0.

### Implement

Four functions in `src/lib.rs`:

```sh
cargo test -p p3-01-03-http-semantics-you-must-know
```

Each one is fully specified in its own doc comment, so you should never need to read the tests to know what to build:

- `is_safe` and `is_idempotent`: the two properties from the table above.
- `status_class`: a `u16` to its class, or `None` if it isn't a status code.
- `best_content_type`: the negotiation rules above, as code. Wildcards, most-specific-wins, `q=0`, and ties broken by the server's own order.

### Build

`decode_chunked` in the same file. It turns a chunked body back into its original bytes, and reports a bad size line (`InvalidLength`) or a body that ends too early (`UnexpectedEnd`) as a proper error. The five `chunked_decode_tests` in the same `cargo test` run check it.

### Challenge (optional)

Real `Accept` headers are messier than this lesson's rule allows. There can be spaces around the `;` (`text/html ; q=0.5`), and parameters other than `q` (`text/html;level=1;q=0.5`). Make `best_content_type` handle both, and add a test for each case to `src/lib.rs`. The existing tests must still pass.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Safe method | no server state is expected to change | deciding what a `GET` handler is allowed to do |
| Idempotent method | N calls leave the same state as one | deciding what's safe to retry |
| Status class | the meaning fixed by a code's first digit | picking the right code, handling one you don't know |
| Content negotiation | picking a format from the client's `Accept` | APIs that serve more than one format |
| `q` value | a weight from `0` to `1`; `0` means "not acceptable" | reading any negotiable header |
| Keep-alive | reusing one TCP connection for many requests | every HTTP/1.1 server, from `axum` down |
| Chunked encoding | sizing a body chunk by chunk, not all up front | streaming responses |

### What you now know

- Safe and idempotent are two separate promises. `PUT` and `DELETE` are idempotent without being safe, and `POST` is neither.
- A status code's first digit is its contract, and the codes in the commonly confused pairs mean different things.
- In content negotiation, the most specific match decides a type's weight, and `q=0` excludes it outright.
- HTTP/1.1 keeps connections open by default and HTTP/1.0 closes them. Same header, opposite defaults.
- Chunked encoding exists because `Content-Length` has to be known up front. The size line is there so a decoder counts bytes instead of scanning for `\r\n`.

### What comes back later

- **Choosing and returning status codes from real handlers** — [3.2.3 — Anime catalog CRUD (in-memory)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.md)
- **Documenting which formats and status codes an endpoint returns** — [3.3.3 — API contracts and OpenAPI (`utoipa`)](../../03-serialization-and-validation/03-api-contracts-and-openapi/README.md)
- **Closing kept-alive connections cleanly on shutdown** — [3.4.3 — Graceful shutdown, health and readiness](../../04-configuration-and-app-structure/03-graceful-shutdown-health-readiness/README.md)
- **One error format for every `4xx`/`5xx` your API returns** — [3.8.1 — Consistent error envelopes](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.md)
- **Streaming responses that never set a `Content-Length`** — [3.8.5 — WebSockets and SSE in `axum`](../../08-error-handling-and-testing-at-scale/05-websockets-and-sse-in-axum/README.md)
- **Idempotency keys for making `POST` safe to retry** — [Phase 5 — Distributed systems patterns](../../../phase5-system-design-mastery/04-distributed-systems-patterns/README.md)

### Can you explain?

- Why is every safe method idempotent, but not every idempotent method safe?
- Why does an HTTP client retry a timed-out `PUT` on its own but not a `POST`?
- What does `q=0` mean, and why isn't it just "lowest priority"?
- Why does a `match` over the five status-code ranges not compile when `status` is a `u16`?
- Why does a chunked body need a size line for each chunk instead of just looking for the next `\r\n`?

---

## Going further

- [RFC 9110 — HTTP Semantics](https://www.rfc-editor.org/rfc/rfc9110.html): §9.2 (safe and idempotent methods), §12.5.1 (`Accept`), §15 (status codes). These are the rules you implemented today.
- [RFC 9112 — HTTP/1.1](https://www.rfc-editor.org/rfc/rfc9112.html): §7.1 (chunked transfer coding) and §9.3 (persistence and keep-alive).
- [MDN — HTTP response status codes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Status): every code, with a plain explanation.
- [MDN — Content negotiation](https://developer.mozilla.org/en-US/docs/Web/HTTP/Content_negotiation): the same rules, from the browser's side.
