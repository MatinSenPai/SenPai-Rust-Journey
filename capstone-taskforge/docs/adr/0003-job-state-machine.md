# ADR-0003: Job state machine

## Status
Accepted

## Context

A job needs a status that's always meaningful — no combination of fields
should be able to represent nonsense (e.g. "succeeded, but also scheduled
to retry"). This is the exact "make illegal states unrepresentable" idea
from `phase1-fundamentals/05-structs-enums-pattern-matching/
02-enums-and-match`'s checkpoint, now applied for real.

## Decision

Model `JobStatus` as an enum, not a status string plus a pile of nullable
columns:

```rust
pub enum JobStatus {
    Pending,
    Running,
    Succeeded,
    Retrying { attempt: u32, next_attempt_at: DateTime<Utc> },
    Failed { error: String },
    DeadLetter { error: String },
}
```

Valid transitions, enforced by `JobStore` implementations (not by the
database schema alone — Postgres stores the enum's serialized shape, but
the *rules* live in Rust, where the compiler and tests can verify them):

```
Pending → Running
Running → Succeeded
Running → Retrying   (attempts < max_attempts)
Running → DeadLetter (attempts >= max_attempts)
Retrying → Running   (once next_attempt_at has passed and a worker claims it)
```

`Failed` (as a standalone terminal state, distinct from `DeadLetter`) is
reserved for a job explicitly cancelled by a user via the API — not part of
the automatic retry path.

## Consequences

- A job can never be "succeeded with a pending retry" or any other
  nonsensical combination — the type doesn't allow constructing it.
- `Retrying`'s `next_attempt_at` carries the backoff-computed timestamp
  directly on the state, so "should a worker claim this yet" is a single,
  obvious comparison (`next_attempt_at <= now()`), not a derived
  calculation scattered across the codebase.
- Storing this enum in Postgres means picking a serialization (TaskForge
  uses a `status` text column for the variant tag plus nullable
  `attempt`/`next_attempt_at`/`error` columns) — this is the one place the
  "no nonsense states" guarantee is *not* automatically enforced by the
  database schema itself, only by every write going through `JobStore`'s
  typed methods. A stretch extension: enforce it with a Postgres `CHECK`
  constraint too, for defense in depth.
