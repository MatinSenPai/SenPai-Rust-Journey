# Checkpoint

1. The `Dockerfile`'s `builder` stage copies `Cargo.toml`/`Cargo.lock`,
   builds a throwaway `src/`, then copies the real `src/` and builds again.
   Walk through what happens to Docker's layer cache on a rebuild where you
   changed one line in `src/lib.rs` but touched no dependency. Which `RUN`
   steps re-execute, and which are served from cache?
2. The final image is `FROM debian:bookworm-slim`, not `FROM rust:1-slim-bookworm`
   (the same image the `builder` stage uses). What would go wrong — or just
   get worse — if you shipped the `builder` stage itself to production
   instead of copying the binary out of it into a second stage?
3. `docker-compose.yml` gives `postgres` a `healthcheck:` block and gives
   `api` a `depends_on: postgres: condition: service_healthy`, not just a
   bare `depends_on: [postgres]`. What's the practical difference in
   container startup ordering between the two, and what bug would a bare
   `depends_on` let through that `condition: service_healthy` prevents?
4. `api` in `docker-compose.yml` deliberately has no `healthcheck:` of its
   own. Why not, given the Dockerfile's minimal runtime stage, and what's
   the alternative a real deployment (Kubernetes, an ALB) uses instead?
5. `.github/workflows/ci.yml` runs `fmt --check`, `clippy`, and `test` on
   every push. If you were adding a "build and push a Docker image on merge
   to main" job to that same workflow file, what should gate it so it only
   runs on `main` and only after the existing `check` job passes — and why
   tag the resulting image by commit SHA rather than always overwriting a
   single `latest` tag?
6. The `builder` stage's dummy-build trick writes a throwaway `src/main.rs`
   *and* `src/lib.rs` before the first `cargo build --release`. This crate
   has both a library target and a binary target. What would happen to the
   caching trick if the dummy build only created `src/main.rs` and skipped
   `src/lib.rs`?
