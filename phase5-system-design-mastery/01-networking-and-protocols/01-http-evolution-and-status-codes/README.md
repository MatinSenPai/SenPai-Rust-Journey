# 01.1 — HTTP evolution & status codes

No code in this lesson. `phase3-backend-foundations` had you parse an
HTTP/1.1 request by hand and then build REST APIs on top of `axum`'s
already-parsed version of the same thing; `phase4-backend-advanced` had
you speak HTTP/2 without ever naming it (`tonic`'s gRPC transport *is*
HTTP/2). This lesson connects those dots explicitly: what actually changed
at each HTTP version bump, why, and — the part every backend engineer
touches daily whether they think about protocol history or not — what
each status code actually promises the caller.

## HTTP/1.0: one request, one connection, then hang up

HTTP/1.0 (1996) opened a new TCP connection for every single request.
Fetch a page with ten images, ten TCP connections, ten rounds of TCP's
handshake (SYN, SYN-ACK, ACK) before a single byte of the actual response
comes back. TCP handshakes aren't free — each one is a full network
round-trip of pure overhead before any real data moves — and on a
slow or high-latency link that overhead dominates.

**You've felt this cost directly.** `phase3-backend-foundations/01-networking-and-http-from-scratch/02-hand-rolled-http-parser` has you parse one request off one connection — the lesson deliberately doesn't
have you handle a second request on the same socket, because HTTP/1.0-style
"one request per connection" doesn't need to. Real servers stopped working
that way for exactly the reason above: the handshake overhead was real and
avoidable.

## HTTP/1.1: persistent connections, and a new bottleneck

HTTP/1.1 (1997) made connections **persistent** by default
(`Connection: keep-alive`): send a request, get a response, reuse the same
TCP connection for the next request instead of tearing it down. This alone
was a big win — no repeated handshake per resource.

But it introduced (or rather, exposed) **head-of-line (HOL) blocking** at
the application layer. HTTP/1.1 allows *pipelining* — sending request 2
before request 1's response arrives — but the spec still requires
responses to come back **in the order requests were sent**, on that one
connection. If request 1 is a slow database query and request 2 is a fast
static asset, request 2's already-computed response sits waiting behind
request 1's slow one. In practice, browsers mostly gave up on pipelining
entirely (too many broken proxies and servers mishandled it) and instead
worked around HOL blocking by opening **multiple parallel connections per
host** (browsers typically capped this around 6) — which brings back some
of HTTP/1.0's handshake overhead, just amortized across fewer, longer-lived
connections instead of one per request.

## HTTP/2: binary framing solves HOL blocking at the app layer

HTTP/2 (2015) throws out HTTP/1.1's text-based, line-oriented wire format
entirely and replaces it with **binary framing**: every request and
response is broken into small binary frames, each tagged with a stream ID,
and frames from *different* streams can be interleaved on the *same* TCP
connection. This is **multiplexing** — request 2's frames don't have to
wait behind request 1's frames the way HTTP/1.1's whole-response ordering
required. One connection, many concurrent logical streams, no
application-layer HOL blocking, no need for the "six parallel connections"
workaround.

