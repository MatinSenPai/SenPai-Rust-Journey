# Checkpoint

1. `note(id: ID!): Note` returns `Option<Note>` and GraphQL turns a `None`
   into `null` rather than an error. The previous lesson's `get_note` RPC
   returned `Err(Status::not_found(...))` for the same situation. Both are
   defensible — when would you want "missing" to be a typed error instead
   of a nullable field, and why might a GraphQL API lean toward nullable by
   default?
2. `notes_query_returns_every_field_a_client_asked_for` queries
   `{ notes { title } }` and checks the response has no `id` key. Where in
   this lesson's code is that behavior actually implemented — is there a
   branch of logic in `MutationRoot`/`QueryRoot` that decides which fields
   to include, or does something else make that happen automatically?
3. `ctx.data_unchecked::<Mutex<NotesState>>()` panics if the type was never
   registered. Look at `build_schema()` — under what circumstance, if any,
   could that panic actually happen in this lesson's code as written? Is it
   a real risk or a theoretical one?
4. `Schema<QueryRoot, MutationRoot, EmptySubscription>` names three type
   parameters. What would you need to add to this file to support a
   subscription like "notify me every time a note is created" — don't
   write the code, just describe what pieces would have to change.
5. Compare `Note`'s `#[derive(SimpleObject)]` to `QueryRoot`/`MutationRoot`'s
   hand-written `#[Object]` impl blocks. What's the concrete rule for which
   one a given type needs?
6. Both this lesson and the previous one model the same "notes" domain with
   an in-memory `Mutex`-guarded `HashMap`. If a team needed both a gRPC
   service *and* a GraphQL API in front of the same underlying data, what's
   the risk of literally copy-pasting the storage struct into both crates
   (as these two lessons do for teaching clarity), and what earlier concept
   in this curriculum (hint: Phase 3's ports-and-adapters lesson) would you
   reach for to avoid that in production?
