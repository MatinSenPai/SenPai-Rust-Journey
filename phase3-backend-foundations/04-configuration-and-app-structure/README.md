# Module 4 — Configuration & app structure

Every lesson so far has hardcoded what a real service can't: a fixed port,
no secrets worth protecting, one process that never needs to stop cleanly.
This module is the plumbing a real deployment actually needs before it ever
touches a database — config that comes from the environment instead of a
constant, one place the whole app's shared state lives, and a shutdown that
finishes in-flight requests instead of dropping them.

1. [01 — 12-factor config and secrets](01-config-and-secrets/README.md)
   — layered configuration (environment over file over default) and
   `SecretString`, so a password never accidentally ends up in a log line
   or a `Debug` print.
2. [02 — Application state and dependency wiring](02-app-state-and-dependency-wiring/README.md)
   — `AppState`, traits vs. concrete types, and when dependency injection
   actually earns its keep in Rust versus when it's just ceremony.
3. [03 — Graceful shutdown, health and readiness](03-graceful-shutdown-health-readiness/README.md)
   — finishing in-flight requests on `SIGTERM` instead of dropping them,
   and the `/health`/`/ready` endpoints a real deployment checks before it
   ever routes traffic to you.

Everything from module 5 onward assumes a real, long-running service — this
module is what makes that assumption true.
