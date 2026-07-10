# 04.3 — Resiliency patterns

No code in this lesson. These four patterns — retries, circuit breakers,
timeouts, bulkheads — are the standard toolkit for keeping one failing
dependency from taking down everything that depends on it. You've already
built one of them for real; the other three, you've configured a piece of
without necessarily naming the pattern.

## Retries with exponential backoff and jitter

You've seen this exact phrase before: `capstone-taskforge/taskforge-worker`
implements it in full. `taskforge-worker/src/backoff.rs`'s `compute_backoff`
computes `base * 2^(attempt - 1)`, capped at a configured `max`, then adds
up to `±jitter_fraction` random jitter:

```rust
pub fn compute_backoff(attempt: u32, config: &BackoffConfig) -> Duration {
    let exponent = attempt.saturating_sub(1).min(20);
    let unjittered = config.base
        .checked_mul(1u32.checked_shl(exponent).unwrap_or(u32::MAX))
        .unwrap_or(config.max)
        .min(config.max);
    // ...jitter applied on top...
}
```

`taskforge-worker/src/pool.rs`'s `run_one_job` calls this on every handler
failure (`Ok(Err(error))`) *and* every handler panic (`Err(join_error)` from
the `JoinHandle`) — both paths compute
`compute_backoff(job.attempts + 1, backoff)` and pass the result as the
job's `next_attempt_at` before calling `store.mark_failed`. The reasoning
is spelled out in `capstone-taskforge/docs/adr/0004-worker-failure-handling.md`:
without jitter, every job that failed at the same moment (say, a
downstream API blip that failed 50 in-flight jobs simultaneously) would
all compute the *identical* backoff delay and all retry at the *identical*
moment again — a synchronized "thundering herd" against whatever they're
calling, potentially causing the exact same blip all over again. Jitter
spreads those retries out over a window instead of a single instant.

Two things worth noticing in the actual implementation: `base=1s, max=300s`
means the delay curve is 1s, 2s, 4s, 8s... up to a 5-minute ceiling —
capping matters because uncapped exponential growth means a job that's
failed 20 times would otherwise wait days for its next retry, long after
anyone's still paying attention to it. And backoff is computed *before*
`mark_failed` is even called, so the delay is baked into the job's
persisted state (`JobStatus::Retrying { next_attempt_at, .. }`) — any
worker, not just the one that just failed the job, can pick it back up
once that time passes, which is what makes retries safe across a pool of
independent worker processes rather than requiring one worker to "remember"
to retry its own failed jobs.

## Circuit breakers

Retries assume the failure is transient — worth trying again shortly. But
if a downstream dependency is *actually down*, not just blipping, retrying
every failed call still sends traffic at it, over and over, which does
nothing but waste your own resources (connections, threads, retry
timers) and can make the downstream service's recovery *slower* (it's now
also fielding a flood of retries on top of whatever caused it to go down
in the first place). A circuit breaker adds a state machine on top of
retries specifically to stop that from happening:

- **Closed** (normal): calls go through as usual. The breaker counts
  recent failures.
- **Open**: once failures cross a threshold (e.g. 50% of the last 20 calls
  failed), the breaker stops calling the downstream service *at all* —
  every call fails immediately, locally, without a network round-trip —
  for a cooldown period.
- **Half-open**: after the cooldown, the breaker lets a small number of
  test calls through. If they succeed, it closes (resumes normal traffic).
  If they still fail, it reopens and waits again.

**Worked example:** imagine `taskforge-worker` had a job type that calls
an external payment service, and that service goes down entirely (not
flaky — genuinely returning errors or timing out on every call).

*Without* a circuit breaker: every worker in the pool keeps claiming
payment jobs, calling the dead service, waiting for the timeout, failing,
and scheduling a backoff retry — but with `concurrency` workers all doing
this independently and the queue continuously handing out newly-eligible
retries as their `next_attempt_at` passes, the system keeps sending a
steady stream of doomed requests at the payment service indefinitely.
Every one of those calls also ties up a worker task for however long the
timeout takes, which is capacity not available for *other* job types
sharing the same worker pool.

