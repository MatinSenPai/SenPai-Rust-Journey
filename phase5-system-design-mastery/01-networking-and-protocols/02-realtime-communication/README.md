# 01.2 — Real-time communication

No code in this lesson. Every REST endpoint you've built so far in this
repo (the anime catalog, `taskforge-api`) follows the same shape: client
asks, server answers, connection's job is done. That shape breaks down the
moment the *server* has something to say before the client asks again — a
new chat message, a job finishing, a price changing. This lesson is a tour
of the four ways to get server-initiated data to a client, their real
tradeoffs, and the "inverse" pattern (webhooks) for when the client
*receiving* the push is itself a server.

## Short polling: the naive baseline

The client just asks repeatedly — `GET /messages?since=<timestamp>` every
2 seconds, forever, whether or not anything changed. Every request pays a
full HTTP request/response round-trip (headers, TLS if applicable, a
database query) even when the answer is "nothing new." Two costs compound:
**latency** (a new message can sit undelivered for up to the poll interval)
and **server load that scales with idle clients**, not with actual events —
1,000 idle clients polling every 2 seconds is 500 requests/second hitting
your database asking "anything new?" and getting "no" back 499 times out
of 500. The interval is a direct trade: shorter means fresher data and
proportionally more wasted load; longer means less load and proportionally
staler data. There's no setting that's actually *good*, only less bad.

## Long polling: hide the latency, keep the request count

The client makes a request; instead of answering immediately with "nothing
new," the server **holds the connection open** (no response sent yet) until
either new data becomes available or a timeout is reached (typically
20-30s) — then responds, and the client immediately opens a new long-poll
request. This eliminates the "polling interval vs. staleness" tradeoff for
*latency* specifically (data shows up the moment it exists, not on the next
tick) — but it doesn't eliminate the *load* problem, it just moves it: your
server now needs to hold open one connection (and the resources backing
it — a task, a buffer, potentially a held-open async handler) **per
currently-connected client**, all the time, rather than periodically. At
1,000 concurrent users that's 1,000 held-open connections your server
infrastructure has to support simultaneously, which is exactly the kind of
load `tokio`'s async model (Phase 2) is good at absorbing cheaply compared
to a thread-per-connection model — but it's still real, ongoing resource
commitment, not "free" the way it might look from the client's perspective.

## Server-Sent Events (SSE): one-directional, built on plain HTTP

SSE is a single long-lived HTTP response with `Content-Type:
text/event-stream` that the server keeps writing lines to over time,
instead of closing after one payload — the browser's `EventSource` API
consumes it as a stream of `data: ...\n\n` events. Two things make SSE
distinctive:

- **It's plain HTTP.** No new protocol, no special handshake — it works
  through any proxy or load balancer that already handles HTTP correctly,
  and the browser (via `EventSource`) automatically reconnects on a dropped
  connection with no client code needed for that.
- **It's one-directional: server → client, only.** The client can't send
  anything back over the same connection — if it needs to send data too,
  that's a completely separate, ordinary HTTP request.

SSE is the right tool exactly when the data flow is genuinely one-way —
live scores, a stock ticker, streaming an LLM's response token by token, a
progress indicator for a long-running job. Forcing a full-duplex protocol
onto a one-directional problem just to have "one connection" buys you
nothing; SSE's simplicity (it's still just HTTP) is a real advantage when
you don't need the client to talk back.

## WebSocket: full-duplex, a different protocol entirely

A WebSocket connection starts as an HTTP request (`Upgrade: websocket`)
and then the *same TCP connection* switches protocols entirely — from then
on it's a persistent, full-duplex byte stream where either side can send a
message at any time, with none of HTTP's request/response framing. This is
the only one of the four options that's genuinely **bidirectional** at the
protocol level: a chat app where any participant can send a message at any
moment, a multiplayer game syncing state both ways, a collaborative editor
— anything where "client sends" and "server sends" are both first-class,
frequent, and not naturally modeled as "the client asks a question."

