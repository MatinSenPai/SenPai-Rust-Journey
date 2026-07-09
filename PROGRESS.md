# Progress

**Currently working on:** Phase 0, Lesson 1 — `phase0-setup/01-what-is-a-compiled-language`

> Check a box the same commit you finish the lesson (see `docs/conventions.md`
> for the commit convention). Tag `phaseN-complete` when a phase's checklist
> is all checked.

<details open>
<summary><b>Phase 0 — Setup & Orientation</b></summary>

- [ ] 01 — What is a compiled language
- [ ] 02 — Installing Rust and toolchains
- [ ] 03 — Cargo basics
- [ ] 04 — Tooling: clippy, fmt, rust-analyzer
- [ ] 05 — Hello, Rust
- [ ] 06 — Git and repo workflow

</details>

<details>
<summary><b>Phase 1 — Core Fundamentals</b></summary>

**01 — Variables, types & control flow**
- [ ] 01 — Variables, mutability, shadowing
- [ ] 02 — Scalar and compound types
- [ ] 03 — Conditionals, loops, match intro

**02 — Ownership & memory**
- [ ] 01 — Move semantics
- [ ] 02 — Clone and Copy
- [ ] 03 — Drop and RAII

**03 — Borrowing & references**
- [ ] 01 — Shared and mutable refs
- [ ] 02 — Borrow checker rules

**04 — Strings & slices**
- [ ] 01 — `String` vs `&str`
- [ ] 02 — Slices

**05 — Structs, enums & pattern matching**
- [ ] 01 — Structs and methods
- [ ] 02 — Enums and match
- [ ] 03 — `if let` / `while let`

**06 — `Option`, `Result` & error basics**
- [ ] 01 — `Option` and null safety
- [ ] 02 — `Result` and the `?` operator
- [ ] 03 — `panic!` vs `Result`

- [ ] **Side-quest 1** — [Anime Quote CLI](../side-quests/sq-01-anime-quote-cli)

</details>

<details>
<summary><b>Phase 2 — Intermediate & Idiomatic Rust</b></summary>

**01 — Collections**
- [ ] 01 — `Vec` and `HashMap`
- [ ] 02 — `BTreeMap`, `HashSet`, `VecDeque`

**02 — Iterators & closures**
- [ ] 01 — Closures and `Fn` traits
- [ ] 02 — Iterator adapters

**03 — Generics & traits**
- [ ] 01 — Generic functions and structs
- [ ] 02 — Defining and implementing traits
- [ ] 03 — Trait objects vs. static dispatch

**04 — Error handling & lifetimes**
- [ ] 01 — Custom error types
- [ ] 02 — `thiserror` and `anyhow`
- [ ] 03 — Lifetime basics and elision

**05 — Smart pointers**
- [ ] 01 — `Box` and heap allocation
- [ ] 02 — `Rc` and `Arc`
- [ ] 03 — `RefCell` and interior mutability

**06 — Project organization & testing**
- [ ] 01 — Modules, visibility, workspaces
- [ ] 02 — Unit, integration & doc tests
- [ ] 03 — `unsafe` Rust overview

**07 — Concurrency & async**
- [ ] 01 — Threads, `Mutex`, `Arc`
- [ ] 02 — Channels and message passing
- [ ] 03 — Futures and runtimes
- [ ] 04 — Tokio basics

- [ ] **Side-quest 2** — [Telegram Quiz Bot](../side-quests/sq-02-telegram-quiz-bot)

</details>

<details>
<summary><b>Phase 3 — Backend Foundations</b></summary>

**01 — Networking & HTTP from scratch**
- [ ] 01 — TCP echo server
- [ ] 02 — Hand-rolled HTTP parser

**02 — `axum` & REST API design**
- [ ] 01 — Routing, handlers, extractors
- [ ] 02 — Anime catalog CRUD (in-memory)

**03 — Serialization & validation**
- [ ] 01 — `serde_json` and `validator`

**04 — PostgreSQL & `sqlx`**
- [ ] 01 — Connecting and pooling
- [ ] 02 — Migrations
- [ ] 03 — Anime catalog, Postgres-backed

**05 — Database design & query performance**
- [ ] 01 — Indexing, `EXPLAIN ANALYZE`, the N+1 problem

**06 — Auth & security**
- [ ] 01 — Password hashing with `argon2`
- [ ] 02 — JWTs and `tower` middleware

**07 — Error handling & testing at scale**
- [ ] 01 — Consistent error envelopes
- [ ] 02 — Integration tests with `testcontainers`

- [ ] **Side-quest 3** — [Webtoon Notification Service](../side-quests/sq-03-webtoon-notifier-service)

</details>

<details>
<summary><b>Phase 4 — Backend Advanced + System Design</b></summary>

**01 — Caching with Redis**
- [ ] 01 — Cache-aside, TTL, invalidation

**02 — Rate limiting & backpressure**
- [ ] 01 — Token bucket and `tower::limit`

**03 — Background jobs & message queues**
- [ ] 01 — Postgres `SKIP LOCKED` toy queue
- [ ] 02 — Broker concepts: RabbitMQ, Kafka, NATS

**04 — gRPC & GraphQL**
- [ ] 01 — `tonic` gRPC service
- [ ] 02 — `async-graphql` overview

**05 — Observability**
- [ ] 01 — Structured logging with `tracing`
- [ ] 02 — Metrics and Prometheus

**06 — System design fundamentals**
- [ ] 01 — CAP, scaling, load balancing, idempotency, distributed locking

**07 — Deployment & operations**
- [ ] 01 — Docker Compose and CI

**08 — Performance & profiling**
- [ ] 01 — Criterion benchmarks and flamegraphs

- [ ] **Side-quest 4** — [Anime/Manga Aggregator API](../side-quests/sq-04-anime-manga-aggregator-api)

</details>

<details>
<summary><b>Phase 5 — Capstone: TaskForge</b></summary>

- [ ] ADRs written (`capstone-taskforge/docs/adr/`)
- [ ] `taskforge-core` — domain types & job state machine
- [ ] `taskforge-storage` — Postgres repository, job claiming
- [ ] `taskforge-worker` — async worker pool, retries/backoff, graceful shutdown
- [ ] `taskforge-scheduler` — recurring jobs, dead-letter handling
- [ ] `taskforge-api` — REST surface, OpenAPI, metrics
- [ ] `taskforge-admin-bot` — Telegram ChatOps client
- [ ] `taskforge-cli` — CLI client (stretch)
- [ ] Load test + "what I'd change at 10x scale" write-up

</details>
