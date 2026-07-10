# Module 2 — Database & storage at scale

Phase 3 gave you working Postgres/`sqlx` fundamentals; this module is
what changes once "working" needs to become "handles real traffic and
real data volume." Sharding, replication, isolation levels, and message
queues are the vocabulary of that jump, and several lessons here
directly revisit code from earlier phases with a "here's the gap, here's
the real-world fix" lens — including a genuine bug this repo's own test
suite hit while being built.

1. [01 — SQL vs NoSQL & choosing a database](01-sql-vs-nosql-and-choosing-a-database/README.md)
2. [02 — Transactions, ACID & isolation levels](02-transactions-acid-and-isolation-levels/README.md)
3. [03 — Sharding, partitioning & consistent hashing](03-sharding-partitioning-and-consistent-hashing/README.md)
4. [04 — Replication & read replicas](04-replication-and-read-replicas/README.md)
5. [05 — Indexing deep dive: B-tree vs LSM-tree](05-indexing-deep-dive-btree-vs-lsm/README.md)
6. [06 — Locking: optimistic vs pessimistic](06-locking-optimistic-vs-pessimistic/README.md)
7. [07 — Message queues & event streaming](07-message-queues-and-event-streaming/README.md)
