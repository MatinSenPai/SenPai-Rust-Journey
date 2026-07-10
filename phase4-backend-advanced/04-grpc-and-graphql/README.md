# 04 — gRPC & GraphQL

Phase 3 built REST APIs with axum — JSON over HTTP, one resource-shaped
endpoint at a time. This module covers two alternatives you'll meet in real
backend work: **gRPC** (a binary, strongly-typed RPC protocol built on
HTTP/2 and Protocol Buffers, common for service-to-service communication)
and **GraphQL** (a query language that lets clients ask for exactly the
fields they need across a graph of types, common for client-facing APIs
with varied consumers). Neither replaces REST universally — each is a
better fit for specific situations, covered in each lesson.

1. [`tonic` gRPC service](01-tonic-grpc-service/README.md)
2. [`async-graphql` overview](02-async-graphql-overview/README.md)
