# 01.3 — Proxies, gateways & load balancers

No code in this lesson. `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`
already introduced round-robin, least-connections, and consistent hashing
in a couple of paragraphs, in service of explaining horizontal scaling.
This lesson goes back and does the topic properly: what a forward proxy
and reverse proxy actually are (genuinely different problems, despite the
similar name), how a reverse proxy, an API gateway, and a load balancer
relate (they overlap enough in real products that the terms get used
sloppily, but the *concepts* are distinct), and a deeper, worked pass at
load balancing algorithms than that earlier lesson had room for.

## Forward proxy vs. reverse proxy: whose interest does it serve?

Both are "a server that sits between a client and another server and
relays traffic" — the mechanism is identical. What differs is **which side
it represents**.

A **forward proxy** sits in front of a group of *clients* and represents
*them* to the outside world — a corporate network's proxy that all
employees' outbound traffic goes through (for content filtering, logging,
or so external sites see one corporate IP instead of each employee's), or
a VPN. The servers being contacted often don't know a proxy is involved at
all; from their perspective a request just arrived from some IP. The
proxy is *the client's* infrastructure, protecting or controlling *the
client's* interests.

A **reverse proxy** sits in front of a group of *servers* and represents
*them* to the outside world — clients think they're talking directly to
"the API," but a reverse proxy (nginx, Envoy, a cloud load balancer) is
actually receiving every request first and forwarding it to whichever
backend instance should handle it. The client often doesn't know (or need
to know) how many real servers are back there, or which one answered. The
proxy is *the server operator's* infrastructure, protecting or controlling
*the server's* interests — hiding internal topology, terminating TLS in
one place instead of on every instance, absorbing traffic spikes before
they reach application code.

Every load balancer you've read about so far in this repo is a reverse
proxy — "reverse proxy" is the general mechanism; "load balancer" names
one specific *job* a reverse proxy is commonly given.

## Reverse proxy vs. API gateway vs. load balancer

