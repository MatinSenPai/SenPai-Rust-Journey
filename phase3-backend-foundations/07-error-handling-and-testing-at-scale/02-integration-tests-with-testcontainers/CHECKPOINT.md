# Checkpoint

1. `live_postgres()` returns `(PgPool, ContainerAsync<Postgres>)` — a tuple,
   not just the pool — and every test binds the second element as
   `_container` rather than discarding it with `_`. What would actually go
   wrong if a test wrote `let (pool, _) = live_postgres().await;` instead?
2. This lesson's tests never call anything like `DELETE FROM p3_07_02_widgets`
   to clean up before running, unlike the shared-database lessons in module
   4. Why is that cleanup unnecessary here specifically?
3. `get_host_port_ipv4(5432)` returns some port that usually isn't 5432.
   Why does `testcontainers` map to a dynamically-chosen host port instead
   of always using 5432 directly — what would break if two of this
   lesson's tests happened to run at the same time and both tried to bind
   host port 5432?
4. Every test in `tests/integration_test.rs` is `#[ignore]`d, yet this
   lesson's `solution/` was fully verified (compiled, clippy-checked,
   formatted) in an environment with no Docker daemon at all. How is that
   possible — what exactly does `#[ignore]` skip, and what does it not skip?
5. This lesson and the previous one (`consistent-error-envelopes`) both
   model a "widget." If you had to add real Postgres-backed integration
   tests for `consistent-error-envelopes`'s `WidgetStore` too, would you
   reach for `testcontainers` the same way, or would you first change
   something about how `WidgetStore` is structured? (Hint: what does
   `WidgetStore` in that lesson depend on today, and would that
   dependency need to change to swap in a real database?)
