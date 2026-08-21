# Progress

**Currently working on:** Phase 0, Lesson 1 — `phase0-setup/01-what-is-a-compiled-language`

> Check a box the same commit you finish the lesson (see `docs/lesson-standard.md`
> for the commit convention). Tag `phaseN-complete` when a phase's checklist
> is all checked.
>
> **This is the secondary tracker.** If you use the web UI (`cargo run -p
> course-ui`), that's the source of truth — it records progress in a gitignored
> `.course-progress.json` and never edits this file, so the two can drift if you
> tick boxes in both places. Why it works that way:
> [`docs/adr/0001-web-ui-progress-state.md`](docs/adr/0001-web-ui-progress-state.md).

<details open>
<summary><b>Phase 0 — Setup & Orientation</b></summary>

- [ ] 01 — What is a compiled language?
- [ ] 02 — Installing Rust and toolchains
- [ ] 03 — Hello, Rust
- [ ] 04 — Cargo basics
- [ ] 05 — Reading compiler errors
- [ ] 06 — Tooling: clippy, fmt, rust-analyzer
- [ ] 07 — Git and repo workflow

</details>

<details>
<summary><b>Phase 1 — Core Fundamentals</b></summary>

**01 — Foundations**
- [ ] 01 — Variables, mutability and shadowing
- [ ] 02 — Scalar types and overflow
- [ ] 03 — Compound types and destructuring
- [ ] 04 — Functions and expressions
- [ ] 05 — Control flow
- [ ] 06 — `Vec` and `String` basics

**02 — Ownership and memory**
- [ ] 01 — Stack and heap
- [ ] 02 — Move semantics
- [ ] 03 — `Clone` and `Copy`
- [ ] 04 — Ownership across functions
- [ ] 05 — `Drop` and RAII

**03 — Borrowing and references**
- [ ] 01 — Shared and mutable references
- [ ] 02 — The rules of the borrow checker
- [ ] 03 — Borrow scopes and NLL
- [ ] 04 — Slices

**04 — Text and strings**
- [ ] 01 — `String` versus `&str`
- [ ] 02 — UTF-8: bytes, chars, graphemes
- [ ] 03 — Building and transforming strings
- [ ] 04 — Slicing text safely

**05 — Your own types**
- [ ] 01 — Structs and methods
- [ ] 02 — Tuple structs and the newtype pattern
- [ ] 03 — Enums as data
- [ ] 04 — `match` in depth
- [ ] 05 — `if let`, `while let`, `let else`

**06 — Absence and failure**
- [ ] 01 — `Option` and null safety
- [ ] 02 — `Option` combinators
- [ ] 03 — `Result` and the question mark
- [ ] 04 — Panic versus `Result`
- [ ] 05 — `From` and error conversion

**07 — Putting it together**
- [ ] 01 — Guided mini-project
- [ ] 02 — Phase review

- [ ] **Side-quest 1** — [Anime Quote CLI](side-quests/sq-01-anime-quote-cli)

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

**08 — Rust toolbox**
- [ ] 01 — Pattern matching in depth
- [ ] 02 — `macro_rules!` basics
- [ ] 03 — Cargo features
- [ ] 04 — `TryFrom` and fallible conversions

- [ ] **Side-quest 2** — [Telegram Quiz Bot](side-quests/sq-02-telegram-quiz-bot)

</details>

<details>
<summary><b>Phase 3 — Backend Foundations</b></summary>

**01 — Networking & HTTP from scratch**
- [ ] 01 — TCP echo server
- [ ] 02 — Hand-rolled HTTP parser

**02 — `axum` & REST API design**
- [ ] 01 — Routing, handlers, extractors
- [ ] 02 — Anime catalog CRUD (in-memory)
- [ ] 03 — CORS and frontend integration

**03 — Serialization & validation**
- [ ] 01 — `serde_json` and `validator`

**04 — PostgreSQL & `sqlx`**
- [ ] 01 — Connecting and pooling
- [ ] 02 — Migrations
- [ ] 03 — Anime catalog, Postgres-backed
- [ ] 04 — Transactions

**05 — Database design & query performance**
- [ ] 01 — Indexing, `EXPLAIN ANALYZE`, the N+1 problem
- [ ] 02 — Pagination: offset vs. keyset

