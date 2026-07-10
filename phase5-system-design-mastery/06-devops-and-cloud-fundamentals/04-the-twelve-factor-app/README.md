# 06.4 — The Twelve-Factor App

No code in this lesson. The Twelve-Factor App is a set of rules for
building software-as-a-service apps that are easy to deploy, scale, and
hand off between developers without tribal knowledge — written up by
Heroku's engineers in 2011, and still the vocabulary most backend teams
reach for when explaining *why* a service is built a certain way. The
honest framing for this lesson: you've been following several of these
rules since well before Phase 5, without the rules being named. This lesson
puts a name on each one and points at the exact place in this repo where
you already made the call — or, in a couple of cases, honestly didn't.

## I. Codebase

*One codebase tracked in revision control, many deploys.* A service has
exactly one codebase (even if many developers work on it), and every
environment it runs in — your laptop, staging, production — deploys from
that same codebase at some specific commit. What breaks this rule is
sharing code between services by copy-pasting it instead of extracting a
shared dependency, or letting one "codebase" quietly fork into
environment-specific variants that drift apart.

**This repo:** `capstone-taskforge` is a single Cargo workspace inside a
single git repository — every `taskforge-*` crate is tracked together, and
any given commit is a specific, reproducible snapshot of the whole system.
A production deploy and your local `cargo run` both trace back to the same
codebase at whatever commit each is running.

## II. Dependencies

*Explicitly declare and isolate dependencies.* Never rely on something
happening to already be installed on the host system — declare every
dependency, and pin exactly which versions, so a fresh checkout on a
different machine builds identically.

**This repo:** this is just `Cargo.toml` + `Cargo.lock`. Every crate this
repo uses is explicitly declared, and the committed `Cargo.lock` pins exact
versions — a clean `cargo build` on a different machine resolves to the
identical dependency graph, not "whatever version happened to be newest
that day." `phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`'s
`Dockerfile` reinforces this at the container level too: the `builder`
stage starts from a specific `rust:1-slim-bookworm` image rather than
whatever Rust happens to be on the host doing the build.

## III. Config

*Store config in the environment.* Anything that varies between
deploys — database credentials, API keys, which environment you're
in — belongs in environment variables, never hardcoded or committed to the
repo. The test: could you make this codebase open source right now without
leaking a credential? If config is hardcoded, no.

**This repo already does this, repeatedly, going back to Phase 3.**
`phase3-backend-foundations/04-postgres-and-sqlx/01-connecting-and-pooling`'s
README has every DB-touching test reading `DATABASE_URL` from the
environment rather than a committed connection string.
`taskforge-storage`'s Postgres integration tests do the same
(`std::env::var("DATABASE_URL").expect(...)`), and `taskforge-admin-bot`'s
`main.rs` reads `TELOXIDE_TOKEN` (via `Bot::from_env()`),
`TASKFORGE_API_URL`, and `TASKFORGE_API_TOKEN` the identical way. None of
these values live in source.

## IV. Backing services

*Treat backing services as attached resources.* A database, a message
queue, a cache — anything the app talks to over the network — should be
swappable by changing config, with zero code changes, whether that's local
Postgres today and a managed RDS instance in production tomorrow, or a
local instance versus a colleague's second local instance during
debugging.

