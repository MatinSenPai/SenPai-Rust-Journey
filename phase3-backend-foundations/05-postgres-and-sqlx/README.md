# Module 5 — PostgreSQL & `sqlx`

Every REST endpoint so far has lived in a `Vec` or a `HashMap` that resets
the moment the process restarts. This module replaces that with a real
database — connecting to it properly, changing its schema safely over time,
and the two guarantees a database gives you that an in-memory store never
could: durability and atomicity.

1. [01 — Connecting and pooling](01-connecting-and-pooling/README.md)
2. [02 — Migrations](02-migrations/README.md)
3. [03 — Anime catalog, Postgres-backed](03-anime-catalog-postgres-backed/README.md)
   — module 2's in-memory API, rebuilt against a real database, so the
   change happening under the hood is the whole point of the lesson.
4. [04 — Transactions](04-transactions/README.md)
5. [05 — The repository pattern](05-repository-pattern/README.md)
   — pulling raw `sqlx` calls behind a trait, so your handlers can be
   tested without a real database and swapped onto a different store later.

**Setup:** PostgreSQL installed locally (or via Docker) starting at this
module — lesson 1's `README.md` covers setup, nothing earlier needed it.
