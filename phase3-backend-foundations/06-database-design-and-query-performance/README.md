# Module 6 — Database design & query performance

A query that works fine on your laptop's ten test rows can fall over at ten
thousand. This module is what changes when data volume becomes the thing
you have to design around: reading a query plan instead of guessing, paging
through results instead of loading all of them, and shaping a schema so the
query patterns above stay fast as the table grows.

1. [01 — Indexing, `EXPLAIN ANALYZE`, the N+1 problem](01-indexing-explain-analyze-n-plus-1/README.md)
2. [02 — Pagination: offset vs. keyset](02-pagination/README.md)
3. [03 — Schema design for a real service](03-schema-design-for-a-real-service/README.md)
   — normalization, foreign keys, and the trade-offs behind the schema
   you've been handed since module 5, made explicit.
