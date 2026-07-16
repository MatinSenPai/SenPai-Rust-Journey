# syntax=docker/dockerfile:1
#
# One Dockerfile, any TaskForge service. Which binary gets built is chosen by
# the BIN build-arg (see docker-compose.yml, where api/worker/scheduler each
# pass a different one).
#
# IMPORTANT: the build context must be the REPO ROOT, not this directory —
# these crates are members of the root Cargo workspace, so `cargo build -p
# <crate>` needs every workspace manifest and the shared Cargo.lock. The
# compose file sets `context: ..` for exactly this reason. A root
# `.dockerignore` keeps `target/` and `.git/` out of the build context.

ARG BIN=taskforge-api

# ---- build stage -------------------------------------------------------------
FROM rust:1-slim AS builder
ARG BIN
WORKDIR /build

# Copy the whole workspace. We don't split out a dependency-caching layer
# (cargo-chef) on purpose — it's a real optimisation for CI, but it triples
# the size of this file and this is a teaching capstone, not a build farm.
# ponytail: whole-workspace COPY, add cargo-chef only if image build time bites.
COPY . .

# `-p ${BIN}` builds just that binary and its dependency graph — not the ~90
# lesson crates elsewhere in the workspace.
RUN cargo build --release -p ${BIN}

# Copy the selected binary to a fixed name so the runtime stage needs no ARG.
RUN cp target/release/${BIN} /build/service

# ---- runtime stage -----------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# ca-certificates: sqlx talks to Postgres over rustls and must verify its
# certificate chain. Nothing else from the build toolchain ships here.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Run as a non-root system user — none of these services ever need root.
RUN useradd --system --no-create-home --user-group taskforge
USER taskforge

COPY --from=builder /build/service /usr/local/bin/service
ENTRYPOINT ["/usr/local/bin/service"]
