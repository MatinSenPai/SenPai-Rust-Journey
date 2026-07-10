# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can defend a paradigm choice for a
concrete system using the three lessons you actually built, not recite a
comparison table.

1. `phase4-backend-advanced/04-grpc-and-graphql/01-tonic-grpc-service`'s
   README says gRPC's typical home is service-to-service calls, not
   browser-facing APIs. Explain the specific technical reason (not just "it's
   less common") browsers can't be a normal gRPC client, and why that reason
   doesn't apply to GraphQL even though GraphQL also has a strict schema.
2. Using `02-anime-catalog-crud-in-memory`'s `GET /anime/{id}` as the
   concrete example, describe a client that would over-fetch from it, and
   then describe how the equivalent GraphQL query
   (`02-async-graphql-overview`'s `Note`/`note` pattern) would avoid that
   over-fetch. Be specific about which fields are involved.
3. `01-consistent-error-envelopes`'s README opens with "every handler
   invents its own error shape" as a REST-specific failure mode. Explain why
   that specific failure mode is structurally much harder to have happen in
   `01-tonic-grpc-service`'s gRPC service, tying your answer to what
   `build.rs` actually enforces at compile time.
4. You're designing a system with a public API for third-party developers
   and a cluster of your own internal microservices that call each other
   frequently. Argue for a specific paradigm for each of those two surfaces,
   and explain what would go wrong (concretely, not just "it'd be
   suboptimal") if you swapped your two choices.
5. `01-tonic-grpc-service`'s tests call `service.create_note(...)` directly,
   `02-async-graphql-overview`'s tests call `schema.execute(...)` directly,
   and `02-anime-catalog-crud-in-memory`'s tests drive the router via
   `tower::ServiceExt::oneshot`. Despite three different wire protocols,
   explain what testing principle all three share, and why that principle
   holds regardless of which API paradigm you pick.
