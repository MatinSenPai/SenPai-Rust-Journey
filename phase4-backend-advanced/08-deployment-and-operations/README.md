# Module 8 — Deployment & operations

Code that only runs on your laptop is not finished. This module covers shipping a Rust service: container images, a CI pipeline, configuration and secrets across environments, and deploys and migrations that do not take the service down.

1. [01 — Docker for Rust and Compose](01-docker-compose-and-ci/README.md)
   — Multi-stage builds, small images, and a local stack with Compose.
2. [02 — CI/CD for Rust](02-ci-cd-for-rust/README.md)
   — A pipeline that builds, tests, lints and ships, and keeps itself fast.
3. [03 — Secrets, config and environments in production](03-config-and-secrets/README.md)
   — Where the 12-factor ideas from Phase 3 meet a real deployment.
4. [04 — Zero-downtime deploys, migrations and rollback](04-zero-downtime-deploys-and-rollback/README.md)
   — Rolling and blue-green deploys, and schema changes that survive both versions.