The cost is real, not free: a WebSocket connection is stateful
infrastructure your server has to hold open per client (same resource
commitment as long polling, but for the entire session rather than
20-30s at a time), it doesn't automatically reconnect the way `EventSource`
does (your client code has to handle drops and retry), and — because it's
not plain request/response HTTP — some older or more restrictive
corporate proxies historically mishandled the `Upgrade` handshake (much
rarer today, but worth knowing as a historical reason SSE sometimes got
chosen over WebSocket even for server-only-push use cases).

**This is exactly where Phase 5 Module 7's "design a chat system" lesson**
(`phase5-system-design-mastery/07-applied-system-design/03-design-a-chat-system`)
**picks up** — a chat system is the textbook case for WebSocket precisely
because messages flow both directions unpredictably, and that lesson builds
it for real using `axum`'s `axum::extract::ws::WebSocketUpgrade` extractor
— no lesson so far in this repo has touched WebSockets yet, this is the
first place you'll see one.

## Picking the right one: a worked comparison

| | Short polling | Long polling | SSE | WebSocket |
|---|---|---|---|---|
| Direction | client-pull | client-pull (delayed) | server-push | full-duplex |
| Latency | up to poll interval | near-instant | near-instant | near-instant |
| Idle server cost | spiky, proportional to poll rate | one held connection per client, always | one held connection per client, always | one held connection per client, always |
| Browser support / infra friendliness | trivial (any HTTP client) | trivial (any HTTP client) | very good, auto-reconnect built in | needs `Upgrade` support, no built-in reconnect |
| Bidirectional? | no (separate requests) | no (separate requests) | no | yes |

A concrete decision: a **live sports score ticker** is server → client
only, tolerates the occasional dropped-and-reconnected update, and
benefits from SSE's free auto-reconnect and "it's just HTTP" operational
simplicity — WebSocket would add full-duplex machinery this use case never
uses. A **chat app** genuinely needs both directions flowing independently
and frequently — SSE would force sending client → server messages over an
awkward second channel (a plain `POST` alongside the SSE stream), which is
exactly the kind of accidental complexity that "pick the protocol that
matches the actual data flow" avoids. **Phase 5 Module 7's "design a
notification system" lesson**
(`phase5-system-design-mastery/07-applied-system-design/04-design-a-notification-system`)
is built specifically around this decision: it compares SSE vs. push
notifications for "tell a user something happened" — one-directional by
nature — as a deliberate contrast with the chat lesson's WebSocket choice,
so the two applied lessons back to back make the tradeoff concrete rather
than abstract.

## Webhooks: the inverse pattern

Everything above assumes the receiving end is a browser. **Webhooks** solve
the same "server has something to tell you before you ask" problem when
the receiver is *your own backend server*, not a browser tab — Stripe
telling your server a payment succeeded, GitHub telling your server a push
happened. The mechanism flips entirely: instead of your server holding a
connection open (SSE, WebSocket) or the caller repeatedly asking (polling),
**the other party makes an ordinary `POST` request to a URL you registered
in advance**, whenever their event happens. Your server is just a normal
HTTP server with one more route — no held-open connections, no polling
loop, nothing exotic. This works precisely because a server (unlike a
browser behind NAT or a firewall) has a stable, publicly reachable address
to receive that `POST` at.

The real complexity webhooks introduce isn't the mechanism, it's
everything *around* it: the sender needs a **retry policy** (your endpoint
was down for a minute — did they retry, and for how long?), which means
your handler needs to be **idempotent** — the exact concept
`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
covers under "Idempotency," applied here from the other side: a webhook
retry hitting your endpoint twice for the same event is the identical
"client can't tell whether the first attempt actually landed" problem that
lesson describes for `POST /jobs`, just with a third-party service instead
of a browser as the retrying client. A production webhook handler also
needs to verify the request genuinely came from who it claims to (a
signature header, checked against a shared secret) — anyone who discovers
your webhook URL can otherwise `POST` fake events at it.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
