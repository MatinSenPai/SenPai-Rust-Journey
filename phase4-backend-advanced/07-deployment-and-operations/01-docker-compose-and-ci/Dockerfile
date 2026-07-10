# syntax=docker/dockerfile:1
#
# Multi-stage build for p4-07-01-docker-compose-and-ci. See README.md for a
# line-by-line walkthrough of every non-obvious choice here. Two stages:
#
#   1. `builder` — the full Rust toolchain, compiles a release binary.
#   2. final stage — no toolchain at all, just the compiled binary and the
#      handful of runtime libraries it's dynamically linked against.
#
# The point: the image you actually ship and run in production never
# contains rustc, cargo, your crate registry cache, or your source code —
# only the artifact those things produced. Smaller image (less to pull,
# less disk), smaller attack surface (nothing to exploit that isn't the
# compiled binary itself).

########################################
# Stage 1: build
########################################
FROM rust:1-slim-bookworm AS builder
WORKDIR /app

# --- Dependency-layer caching ---
# Copy *only* the files that describe your dependency graph first, and
# build against a throwaway `src/`. Docker caches each `RUN`/`COPY` layer
# keyed on its inputs; as long as Cargo.toml and Cargo.lock are unchanged,
# Docker reuses this (slow — it compiles every dependency) layer on every
# subsequent build, even after you've edited real application code. Skip
# this trick and *any* one-line change to `src/main.rs` invalidates the
# dependency-compilation layer too, turning a few-second rebuild into a
# multi-minute one on every iteration.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && echo "// dummy build target" > src/lib.rs \
    && cargo build --release \
    && rm -rf src

# --- Real source, real build ---
# Only *this* layer re-runs when application code changes — the dependency
# layer above stays cached. `touch` forces Cargo to see the copied files as
# newer than the dummy build's output (Cargo keys incremental rebuilds off
# mtimes; without the touch, a `COPY` that preserves an old mtime can, on
# some filesystems, leave Cargo thinking nothing changed).
COPY src ./src
RUN touch src/main.rs src/lib.rs && cargo build --release

########################################
# Stage 2: runtime
########################################
# `debian:bookworm-slim` rather than `FROM scratch` or a `distroless`
# image: this binary is dynamically linked against glibc (the default for
# the `rust:*` builder images), so the runtime image needs *some* libc to
# link against. `bookworm-slim` is the same Debian base the builder image
# is built on, so glibc versions match exactly — no risk of a runtime glibc
# older than what the binary was linked against. (A fully static musl
# build, `x86_64-unknown-linux-musl` target, would let you use
# `gcr.io/distroless/static` or even `FROM scratch` instead — smaller
# still, but out of scope here; see the README.)
FROM debian:bookworm-slim AS runtime

# ca-certificates: only needed if this service makes outbound HTTPS calls
# (a real service usually does — calling another internal API, a
# third-party webhook, etc.). Included here for realism even though this
# lesson's health endpoint doesn't need it. `--no-install-recommends` and
# clearing the apt list keep this layer from dragging in packages (and
# their CVEs) nobody asked for.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/p4-07-01-docker-compose-and-ci /usr/local/bin/app

# Run as a non-root user. If an attacker ever does get code execution
# inside this container, "root inside the container" is a meaningfully
# worse starting position than "an unprivileged user with no shell and no
# package manager" — defense in depth, not a guarantee of anything on its
# own. Created (and switched to) *after* the COPY above: `/usr/local/bin`
# is root-owned but world-executable, so `appuser` can run the binary
# without needing write access to anything — no `--chown` juggling needed.
RUN useradd --system --no-create-home --shell /usr/sbin/nologin appuser
USER appuser

EXPOSE 3000
CMD ["/usr/local/bin/app"]
