# 07 — Deployment & Operations

Every earlier module in this phase assumed a Rust service already exists,
running, reachable, ready to be called. This module closes that gap: how
does the binary `cargo build --release` produces actually end up as a
running container next to a database, and how does the CI pipeline that's
been quietly fmt/clippy/test-ing every push in this repo's
`.github/workflows/ci.yml` connect to *shipping* that container?

1. [Docker Compose & CI](01-docker-compose-and-ci/README.md)
2. [Config & secrets](02-config-and-secrets/README.md)
