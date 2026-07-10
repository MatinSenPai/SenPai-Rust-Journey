# 04.4.2 — `async-graphql` overview

Previous lesson: gRPC, strongly-typed and schema-first, aimed at
service-to-service calls. This lesson: GraphQL, also schema-first, but aimed
at the opposite end of a system — client-facing APIs, where different
clients (a web app, a mobile app, a third-party integration) each want a
different slice of the same data.

## The core idea: the client shapes the response

A REST `GET /notes/1` returns whatever fields the server decided to include
— every caller gets the same shape, whether they need all of it or not. A
GraphQL query asks for exactly the fields it wants:

```graphql
query { note(id: "1") { title } }
```

returns `{"note": {"title": "..."}}` — no `id`, no `body`, because the
client didn't ask for them. Same server, same underlying data, different
callers can request different shapes without the backend team publishing a
new endpoint or an API version bump for every UI variation. This is the
lesson's `notes_query_returns_every_field_a_client_asked_for` test made
concrete: it queries `{ notes { title } }` and asserts the response objects
have no `id` key at all.

## Reading the schema: `Object` vs `SimpleObject`

```rust
#[derive(Clone, SimpleObject)]
pub struct Note {
    pub id: ID,
    pub title: String,
    pub body: String,
}
```

`Note` is "just data" — every field is resolved by reading the struct
directly, so `#[derive(SimpleObject)]` generates all three field resolvers
for you, the GraphQL analogue of `#[derive(Serialize)]`.

```rust
pub struct QueryRoot;

#[Object]
impl QueryRoot {
    async fn note(&self, ctx: &Context<'_>, id: ID) -> Option<Note> { ... }
    async fn notes(&self, ctx: &Context<'_>) -> Vec<Note> { ... }
}
```

`QueryRoot` and `MutationRoot` are different: each method needs to *do*
something (read shared state) rather than just expose a field, so they're
written by hand inside a `#[Object]` impl block — one method per
query/mutation the schema exposes. `note`'s `Option<Note>` return type is
what makes the field nullable in the schema; a missing note is `null`, not
an error (contrast this with the previous lesson's `Status::not_found` —
GraphQL and gRPC make different defaults here, and both are reasonable for
their respective use cases).

## Shared state: `ctx.data_unchecked::<T>()`

```rust
pub fn build_schema() -> NotesSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(Mutex::new(NotesState::default()))
        .finish()
}
```

`.data(...)` attaches request-independent shared state to the schema —
conceptually the same role `axum::extract::State` played in Phase 3, or the
`Arc<AppState>` you'd pass into a `tonic` service. Every resolver reaches
it back out via `ctx.data_unchecked::<Mutex<NotesState>>()`. The
`_unchecked` suffix means it panics (rather than returning a `Result`) if
that type was never registered with `.data(...)` — acceptable here because
`build_schema()` always registers it, so a missing-data panic would only
ever indicate a programmer error, not bad user input. (The fallible
sibling, `ctx.data::<T>()`, exists for cases where that assumption doesn't
hold.)

## `EmptySubscription`

`Schema<QueryRoot, MutationRoot, EmptySubscription>` — GraphQL has a third
root type beyond Query and Mutation: **Subscription**, for a client that
wants a live stream of updates over a persistent connection (e.g. "notify
me every time a note is created"). This lesson doesn't need one, so
`EmptySubscription` is a real, provided type that fills the slot without
implementing anything — the schema simply has no subscription fields. Real
subscription support would look like a `#[Subscription]` impl block
returning a `Stream`, out of scope here.

## Testing without a server, again

Just like the `tonic` lesson, every test here calls `schema.execute(...)`
directly with a GraphQL query string — no `axum` router, no
`async-graphql-axum` glue, no HTTP request at all. `Schema::execute` is a
plain async method; wiring a schema behind an HTTP endpoint (`POST
/graphql` accepting the query in a JSON body, typically via
`async-graphql-axum`'s `GraphQL` extractor) is a separate, mechanical step
you'd add in a real service, layered on top of a schema that's already
fully testable on its own — the same "test the logic, trust the transport"
shape as every earlier lesson in this module.

## Your task

Open `src/lib.rs`. Implement the three resolvers: `QueryRoot::note`,
`QueryRoot::notes`, and `MutationRoot::create_note`.

## Checkpoint

`cargo test -p p4-04-02-async-graphql-overview`, then `CHECKPOINT.md`, then
`solution/SOLUTION.md`.