**This repo:** `taskforge-core`'s `JobStore` trait is exactly this
principle in code form, not just in config. `taskforge-storage` ships two
implementations — `PostgresJobStore` for real use and `InMemoryJobStore`
for tests (you saw the latter throughout `taskforge-worker`'s test suite) —
and nothing above the trait boundary (`WorkerPool`, `taskforge-api`'s
handlers) knows or cares which one it's talking to. Swapping backing stores
is a config/wiring change at the composition root, not a rewrite. The
*connection string itself* being config (Factor III, `DATABASE_URL`) is
what makes even a single implementation swappable across environments —
same Postgres code, different attached instance in dev vs. prod.

## V. Build, release, run

*Strictly separate the build and run stages.* Build compiles code into an
artifact. Release combines that artifact with a specific environment's
config. Run executes that release. These need to stay distinct steps you
can't blur together — you should never be able to change code as part of
"running" it, or the whole point of having a reproducible, auditable
artifact falls apart.

**This repo:** this is the whole shape of
`phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`'s
multi-stage `Dockerfile`. The `builder` stage is **build** — it takes
source and produces a binary, nothing more. The tagged image
(`03-deployment-strategies` covers tagging by commit SHA) is **release** —
a build artifact plus everything needed to run it, addressable and
immutable. `docker run` (or a Kubernetes Deployment scheduling a Pod from
that image, per `02-kubernetes-basics`) is **run** — executing a specific
release, with no code changes possible at this stage, only config
(environment variables) supplied at startup.

## VI. Processes

*Execute the app as one or more stateless processes.* Anything that needs
to persist across requests — session state, cached data that must be
consistent — belongs in a backing service (a database, a shared cache),
never in a process's own memory. A stateless process can be killed and
replaced, or run as N identical replicas, with zero coordination between
replicas.

**This is a direct rerun of a lesson you already had, not new territory:**
`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`'s
horizontal-scaling section makes exactly this argument about
`taskforge-api` — `AppState` holds only `Arc<dyn JobStore>` (confirmed in
`taskforge-api/src/lib.rs`), no per-request mutable state, so every handler
re-derives everything it needs from Postgres on every request. That's what
makes it safe to run `taskforge-api` as multiple replicas behind a load
balancer with zero code changes — precisely the "stateless process" factor,
named.

## VII. Port binding

*Export services via port binding.* The app itself should bind a port and
speak its protocol directly (HTTP, in this repo's case) — it shouldn't
depend on being injected into a container of a separate webserver process
(the old-school "drop a `.war` into Tomcat" model) to become reachable.

**This repo does this — with one honest gap worth naming.**
`phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`'s
`src/main.rs` binds its own port directly: `tokio::net::TcpListener::bind(addr)`
followed by `axum::serve(listener, app())` — the binary *is* the HTTP
server, no external process required. The gap: `addr` is the hardcoded
literal `"0.0.0.0:3000"`, not read from a `PORT` environment variable. A
strict reading of this factor says the port itself should also be
externally configurable — many real deploy targets (some PaaS providers,
some Kubernetes setups) assign the port dynamically and expect the process
to read it from `$PORT`. This repo's lesson hardcodes it because the
`Dockerfile`'s own `EXPOSE 3000` and `docker-compose.yml`'s `"3000:3000"`
mapping are written to match — consistent, but not the fully environment-
driven version the factor describes. A real next step, same shape as the
idempotency gap called out in the CAP lesson: read `PORT` from the
environment with a `3000` fallback, one line, genuinely not done today.

## VIII. Concurrency

*Scale out via the process model.* Rather than making one process juggle
more work through increasingly complex internal concurrency, run more
processes — and different kinds of work (a web-request-handling process, a
background-job-processing process) scale independently of each other,
each along its own axis.

**This repo:** `taskforge-worker`'s `WorkerPool` has both dimensions of
this built in. Within one process, `.with_concurrency(n)` controls how many
concurrent worker loops run (`pool.rs`'s `WorkerPool::run` spawns
`self.concurrency` independent `tokio::spawn` loops, each claiming and
running jobs on its own). Across processes, `02-kubernetes-basics`'s
mapping table covers the other half: `taskforge-api` and `taskforge-worker`
are different process *types* doing different work, and they scale on
independent axes — more API replicas for more HTTP throughput, more worker
replicas for more job-processing throughput, with no requirement that they
scale together. Neither dimension requires touching the other's code.

## IX. Disposability

*Maximize robustness with fast startup and graceful shutdown.* A process
should be startable and killable at any moment with minimal consequence —
fast startup so scaling up or recovering from a crash is quick, and
graceful shutdown so a process being killed (a deploy, a crash, a
downscale) doesn't corrupt in-progress work.

**This repo:** fast startup is `01-docker-and-containers`'s whole point —
a container is process isolation on the already-running host kernel, which
is exactly why it starts in milliseconds rather than the seconds-to-minutes
a VM takes to boot its own kernel. Graceful shutdown is concretely built
into `taskforge-worker`: `WorkerPool::run` takes a `tokio::sync::watch::Receiver<bool>`
shutdown signal, and per the doc comment on `run` and ADR-0004
(`docs/adr/0004-worker-failure-handling.md`), each worker loop "stops
claiming *new* jobs immediately... but waits for in-flight jobs to finish
... before exiting" — a deploy or restart doesn't abandon a job mid-run.
The one honest gap: nothing in this repo's source wires that
`watch::Receiver` up to an actual OS signal (`SIGTERM`, which is what
Docker/Kubernetes send when they want a container to stop) — `taskforge-worker`
has no `main.rs` yet (only `taskforge-admin-bot` and `taskforge-cli` do),
so the shutdown *mechanism* is fully built and tested, but the last wire
from "the OS asked this process to stop" to "flip the `watch` channel" isn't
connected yet.

## X. Dev/prod parity

*Keep development, staging, and production as similar as possible* —
same backing services, same versions, ideally the same *kind* of
environment entirely, not "SQLite locally, Postgres in production" gaps
that let a whole category of bug (SQL dialect differences, transaction
behavior differences) hide until production.

**This repo:** `capstone-taskforge/docker-compose.yml` runs
`postgres:16-alpine` — a real Postgres, the same database engine any
production deployment would use — rather than substituting something
lighter-weight for local development. This is deliberate: every
Postgres-specific behavior this repo's lessons rely on (`SELECT ... FOR
UPDATE SKIP LOCKED`, covered repeatedly across the distributed-locking
material) is real Postgres behavior in every environment, dev included,
not an approximation that might diverge under a different engine in
production.

## XI. Logs

*Treat logs as event streams.* An app shouldn't concern itself with
routing or storing its own log output — no writing to a log file, no
managing log rotation. It just writes each event, unbuffered, to `stdout`;
the execution environment (a container runtime, an orchestrator, a log
aggregator) is responsible for capturing that stream and doing something
with it.

**This is, almost word for word, what
`phase4-backend-advanced/05-observability/01-structured-logging-with-tracing`
already taught, under a different name.** That lesson's `tracing`/
`tracing_subscriber` setup emits structured events — either human-readable
or JSON via `build_subscriber(json: bool)` — and a real `main.rs` installs
it once with `.init()`, exactly what `taskforge-admin-bot/src/main.rs` does
(`tracing_subscriber::fmt::init()`, first line of `main`). Nothing in
either lesson writes to a file or manages rotation — every event goes to
`stdout` as a stream, and it's whatever's running the process (a terminal
during local dev, a container runtime piping `stdout` to a log driver in
production) that decides where that stream ends up. JSON-formatted output
specifically exists *for* the "log aggregator" half of that handoff — a
structured event stream is what tools like Loki or CloudWatch Logs
Insights are built to consume.

## XII. Admin processes

*Run admin/management tasks as one-off processes, in the same environment,
against the same codebase and config as the app itself* — never a
different, drifted script maintained on the side.

**This repo:** `taskforge-admin-bot`'s Telegram commands (`/status`,
`/retry <id>`, `/pause_queue`, per `capstone-taskforge/README.md`'s
architecture table) are exactly this: one-off administrative actions,
issued on demand, running against the *same* `taskforge-api` and reading
config (`TASKFORGE_API_URL`, `TASKFORGE_API_TOKEN`) the same way the rest
of the system does — not a separate maintenance script with its own
hardcoded connection details that could quietly drift out of sync with
what's actually deployed. `taskforge-storage`'s migrations
(`taskforge-storage/migrations/`, run via `sqlx migrate`) are the other
classic example of this factor: a one-off command run against the same
database the app itself connects to via the same `DATABASE_URL`, not a
hand-maintained SQL script applied out-of-band.

## The honest summary

Ten of these twelve you'd already satisfied by the time you finished Phase
4, without the rule being named — Config, Dependencies, Backing services,
Processes, Concurrency, most of Disposability, Dev/prod parity, Logs,
Codebase, and Admin processes all trace directly back to a decision this
repo made in an earlier lesson. Two have a real, specifically-named gap:
Port binding (the port itself is hardcoded, not read from `$PORT`) and the
last mile of Disposability (the shutdown *mechanism* exists and is tested,
but isn't wired to a real `SIGTERM` handler yet because the relevant
binaries don't exist yet). That's not a knock on the repo — it's the
difference between "this discipline is baked into how the code is
structured" (true, repeatedly, for ten of twelve) and "every last wire is
connected end to end in a deployed system" (not yet true, because this
repo has never actually been deployed). Naming the gaps precisely is itself
the skill this lesson is teaching.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
