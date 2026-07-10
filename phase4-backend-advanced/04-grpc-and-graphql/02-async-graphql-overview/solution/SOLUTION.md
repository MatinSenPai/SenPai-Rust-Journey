# Solution

```rust
async fn note(&self, ctx: &Context<'_>, id: ID) -> Option<Note> {
    let state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();
    state.notes.get(id.as_str()).cloned()
}

async fn notes(&self, ctx: &Context<'_>) -> Vec<Note> {
    let state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();
    state.notes.values().cloned().collect()
}

async fn create_note(&self, ctx: &Context<'_>, title: String, body: String) -> Note {
    let mut state = ctx.data_unchecked::<Mutex<NotesState>>().lock().unwrap();

    let id = state.next_id.to_string();
    state.next_id += 1;

    let note = Note {
        id: ID::from(id.clone()),
        title,
        body,
    };
    state.notes.insert(id, note.clone());
    note
}
```

Same `Mutex<NotesState>`-guarded `HashMap` as the `tonic` lesson — lock,
read or mutate, return a clone, let the guard drop at the end of the
block. `id.as_str()` is needed because `async_graphql::ID` wraps a
`String` but isn't itself a `&str`; `HashMap::get` needs a `&str` key
lookup against `HashMap<String, Note>`.

## Where "only the requested fields come back" actually happens

Nothing in `QueryRoot` or `MutationRoot` decides which fields to return —
`notes()` always returns full `Note` values, `create_note` always builds a
complete `Note`. The field-selection behavior lives entirely in
`#[derive(SimpleObject)]` on `Note` and in `async-graphql`'s query executor:
each field in a GraphQL query string (`{ notes { title } }`) triggers a
call to *just that field's* generated resolver, and the executor assembles
the final JSON from only the fields that were actually asked for. This is
the same separation as `serde`: your Rust struct always fully exists in
memory, but what gets serialized (or, here, resolved into the response) is
controlled entirely by what the query asks for, not by any code you wrote
in the resolver methods.

## Is the `ctx.data_unchecked` panic a real risk here?

No — not as this lesson is written. `build_schema()` is the *only* place
`Schema::build(...).finish()` is called, and it always calls
`.data(Mutex::new(NotesState::default()))` first, so every schema instance
that ever executes a query has that data registered. The panic would only
become a real risk if a second code path built a schema without that
`.data(...)` call (e.g. a stripped-down schema for introspection-only
tooling) — a genuine failure mode in larger codebases with multiple
schema-construction call sites, which is exactly why `async-graphql`
provides the fallible `ctx.data::<T>()` alternative for code that can't
make the same one-constructor guarantee.

## Adding a subscription (conceptually)

`Schema<QueryRoot, MutationRoot, EmptySubscription>` would become
`Schema<QueryRoot, MutationRoot, SubscriptionRoot>`, where `SubscriptionRoot`
is a new unit struct with a `#[Subscription]` impl block (not `#[Object]`)
containing an `async fn note_created(&self, ctx: &Context<'_>) -> impl
Stream<Item = Note>`. `MutationRoot::create_note` would need to push each
new `Note` into some broadcast channel (a `tokio::sync::broadcast::Sender`
stored alongside `NotesState`) that the subscription resolver turns into a
`Stream` for each connected client — conceptually the same "one producer,
many independent consumers" shape as `tokio::sync::broadcast` covered in
Phase 2's channels lesson, just wired into GraphQL's subscription transport
(typically WebSockets) instead of an in-process `mpsc`.

## `SimpleObject` vs. hand-written `#[Object]`

The rule: if every field can be answered by reading a struct field
directly with no extra logic, no shared state, and no `async` work,
`#[derive(SimpleObject)]` is sufficient and shorter. The moment a field
needs to *do* something — look up shared state via `ctx`, hit a database,
compute a derived value, return `Option`/`Result` based on runtime
conditions — that field (and usually the whole type) needs a hand-written
`#[Object]` impl block, because a derive macro has no way to know what
custom logic you want. `Note` never touches `ctx`; `QueryRoot` and
`MutationRoot` do nothing but touch `ctx` — that's the entire reason one
type gets `SimpleObject` and the other two don't.

## The copy-pasted storage risk

`NotesState`/`Mutex<HashMap<...>>` is duplicated verbatim between this
lesson and the `tonic` one specifically because each lesson is meant to
stand alone and be readable in isolation — a deliberate, *pedagogical*
tradeoff, not a template for a real system. In production, running both a
gRPC service and a GraphQL API over the same underlying notes would mean
two independent in-memory copies silently diverging (a note created via
gRPC would never appear in a GraphQL query). Phase 3's ports-and-adapters
lesson is the fix: define a `NotesRepository` trait (`create`, `get`,
`list`) once, implement it against a real datastore (Postgres, in the
capstone's style), and have both the `tonic` service and the GraphQL
resolvers depend on `Arc<dyn NotesRepository>` — exactly how
`taskforge-api` and `taskforge-worker` both depend on the same
`JobStore` trait rather than each keeping their own copy of job state.