**This is the exact fragility you built by hand, made obsolete.**
`02-hand-rolled-http-parser`'s `parse_request` spends most of its logic on
things that are only problems *because* HTTP/1.1 is text: splitting on
`\r\n` (and the "stray `\r` left over from a naive `\n`-only split" trap
the README calls out), confirming the bytes are valid UTF-8 before treating
them as a string at all, handling header names case-insensitively because
`Host` and `host` are the same header only by *convention*, not by any
structural guarantee. None of that ambiguity is possible in HTTP/2 —
frame types, stream IDs, and header encoding (via HPACK, a compression
scheme built specifically for HTTP's many-repeated-headers pattern) are
all defined by fixed binary layouts, not by hoping every implementation
agrees on where a line ends. That's not a coincidence: binary framing was
chosen *specifically* to eliminate the class of parsing ambiguity that
lesson makes you feel firsthand. You already benefited from this without
naming it — `phase4-backend-advanced/04-grpc-and-graphql/01-tonic-grpc-service`'s
`tonic` server speaks HTTP/2 under the hood for every RPC; the README's
"binary Protobuf encoding is smaller and faster... and HTTP/2 multiplexes
many calls over one connection" line is describing this exact mechanism.

## HTTP/3: when TCP itself becomes the bottleneck

HTTP/2 fixed HOL blocking *at the application layer*, but it still runs
over **TCP**, and TCP has its own HOL blocking problem, one layer down: TCP
guarantees bytes arrive at the application **in order**. If one TCP packet
is lost, *every* stream multiplexed onto that connection stalls — even
streams whose packets all arrived fine — because TCP won't hand any of the
buffered-but-out-of-order bytes up to HTTP/2 until the missing packet is
retransmitted and arrives. HTTP/2 multiplexing made this *worse* in one
sense: on HTTP/1.1's six separate connections, one connection stalling
didn't touch the other five; on HTTP/2's one shared connection, one lost
packet now stalls everything.

HTTP/3 (standardized 2022) fixes this by dropping TCP entirely and running
over **QUIC**, a transport protocol built on top of **UDP**. QUIC
implements its own reliability and ordering — but critically, *per stream*,
not per connection: a lost packet on stream 4 only blocks stream 4, streams
1-3 keep flowing. QUIC also folds in what used to be separate round-trips
(TCP handshake, then TLS handshake on top) into a single combined
handshake, and — a detail that matters a lot for mobile clients — a QUIC
connection is identified by a connection ID, not by the client's
IP/port tuple, so switching from WiFi to cellular mid-request doesn't
force a fresh connection the way it would over TCP. The tradeoff: UDP is
unreliable and unordered *by design*, so QUIC has to reimplement, in
userspace, a meaningful chunk of what TCP gives you for free in the kernel
— real engineering complexity, paid once by browser and server vendors, in
exchange for every application built on top not having to think about it.

## Status codes: the meaningful distinctions

Every backend lesson in this repo already produces status codes — this
section is about the decisions behind *which* code, not the numbers
themselves.

### 401 Unauthorized vs. 403 Forbidden

These get swapped constantly, and the distinction is worth being precise
about: **401 means "I don't know who you are"** (missing or invalid
credentials — no `Authorization` header, an expired token). **403 means "I
know exactly who you are, and you're not allowed to do this"** (a valid,
authenticated user requesting a resource their role doesn't grant access
to). A login endpoint rejecting a bad password is 401. An authenticated
regular user trying to hit an admin-only endpoint is 403. Getting this
backwards is a common code-review nit for a reason: a client (or a
frontend's error handling) needs to know whether "log in again" (401) or
"this account can't do that, don't retry" (403) is the right next action —
conflating them produces exactly the "the frontend can't write one
error-handling function" problem `phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes`'s
README opens with.

### 409 Conflict vs. 422 Unprocessable Entity (and where this repo lands on 400)

**409** means the request is well-formed and the data would be valid *in
general*, but it conflicts with the *current state* of the resource.
**422** means the request is well-formed (valid JSON, right shape) but
semantically invalid regardless of any state — a value out of range, a
required field with a nonsensical value.

You've already built a real 409: `capstone-taskforge/taskforge-api/src/error.rs`
maps `JobError::NotCancellable(status)` to `ApiError::conflict(...)` — a
`StatusCode::CONFLICT`. Trying to cancel a job isn't invalid as a *request
shape*; it's invalid because *this specific job*, right now, is already
`running` or `completed`. That's a conflict with current state, textbook
409. Compare that to `phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes`'s
`CreateWidget` validation (`quantity` out of the 1..100_000 range) — that's
not a conflict with any existing state, it's invalid on its own terms,
which is the 422 case in the strictest reading of the spec. Notice, though,
that both `01-consistent-error-envelopes` and `taskforge-api` map
validation failures to **400 Bad Request**, not 422 — a genuinely common
real-world choice (Rails and many REST APIs use 422 for this; Stripe and
plenty of others use 400) because 422 is a WebDAV extension code that many
HTTP clients, proxies, and monitoring dashboards historically treat as an
unrecognized edge case rather than a first-class 4xx. Neither choice is
"wrong" — what matters is picking one and being consistent across every
endpoint, which is precisely what centralizing the mapping in one
`impl IntoResponse for ApiError` (rather than letting each handler decide)
guarantees you can't accidentally violate.

### 202 Accepted vs. 200/201

**200 OK** (or **201 Created** for a `POST` that made a new resource) means
"done, here's the result, right now." **202 Accepted** means "I've taken
this in, but the work isn't finished yet — check back later" — the honest
status code for anything that gets handed off to async processing instead
of completed synchronously within the request.

`capstone-taskforge/taskforge-api/src/handlers.rs`'s `POST /jobs` handler
returns `StatusCode::CREATED` (201) — and it's worth sitting with *why*
that's defensible even though the job itself hasn't actually *run* yet by
the time the response goes out (a `taskforge-worker` process picks it up
later via `claim_next`'s `FOR UPDATE SKIP LOCKED`, asynchronously). 201 is
correct here because the *resource the endpoint promises* is the job
**record** — and that record genuinely was created, synchronously, before
the response was sent; the client can immediately `GET /jobs/{id}` and see
it. If `POST /jobs` instead needed to say "I've accepted your request to
create a job, but I haven't even validated or persisted it yet, a
downstream queue will do that" — a design with an extra layer of
indirection this repo didn't need — 202 would be the more honest code,
because there'd be no queryable resource yet at response time. The general
rule: 201 promises "the thing exists now, here's its location"; 202
promises only "I heard you, come back later to find out what happened."

### 4xx vs. 5xx, and why the distinction drives retry logic

The single most consequential thing a status code communicates to an
automated caller isn't "did it work" — it's **"should I retry, and how."**
**4xx** means the client did something the server won't accept no matter
how many times you retry it *unchanged* (bad auth, invalid input, a
resource that doesn't exist) — retrying the identical request forever
just wastes everyone's time. **5xx** means the server itself failed to do
its job on a request that was plausibly fine (a database connection
dropped, an unhandled panic, a downstream dependency timed out) — retrying
later, ideally with backoff, is often the *correct* move, because the same
request might well succeed once whatever broke recovers.

This is the same distinction `phase4-backend-advanced/02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit`
leans on without spelling it out: an exhausted token bucket answers with
**429 Too Many Requests** — a 4xx, but a special one, since retrying is
*correct* here (that's the whole point of a rate limit), just not
immediately; a well-behaved client reads `429` as "back off and retry
later," never as "something is permanently wrong with this request." A
retry policy that blindly retries every non-2xx response, or one that
never retries anything, both throw away information the status code is
specifically there to carry.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