*With* a circuit breaker wrapping the payment-service call: after the
first handful of failures, the breaker opens. Every subsequent claimed
payment job fails instantly (no network call, no waiting for a timeout) —
still scheduled for retry via the normal backoff mechanism, but the
*attempt itself* costs microseconds instead of a full timeout window. Once
the cooldown passes, one test call checks whether the payment service has
recovered before resuming full traffic — so recovery is detected quickly,
without ever fully hammering the service while it's down. The failure is
still visible (jobs still end up `Retrying`, eventually `DeadLetter` if
they exhaust `max_attempts` while the outage continues) — a circuit
breaker doesn't hide failure, it just stops *making it worse*.

## Timeouts

Every network call needs an explicit timeout, full stop — without one, a
single slow (not even failed, just *slow*) dependency can quietly exhaust
every resource waiting on it. You've already configured exactly this
concept: `phase3-backend-foundations/04-postgres-and-sqlx/01-connecting-and-pooling`
sets `PgPoolOptions::new().max_connections(5).acquire_timeout(Duration::from_secs(3))`.
`acquire_timeout` bounds how long a caller will wait to check out a
connection from the pool before giving up — without it, a caller waits
indefinitely (`sqlx`'s own internal default is 30 seconds, but that's
still a default *you* didn't choose, and "give up eventually" is not the
same guarantee as "give up predictably"). The failure mode that lesson's
`SOLUTION.md` walks through directly: if every one of the pool's 5
connections is tied up running a slow query, a 6th caller with no
`acquire_timeout` just hangs forever waiting for one to free up — and if
enough callers pile up in that state, you've exhausted an entire request
path's capacity because of *one* slow dependency, exactly the same shape
of problem a missing circuit breaker causes, just one layer lower (a
connection pool instead of a whole downstream service).

The general principle `acquire_timeout` is one instance of: every blocking
call across a process boundary — an HTTP request to another service, a
database query, a lock acquisition, a message queue poll — needs a bound
on how long you're willing to wait, chosen deliberately, not inherited
from whatever library default happens to apply.

## Bulkheads

Named after ship design: a ship's hull is divided into watertight
compartments so a hole in one section floods only that section, not the
whole ship. Applied to software: **isolate resource pools per-dependency**,
so one dependency's resource exhaustion doesn't starve requests that don't
even touch it.

Concretely, imagine `taskforge-api` had one shared connection pool used
for both the main `jobs` table queries *and* some hypothetical slow
reporting query hitting the same Postgres instance. If the reporting query
is slow enough to hold connections for a long time under load, it can
exhaust the *entire* shared pool — and now `POST /jobs` (which has nothing
to do with reporting) starts failing too, purely because it's competing
for the same limited connection budget as an unrelated, slower request
path. A bulkhead fix: give the reporting path its own separate,
smaller-capped pool (or its own read replica entirely), so it can only
ever exhaust *its own* resources, never the main request path's. The same
idea applies to worker pools: if `taskforge-worker` ran payment jobs and,
say, email-sending jobs through the *same* `concurrency`-limited pool, a
payment-service outage that fills every worker slot with retrying payment
jobs would also stop email jobs from being claimed at all — even though
email-sending has nothing to do with the payment outage. Splitting job
types across separate worker pools (or separate `concurrency` budgets per
job type) is the bulkhead pattern applied to `taskforge-worker`'s actual
architecture.

Bulkheads and circuit breakers solve related but distinct problems: a
circuit breaker stops you from *calling* a known-bad dependency; a
bulkhead limits the *blast radius* if a dependency (bad or just slow)
still manages to consume more than its fair share of a shared resource.
You generally want both — a circuit breaker to fail fast once you know
something's down, and bulkheads so that even before you know, the damage
stays contained to the path that's actually affected.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
