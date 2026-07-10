# Module 5 — Database design & query performance

Module 4 got you talking to real Postgres. This module is about the
difference between a query that *works* and a query that works at 2 million
rows — reading `EXPLAIN ANALYZE` output, understanding what an index
actually buys you, and recognizing the N+1 query problem in raw SQL instead
of behind an ORM's lazy-loading magic.

1. [01 — Indexing, `EXPLAIN ANALYZE`, and the N+1 problem](01-indexing-explain-analyze-n-plus-1/README.md)
   — B-tree indexes, seq scan vs. index scan, and the exact Django-ORM N+1
   trap you already know, rebuilt over `sqlx`.

By the end of this module you'll be able to look at a slow query, explain
*why* it's slow from its query plan, and recognize an N+1 access pattern
before it ships.
