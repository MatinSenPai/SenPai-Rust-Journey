# 07.1 — Docker Compose and CI

Every lesson so far has ended at `cargo test` passing on your machine. This
lesson closes the gap between "works on my machine" and "runs somewhere a
user can actually reach it": packaging a Rust service into a container,
running it alongside the databases it depends on, and connecting that to
the CI pipeline that's already been checking every push to this repo.

## Why a container, and why multi-stage

A Docker image is a filesystem snapshot plus metadata about how to run it.
The naive way to build one for a Rust service — `FROM rust:1-slim-bookworm`,
`COPY . .`, `cargo build --release`, `CMD` the binary — works, but ships
something absurd: the *entire* Rust toolchain (rustc, cargo, linker,
hundreds of MB), your full crate registry cache, and your source code, all
sitting in the image you deploy, next to a single ~10MB binary that's the
only thing that actually needs to run.

That's wasteful in two concrete ways:

- **Image size.** Every `docker pull` on every deploy, every autoscaled
  replica spinning up, moves however many hundred extra MB across the
  network for tooling that will never run again after the build finished.
- **Attack surface.** A shell, a package manager, and a compiler sitting in
  your production image are all things an attacker who gets any code
  execution inside the container can use. A minimal image that's *only*
  your binary and its runtime libraries gives them nothing to work with
  beyond the app itself.

**Multi-stage builds** fix this: `Dockerfile` in this directory defines two
`FROM` stages. The first (`builder`) has the full toolchain and produces a
release binary. The second (`runtime`) starts from a much smaller base
image and `COPY --from=builder`s *only* the compiled binary out of the
first stage. Everything from the `builder` stage — rustc, the dependency
build cache, your `src/` — is discarded; it never becomes part of the image
you actually push and run. Docker builds the `builder` stage, uses it as a
throwaway build environment, and only the final stage's filesystem ends up
as the image.

## Reading the `Dockerfile`

**Stage 1 — `builder`:**

```dockerfile
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && echo "// dummy build target" > src/lib.rs \
    && cargo build --release \
    && rm -rf src

COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release
```

The two-`COPY`, two-`RUN` shape is the **layer-caching trick**, and it's the
single most important thing to understand in this file. Docker builds an
image as a stack of layers, one per `RUN`/`COPY`/etc. instruction, and
caches each layer keyed on that instruction plus its inputs. If you `COPY
. .` and `cargo build` in one shot, *any* change to *any* file — including
a one-character change to a doc comment — invalidates the layer that
compiles every dependency, because Docker can't tell "only my code changed"
from "everything might have changed": the cache key is the whole copied
directory tree.

So instead: copy only `Cargo.toml`/`Cargo.lock` (the files that actually
describe your dependency graph) first, fabricate a throwaway `src/main.rs`
and `src/lib.rs` that compile trivially, and run `cargo build --release`
against *that*. This forces Cargo to fetch and compile every real
dependency — the slow part — and Docker caches the resulting layer keyed on
the manifest files' contents. Only once that's cached do we `COPY src
./src` (the real source) and build again. As long as `Cargo.toml`/
`Cargo.lock` haven't changed, every rebuild after the first reuses the
cached dependency layer and only recompiles *your* crate — seconds instead
of however long the full dependency graph takes from scratch. The final
`rm -rf src` before the real `COPY` isn't strictly required for caching to
work, but keeps the dummy files from lingering in a layer if something
downstream ever inspected the image's history.

`touch src/main.rs src/lib.rs` before the second build: Cargo's incremental
build logic partly keys off file modification times, and a `COPY` can, on
some filesystems/Docker versions, preserve timestamps in a way that leaves
Cargo unconvinced anything changed. Touching forces a fresh mtime, so the
second `cargo build --release` is guaranteed to actually see and compile
the real source rather than silently reusing the dummy build's output.

**Stage 2 — `runtime`:**

```dockerfile
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/p4-07-01-docker-compose-and-ci /usr/local/bin/app
RUN useradd --system --no-create-home --shell /usr/sbin/nologin appuser
USER appuser
EXPOSE 3000
CMD ["/usr/local/bin/app"]
```

- **`debian:bookworm-slim`, not `FROM scratch`.** The `rust:*` builder
  images link dynamically against glibc by default, so the binary needs
  *some* libc present at runtime — `FROM scratch` (a genuinely empty
  filesystem) would fail immediately with "no such file or directory" even
  though the binary is right there, because the dynamic linker it depends
  on doesn't exist in an empty image. Using the same Debian base
  (`bookworm`) the builder image is built on guarantees the glibc versions
  match. (A fully static build against the `x86_64-unknown-linux-musl`
  target *could* use `FROM scratch` or a `distroless/static` base for an
  even smaller image — real projects do this — but it's an extra
  cross-compilation step out of scope for this lesson.)
- **`ca-certificates`.** Only relevant once this service makes outbound
  HTTPS calls (another internal API, a webhook, a third-party SDK) — this
  lesson's `/health` endpoint doesn't need it, but it's included because
  essentially every real service eventually does, and it's cheap:
  `--no-install-recommends` plus clearing `/var/lib/apt/lists/*` keeps this
  layer from pulling in anything beyond the certificate bundle itself.
- **Non-root user.** `useradd --system ... appuser` then `USER appuser`
  switches the container's runtime identity away from root *after* the
  binary is already copied in (copying as root, then switching, means we
  never need `--chown` since `/usr/local/bin` is world-executable). This
  is defense in depth: if this service is ever compromised, "unprivileged
  user with no shell, no package manager, nothing to sudo to" is a
  meaningfully worse position for an attacker than root.
