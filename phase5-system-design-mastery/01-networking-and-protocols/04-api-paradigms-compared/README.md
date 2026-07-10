# 01.4 — API paradigms compared

No code in this lesson — this is the payoff lesson. You've now actually
*built* a REST API, a gRPC service, and a GraphQL schema in this repo, each
in its own lesson, each explained on its own terms. This lesson puts them
side by side on purpose, so the differences stop being three separate facts
you memorized and become one coherent decision you can make: given a new
system to design, which paradigm, and why.

## The four (five) paradigms, briefly

- **REST** — resources (nouns) at URLs, HTTP verbs as the operations
  (`GET`/`POST`/`PATCH`/`DELETE`), JSON bodies, no single client-visible
  schema file (the closest thing is an OpenAPI spec, maintained separately
  and can drift from the actual code). You built this:
  `phase3-backend-foundations/02-axum-and-rest-api-design/02-anime-catalog-crud-in-memory`.
- **GraphQL** — one endpoint, a single strongly-typed schema, the *client*
  chooses exactly which fields it wants in each query. You built this:
  `phase4-backend-advanced/04-grpc-and-graphql/02-async-graphql-overview`.
- **gRPC** — a `.proto` schema defines services and messages, code
  generation produces strongly-typed client/server stubs, binary Protobuf
  over HTTP/2. You built this:
  `phase4-backend-advanced/04-grpc-and-graphql/01-tonic-grpc-service`.
- **SOAP** — XML-based, envelope-and-schema-heavy (WSDL), dominant in
  enterprise and legacy systems (banking, healthcare, government) through
  the 2000s. Mentioned here for historical completeness, not because you'll
  reach for it: verbose, XML-parsing-heavy, largely superseded by REST and
  gRPC for new systems — but if you ever integrate with an older financial
  or insurance-industry API, don't be surprised to find it still there.