**06 — Auth & security**
- [ ] 01 — Password hashing with `argon2`
- [ ] 02 — JWTs and `tower` middleware

**07 — Error handling & testing at scale**
- [ ] 01 — Consistent error envelopes
- [ ] 02 — Integration tests with `testcontainers`

- [ ] **Side-quest 3** — [Webtoon Notification Service](side-quests/sq-03-webtoon-notifier-service)

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
- [ ] 02 — Config & secrets

**08 — Performance & profiling**
- [ ] 01 — Criterion benchmarks and flamegraphs

- [ ] **Side-quest 4** — [Anime/Manga Aggregator API](side-quests/sq-04-anime-manga-aggregator-api)

</details>

<details>
<summary><b>Capstone — TaskForge</b></summary>

_Half 1 — study the reference core (read the ADRs + code, run each crate's tests):_
- [ ] ADRs read (`capstone-taskforge/docs/adr/`)
- [ ] `taskforge-core` — domain types & job state machine
- [ ] `taskforge-storage` — Postgres repository, job claiming (`SKIP LOCKED`)
- [ ] `taskforge-worker` — async worker pool, retries/backoff, graceful shutdown
- [ ] `taskforge-scheduler` — recurring jobs, dead-letter handling
- [ ] `taskforge-api` — REST surface, OpenAPI, metrics
- [ ] `taskforge-admin-bot` — Telegram ChatOps client
- [ ] `taskforge-cli` — CLI client

_Half 2 — build the ops layer (the `todo!()`s are yours):_
- [ ] `taskforge-api/src/main.rs` — pool, migrations, serve w/ graceful shutdown
- [ ] `taskforge-worker/src/main.rs` — pool + watch-channel drain on shutdown
- [ ] `taskforge-scheduler/src/main.rs` — run schedules until shutdown
- [ ] `docker compose up --build` brings up the whole stack and it stays up
- [ ] Load test (`loadtest/`) passes + "what I'd change at 10x scale" write-up

</details>

<details>
<summary><b>Phase 5 — System Design Mastery</b></summary>

**01 — Networking & protocols**
- [ ] 01 — HTTP evolution & status codes
- [ ] 02 — Real-time communication (polling, SSE, WebSocket, webhooks)
- [ ] 03 — Proxies, gateways & load balancers
- [ ] 04 — API paradigms compared (REST/GraphQL/gRPC/SOAP/RPC)

**02 — Database & storage at scale**
- [ ] 01 — SQL vs NoSQL & choosing a database
- [ ] 02 — Transactions, ACID & isolation levels
- [ ] 03 — Sharding, partitioning & consistent hashing
- [ ] 04 — Replication & read replicas
- [ ] 05 — Indexing deep dive: B-tree vs LSM-tree
- [ ] 06 — Locking: optimistic vs pessimistic
- [ ] 07 — Message queues & event streaming (Kafka/RabbitMQ)

**03 — Caching & performance**
- [ ] 01 — Caching strategies & eviction policies
- [ ] 02 — CDNs & latency numbers every engineer should know

**04 — Distributed systems patterns**
- [ ] 01 — CAP theorem & consistency models
- [ ] 02 — Idempotency & distributed locking
- [ ] 03 — Resiliency patterns (retries, circuit breakers, timeouts)
- [ ] 04 — Unique ID generation at scale
- [ ] 05 — Scalability strategies (vertical/horizontal, partitioning)

**05 — Security & auth at scale**
- [ ] 01 — Encoding vs encryption vs hashing
- [ ] 02 — HTTPS & the TLS handshake
- [ ] 03 — Auth strategies compared (sessions/JWT/OAuth2/SSO)

**06 — DevOps & cloud fundamentals**
- [ ] 01 — Docker & containers
- [ ] 02 — Kubernetes basics
- [ ] 03 — Deployment strategies (blue-green, canary, rolling)
- [ ] 04 — The Twelve-Factor App

**07 — Applied system design**
- [ ] 01 — Design a URL shortener
- [ ] 02 — Design a rate limiter (distributed)
- [ ] 03 — Design a chat system
- [ ] 04 — Design a notification system
- [ ] 05 — Design a distributed job scheduler (revisiting TaskForge)

</details>
