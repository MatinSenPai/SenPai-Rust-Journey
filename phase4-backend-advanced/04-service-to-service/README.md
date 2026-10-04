# Module 4 — Service-to-service communication

Once there is more than one service, how they talk becomes a design decision. This module builds a gRPC service with tonic, looks at GraphQL with async-graphql, and then steps back to the question that matters more than either: which of REST, gRPC, GraphQL, or events fits which job.

1. [01 — A gRPC service with tonic](01-tonic-grpc-service/README.md)
   — Protobuf, tonic, and a service you can call from another language.
2. [02 — async-graphql: an overview](02-async-graphql-overview/README.md)
   — A GraphQL schema in Rust and what the resolver model costs you.
3. [03 — Choosing between REST, gRPC, GraphQL and events](03-choosing-rest-grpc-graphql-or-events/README.md)
   — A decision guide, with the failure mode of each choice.