- **Plain RPC** (JSON-RPC, XML-RPC, or an ad hoc "call a function over
  HTTP" convention with no formal schema at all) — the conceptual
  ancestor gRPC formalized: "call a named function with arguments, get a
  result back," without REST's resource/verb modeling. Worth naming mostly
  so gRPC doesn't look like it invented the idea of RPC over a network —
  it invented (well, productionized) *schema-first, code-generated,
  binary* RPC specifically.

## Why gRPC exists alongside REST, not instead of it

Rather than re-deriving this, `phase4-backend-advanced/04-grpc-and-graphql/01-tonic-grpc-service`'s
README already states it precisely, and this lesson treats that as the
source of truth rather than repeating it with different words: gRPC's
home is **service-to-service** calls where both ends are your own code
generated from the same `.proto` (that lesson's example: `taskforge-worker`
calling `taskforge-api`), specifically *because* browsers can't natively
speak gRPC's HTTP/2-binary-framing-plus-trailers wire format — which is
exactly why public, browser-facing APIs stay REST or GraphQL. Keep that
framing in mind for the rest of this lesson: "REST vs. gRPC" is not a
single axis with one winner, it's two tools solving different-shaped
problems (public/client-facing vs. internal/service-to-service) that
happen to both be "an API."

## Request/response shape, and over-/under-fetching

**REST** returns whatever fields the endpoint author decided the resource
has — every caller gets the same shape. `02-anime-catalog-crud-in-memory`'s
`GET /anime/{id}` always returns the full `Anime` struct (title, status,
rating, whatever else it has), whether the caller wanted all of it or just
the title. If a mobile client only needs `title` and `status` to render a
list row, it still downloads every field — **over-fetching**. If a
different screen needs data that spans two resources (an anime and its
reviews), the client has to make two separate REST calls and stitch them
together client-side — a related problem sometimes called
**under-fetching** (one endpoint alone doesn't give you what the screen
needs).

**GraphQL** was built specifically to kill over-fetching:
`02-async-graphql-overview`'s own worked example is exactly this — a query
asking for `{ note(id: "1") { title } }` gets back `{"note": {"title":
"..."}}`, no `id`, no `body`, because the client didn't ask. The same
schema also answers the under-fetching problem: a client can ask for a
note *and* its related data in one query, no client-side stitching of
multiple round-trips required, because the resolver graph — not the
client — does the joining.

**gRPC** doesn't really have this problem in the same shape, because it's
not solving it — each `rpc` method returns a fixed message defined in the
`.proto`, the same "you get what the schema says" shape as REST, just
binary instead of JSON. That's a deliberate non-goal: gRPC's typical
caller is your own other service, which needs a specific, known shape
every time, not a flexible per-caller query language.

## Schema strictness: where the contract lives

This is the axis where the three paradigms differ most sharply, and it's
worth being precise about *where* each contract actually lives, not just
whether one "has a schema."

- **gRPC**: the `.proto` file **is** the contract, and it's not optional —
  no `.proto`, no generated types, no compiling. `01-tonic-grpc-service`'s
  `build.rs` runs `tonic_build::compile_protos` at compile time; if the
  schema and the implementation disagree, the code doesn't compile. Client
  and server, even in different languages, are mechanically derived from
  the same source file, so they cannot silently drift.
- **GraphQL**: the schema is also enforced, but it's derived *from your
  Rust types* rather than a separate file you write first —
  `02-async-graphql-overview`'s `#[derive(SimpleObject)]` on `Note` and the
  `#[Object]` impl on `QueryRoot` *generate* the schema from the code, and
  a client can introspect that schema at runtime to discover exactly what's
  queryable. Strict, but the source of truth is Rust code, not a
  standalone IDL file the way `.proto` is.
- **REST**: has no schema enforcement built into the protocol at all. An
  OpenAPI spec is the closest equivalent, but it's a *separate* artifact
  someone has to write and keep in sync by hand (or generate from code with
  extra tooling) — nothing stops a REST handler's actual JSON response from
  silently drifting away from whatever documentation claims it returns.
  `phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes`'s
  whole premise — every handler independently inventing its own error JSON
  shape until someone centralizes it — is a direct symptom of REST having
  no structural mechanism forcing consistency the way a `.proto` file or a
  GraphQL schema does. That's not a flaw exclusive to REST's design so much
  as the tradeoff of *not* requiring a schema: more flexibility, less
  built-in safety.

## Browser-friendliness and typical use case

| | REST | GraphQL | gRPC | SOAP | Plain RPC |
|---|---|---|---|---|---|
| Browser-native? | yes | yes (POST + JSON) | no (needs grpc-web proxy) | yes (rare today) | usually (JSON-RPC over HTTP) |
| Schema | optional, external (OpenAPI) | required, introspectable, code-derived | required, `.proto`-first | required, WSDL/XML | usually none |
| Wire format | JSON (usually) | JSON (usually) | binary Protobuf | XML | JSON or XML |
| Best fit | public API, simple resource CRUD | client-flexible-query needs (many client shapes, one backend) | internal service-to-service, performance-sensitive | legacy enterprise/finance interop | quick internal calls, no tooling investment |

A genuine three-way decision, worked: you're building a system with (a) a
public-facing API third-party developers and your own web/mobile clients
both use, and (b) several internal services that call each other
constantly and are all written by your own team. For (a), REST is the
default-correct choice unless you specifically know different client
surfaces need meaningfully different data shapes from the same resources
(several mobile screens vs. a web dashboard vs. a partner integration, each
wanting different slices) — in which case GraphQL's client-driven queries
earn their added schema/resolver complexity. For (b), gRPC is the
default-correct choice: you control both ends, you want the compiler to
catch a mismatched field before it ships, and the performance difference
(binary framing, HTTP/2 multiplexing) actually matters at the volume of
internal service-to-service traffic. Using GraphQL for internal
service-to-service calls would mean paying for client-side query
flexibility that no client (your own other services, which always want
the same fields) actually needs. Using gRPC for the public API would mean
losing browser-nativeness for essentially every consumer, for a
performance win that matters far less at public-API traffic volumes than
it does internally.

## What this repo already showed you about testing across all three

One more thread worth pulling together, because you already lived it three
times without it being named as one idea: `01-tonic-grpc-service`'s tests
call `service.create_note(Request::new(...)).await` directly — no server
bound to a port. `02-async-graphql-overview`'s tests call
`schema.execute(...)` directly — no HTTP request at all.
`02-anime-catalog-crud-in-memory`'s `tests/api_test.rs` drives the *whole*
router through `tower::ServiceExt::oneshot` — closer to the real transport,
but still in-process, no actual socket. All three paradigms, despite
wildly different wire formats, converge on the identical discipline: the
transport (HTTP/1.1, HTTP/2, GraphQL-over-HTTP) is a thin, already-correct
layer generated or provided by a library; what your tests actually need to
exercise is *your* request-handling logic underneath it. Picking a
paradigm changes the wire format and the schema story — it doesn't change
that discipline at all.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