- **`CMD` vs `RUN`.** Everything above used `RUN` (executes *during the
  build*, its result baked into a layer). `CMD` is different: it names the
  process that runs when a container is *started* from the finished image
  — this is the one line that actually makes the image "an axum service"
  rather than just "a filesystem with a binary in it."

## Reading `docker-compose.yml`

```yaml
services:
  api:
    build: .
    ports:
      - "3000:3000"
    depends_on:
      postgres:
        condition: service_healthy

  postgres:
    image: postgres:16-alpine
    environment:
      POSTGRES_USER: p407
      POSTGRES_PASSWORD: p407
      POSTGRES_DB: p407
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U p407"]
      interval: 5s
      timeout: 5s
      retries: 5
```

This mirrors `capstone-taskforge/docker-compose.yml`'s conventions: a
named-volume Postgres service with a `healthcheck:` block, environment
variables for user/password/db, and the standard `pg_isready` probe.

The part worth slowing down on is `depends_on: postgres: condition:
service_healthy` instead of the bare `depends_on: [postgres]` form.
Compose's plain `depends_on` only waits for the dependency's container to
**start** — for Postgres, the container process starting and Postgres
being ready to accept connections are different moments (initdb, WAL
replay, etc. all happen after the process starts). A service that connects
to Postgres on first request, started right after the *container* exists
but before Postgres is actually *accepting connections*, gets a connection
refused and — depending on how its own retry logic is written — might
crash-loop before Postgres finishes starting. `condition: service_healthy`
makes Compose wait for the dependency's `healthcheck:` to report healthy
(here: `pg_isready` succeeding) before starting `api` at all, which is
exactly what a `healthcheck:` block is *for* — a container reporting
"started" and a container reporting "ready to do useful work" are
deliberately different signals.

Notice `api` has **no `healthcheck:` of its own** — see the comment in the
file. The runtime image's whole design goal was "no shell tools beyond the
binary," and a container `HEALTHCHECK`'s `CMD`/`CMD-SHELL` has to run
*inside* that same container — there's no `curl` or `wget` in there to run
it with. Adding one just to satisfy a healthcheck would undercut the
minimal-image work done above. Real orchestrators solve this from
*outside* the container instead: a Kubernetes readiness/liveness probe or
an ALB target group's health check both make an HTTP request from the
orchestrator to the container's exposed port, no shell execution inside the
container required at all.

## Connecting this to `.github/workflows/ci.yml`

The CI pipeline already running on every push to this repo
(`.github/workflows/ci.yml`) does three things, in order: `cargo fmt --all
-- --check` (is everything formatted), `cargo clippy --workspace
--all-targets -- -D warnings` (does anything look wrong, treated as a hard
failure, not just a warning), and `cargo test --workspace` (does everything
still behave correctly). That's **continuous integration**: verifying every
change is correct *before* it merges. Nothing in that file builds or
publishes a container — CI's job ends at "this code is good," not "this
code is running somewhere."

**Continuous deployment** is the next stage, and it isn't in this repo's
CI file today, but the shape of what you'd add is a direct extension of
what's already there — a second job in the same workflow, gated to run
only on `main` and only after `check` passes:

```yaml
  build-and-push:
    needs: check
    if: github.ref == 'refs/heads/main'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: docker/setup-buildx-action@v3
      - uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}
      - uses: docker/build-push-action@v6
        with:
          context: .
          push: true
          tags: ghcr.io/your-org/your-service:${{ github.sha }}
```

Two choices worth noticing: `needs: check` means this job never even starts
if fmt/clippy/test failed — no point building an image from code that
isn't known-good. And the image is tagged by **commit SHA**
(`${{ github.sha }}`), not a single floating `latest` tag. A SHA tag makes
every image traceable back to the exact commit that produced it (essential
for "which deploy introduced this bug" debugging and for rolling back to a
known-good image by name) and makes deploys immutable — `latest` silently
means something different every time you push, which is exactly the kind
of implicit, un-auditable state this whole curriculum has been steering you
away from.

## Reading `src/lib.rs`

`GET /health` is deliberately the simplest possible endpoint: no
extractors, no state, just proof that the process is up and answering HTTP.
This is what `docker-compose.yml` and a real orchestrator's liveness probe
poll — a *liveness* check ("is the process alive, should it be restarted if
not") is intentionally simpler than a *readiness* check ("is it ready to
serve real traffic," which might also verify the database connection pool
is up) — conflating the two in a real service can cause an orchestrator to
kill and restart a perfectly healthy process just because a downstream
dependency is briefly slow.

## Your task

Open `src/lib.rs`. Implement `health` (return `Json(HealthResponse { status:
"ok" })`) and `app` (a `Router` routing `GET /health` to `health`). Then
read `Dockerfile` and `docker-compose.yml` end to end — they aren't
`todo!()`-gated, they're finished, real artifacts to study; there's nothing
to fill in, only to understand.

Docker itself isn't runnable in this lesson's environment, so there's
nothing here to `docker build`/`docker compose up` and check by hand —
verification for this lesson is `cargo test` plus reading both files
carefully enough to answer the recall questions.

## Next

`cargo test -p p4-07-01-docker-compose-and-ci`, then the recall questions, then
`solution/SOLUTION.md`.
