# Solution

```rust
pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub fn app() -> Router {
    Router::new().route("/health", get(health))
}
```

There isn't much to say about the Rust here on purpose — this lesson's
weight is in the `Dockerfile` and `docker-compose.yml`, not the handler.
`health` follows exactly the no-extractor, `Json<T>`-return shape from
`hello`/`greet` in Phase 3's first axum lesson: construct the response
struct, wrap it in `Json`, return it. `app` wires it onto `GET /health` the
same way every earlier `Router::new().route(...)` did.

## Why this endpoint has no failure mode

Every other axum handler you've written so far had at least one
`Result`-shaped path — a missing resource, a validation failure, a lock
that could theoretically be poisoned. `health` doesn't, deliberately: it
takes no input (no `Path`, `Query`, or `Json<T>` extraction that could
fail) and does no I/O (no database call, no lock). The only way for `GET
/health` to *not* return `200 { "status": "ok" }` is for the process itself
to not be running, which is exactly the one bit of information a liveness
probe wants. Give it more logic — a database ping, a downstream service
check — and you've turned a liveness check into a readiness check, which
is a legitimate thing to want, but under a different name and a different
probe (`/ready`, typically), so an orchestrator doesn't restart a perfectly
healthy process just because Postgres is having a slow morning.

## The Dockerfile and docker-compose.yml aren't "solved" — they're read

Unlike `src/lib.rs`, `Dockerfile` and `docker-compose.yml` were never
`todo!()`-gated — there's no diff to show here because the starter and
solution versions are identical. The exercise for those two files was
reading them closely enough to answer the recall questions, not writing them.
If any recall question was hard to answer, re-read the matching section
of `README.md` — every non-obvious line in both files is explained there,
line by line.

## The one thing worth re-stating: caching only helps if you get the copy order right

The most common way to accidentally defeat the `Dockerfile`'s layer-caching
trick, once you're writing your own multi-stage builds later, is reordering
the `COPY` instructions — e.g. `COPY . .` before the dummy build, "just to
be safe." That single change makes the dummy-build layer's cache key
include every file in the build context, so it invalidates on *any* source
change exactly like the naive single-stage build did, silently throwing
away the entire point of the two-`COPY` structure while still *looking*
like a multi-stage, cache-friendly build. The rule that actually matters:
nothing that changes more often than your dependencies should be `COPY`'d
before the dependency-only build step runs.

## Tagging by commit SHA, revisited

the recall questions asks why `README.md`'s example deployment job tags images
by `${{ github.sha }}` instead of overwriting `latest`. The concrete
failure mode `latest` invites: two people deploy in quick succession, the
second deploy's `latest` silently replaces the first's, and if the second
deploy turns out to be broken, "roll back to the previous image" requires
someone to remember what the previous image even *was* — there's no name
for it anymore, because `latest` never kept one. A SHA tag makes every
build permanently, unambiguously addressable: `git log` tells you exactly
what `ghcr.io/your-org/your-service:a1b2c3d` contains, and rolling back is
just redeploying a tag that still exists and still means exactly one thing.
