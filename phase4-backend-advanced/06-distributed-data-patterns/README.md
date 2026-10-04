# Module 6 — Distributed data patterns

A transaction stops at the edge of one database. This module covers what you do when a business operation spans several: sagas with compensating steps, event sourcing and CQRS (with an honest account of when not to use them), and the multi-tenancy models that decide how one deployment safely serves many customers.

1. [01 — Sagas and compensating transactions](01-sagas-and-compensating-transactions/README.md)
   — A multi-step operation across services, and how to undo it.
2. [02 — Event sourcing and CQRS](02-event-sourcing-and-cqrs/README.md)
   — A lite, honest treatment, including when not to use them.
3. [03 — Multi-tenancy models](03-multi-tenancy-models/README.md)
   — Shared schema, schema per tenant, database per tenant, and their trade-offs.
