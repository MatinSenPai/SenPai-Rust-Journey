# Phase 5 — System Design Mastery

Phases 0-4 and the TaskForge capstone taught you to *build*: ownership,
async, `axum`, Postgres, Redis, gRPC, observability, a real production
crate graph. This phase teaches you to *talk about* what you built, and
fills in the vocabulary and patterns that show up constantly in system
design interviews and real production incidents but that no single lesson
so far needed in isolation — sharding, consistent hashing, event
streaming, resiliency patterns, auth strategies compared side by side,
the DevOps layer underneath everything.

This phase is curated from
[ByteByteGo's *System Design 101*](https://github.com/ByteByteGoHq/system-design-101),
a large, excellent, visual system-design primer — but that source has
roughly 400 short posts, many overlapping, some company-trivia
("Netflix's Tech Stack"), some deep in payment-rail specifics (SWIFT,
VISA, ACH) far outside general backend engineering, some pure book/blog
recommendation lists. Rather than mechanically porting every post, this
phase distills the genuinely teachable engineering concepts into
coherent lessons, cross-referenced back to code you've already written
wherever one exists — CAP theorem points at `taskforge-storage`, locking
points at the toy queue's `FOR UPDATE SKIP LOCKED` *and* the lost-update
race in module 3's Postgres-backed anime catalog, caching points at the
Redis lesson, and so on. Nothing here should feel like trivia memorized
in isolation.

## Structure

Modules 1-6 are **reading + worked-example** lessons — no `Cargo.toml`,
same anatomy as `06-system-design-fundamentals` back in Phase 4 (a
substantial `README.md`, a `CHECKPOINT.md`, no code, no solution).
Module 7 is **hands-on**: five applied "design X" mini-projects, built
for real with `axum`/Postgres/Redis, in this phase's own workspace
crates.

1. [Networking & protocols](01-networking-and-protocols/README.md)
2. [Database & storage at scale](02-database-and-storage-at-scale/README.md)
3. [Caching & performance](03-caching-and-performance/README.md)
4. [Distributed systems patterns](04-distributed-systems-patterns/README.md)
5. [Security & auth at scale](05-security-and-auth-at-scale/README.md)
6. [DevOps & cloud fundamentals](06-devops-and-cloud-fundamentals/README.md)
7. [Applied system design](07-applied-system-design/README.md) — hands-on

When Phase 5 is fully checked off in [`PROGRESS.md`](../PROGRESS.md),
you've covered the full arc: write Rust confidently, ship a real backend,
and design (and defend, in an interview or an ADR) a distributed system
at whatever scale the job actually needs.