These three terms get used almost interchangeably in casual conversation,
and in real deployments the *same piece of software* (nginx, Envoy, a
cloud provider's managed gateway) often does all three jobs at once —
which is exactly why it's worth being precise about the three
*conceptually separate* responsibilities being bundled together:

- **Load balancer** — one job: **distribute traffic across multiple
  backend instances** so no single instance is overwhelmed and a dead
  instance doesn't take the whole service down. Nothing more.
- **Reverse proxy** — the general mechanism: receive a request on behalf of
  a backend, forward it, relay the response back. TLS termination,
  request/response rewriting, and load balancing are all things a reverse
  proxy *can* be configured to do — the term itself doesn't specify which.
- **API gateway** — a reverse proxy specifically positioned at the **edge**
  of a system (the one place all external traffic enters) and given
  application-aware responsibilities beyond routing: **authentication**
  (verify a token before anything reaches your services), **rate
  limiting** (reject abusive clients before they cost you compute),
  **request routing by path or version** (`/v1/orders` → the orders
  service, `/v2/orders` → its replacement), and sometimes **protocol
  translation** (accept public REST/JSON, translate to internal gRPC calls
  between your own services — bridging exactly the "browsers can't
  natively speak gRPC" gap `phase4-backend-advanced/04-grpc-and-graphql/01-tonic-grpc-service`'s
  README names as the reason gRPC stays internal).

The useful mental model: **load balancing is a subset of what a reverse
proxy can do, and an API gateway is a reverse proxy with opinions about
auth, rate limits, and routing layered on top.** A plain nginx load
balancer forwarding round-robin to three identical backend instances is a
reverse proxy doing load balancing and nothing else — no auth, no rate
limiting. Something like AWS API Gateway or Kong, sitting in front of a
dozen different internal services, checking a JWT, rejecting requests over
quota, and routing `/orders/*` to one service and `/users/*` to another —
that's earning the "gateway" name because it's making *application-level*
decisions, not just "which healthy instance is next."

**Where this repo already made the load-balancing-vs-gateway tradeoff, and
paid its cost knowingly:** `phase4-backend-advanced/02-rate-limiting-and-backpressure/01-token-bucket-and-tower-limit`
builds rate limiting as a `TokenBucket` living **inside the application**
(and shows the production path, `tower::limit::RateLimitLayer`, as a layer
stacked directly onto that same service's `Router`) — not at a gateway
sitting in front of it. That's a deliberate, common real-world choice, and
it comes with real tradeoffs in both directions. In-app rate limiting
means the limiting logic can see application-specific context a generic
gateway can't (a token bucket keyed per-user based on data already loaded
for the request, a rate limit that varies by subscription tier looked up
from the same database call the handler needed anyway) — but it also means
every service that needs rate limiting has to implement it itself, and if
you run three replicas of that service behind a load balancer, each
replica's `TokenBucket` tracks its own independent count unless you push
the state somewhere shared (Redis, say) — the same "in-process state
doesn't survive horizontal scaling" problem
`01-cap-scaling-lb-idempotency-locking`'s horizontal-scaling section
describes for a `HashMap` cache. Rate limiting at an API gateway instead
gets you one shared, consistent limit across every replica for free (the
gateway sees *all* traffic before it's distributed), at the cost of the
gateway not knowing anything about your application's internal state
unless you build a way to feed it in.

## Load balancing algorithms, worked

`01-cap-scaling-lb-idempotency-locking` introduced these three in a few
sentences each; here's a worked scenario for when each one actually wins.

### Round-robin

Requests 1, 2, 3, 4, 5, 6 go to instances A, B, C, A, B, C, in strict
rotation, no regard for how busy each instance currently is.

**Wins when:** every request costs roughly the same amount of work, and
every instance has roughly the same capacity. A stateless CRUD API where
every handler does "one lookup, one JSON response" — `taskforge-api`'s
`GET /jobs/{id}` is a good example — fits this well: there's no reason to
think instance A will be meaningfully busier than B after routing them
evenly, because the work each request generates doesn't vary much.

**Loses when:** request cost varies a lot. If request 1 triggers an
expensive report generation that takes 4 seconds and requests 2 and 3 are
1ms lookups, strict rotation still sends request 4 to instance A — the one
still chewing on request 1 — purely because "it's A's turn again," even
though B and C are sitting idle. Round-robin has no concept of "currently
busy," only "whose turn is it."

### Least-connections

Send the next request to whichever instance currently has the fewest
in-flight (not-yet-completed) requests.

**Wins when:** request cost is unpredictable or highly variable — exactly
the case round-robin loses on above. Route that same expensive
report-generation request to instance A; least-connections naturally
routes requests 2 and 3 to B and C instead, because A's in-flight count is
now higher than either — no request piles up behind slow work just because
of arrival order.

**Loses when:** "fewest connections" isn't actually "least loaded." A
long-lived SSE or WebSocket connection (Lesson 2 of this module) counts as
one open connection for a long time but might be doing almost no active
work most of that time — an instance with 500 mostly-idle WebSocket
connections could look "busier" by connection count than an instance doing
constant CPU-heavy work on 5 connections, even though the second instance
is actually closer to its real limit. Connection count is a proxy for
load, not load itself.

### Consistent hashing

Route based on a hash of some request property (a cache key, a user id, a
conversation id) so the *same* key reliably lands on the *same* instance
every time.

**Wins when:** instances hold state that isn't shared, and you need
repeat requests for the same key to keep hitting the instance that already
has the relevant state warm. `01-cap-scaling-lb-idempotency-locking`
already walks through this for the anime/manga aggregator side-quest's
in-process `TtlCache`: without consistent hashing, round-robin bounces the
same title id between replicas and each replica's cache stays cold. The
same reasoning applies directly to Lesson 2's WebSocket connections: once
a client's WebSocket handshake lands on instance B, every subsequent
message for that connection has to keep reaching instance B specifically —
there's no "the connection state migrated to instance C" fallback, the
TCP connection *is* pinned to whichever instance accepted the `Upgrade`.
A load balancer routing WebSocket traffic needs to either hash consistently
on something identifying the client/session, or hand out **sticky
sessions** (a related idea: pin a *session*, not necessarily via a hash, to
one backend for its duration) — plain round-robin would attempt to route a
mid-conversation message to an instance that never accepted that
connection in the first place, which simply doesn't work.

**Loses when:** the work truly is stateless and evenly costly — consistent
hashing adds complexity (computing and maintaining a hash ring, handling
what happens to a key's mapping when an instance is added or removed) that
buys you nothing over round-robin if there's no state to keep warm and no
connection to keep pinned. It also risks uneven load if your key
distribution is skewed (a viral post's id gets hashed to one instance,
which now eats a disproportionate share of traffic for that one hot key,
regardless of how "fairly" the hash function distributes keys on average).

## Next

No `cargo test` for this lesson — it is a reading lesson.
