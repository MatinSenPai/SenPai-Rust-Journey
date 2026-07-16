# Load testing TaskForge

The last thing standing between "it passes tests" and "I'd put my name on it"
is knowing how it behaves under load — and being able to say, concretely, what
you'd change to take it 10×. That's the deliverable here: run the load test,
read the numbers, then write the analysis.

## What you need

- The full stack up: from the repo root,
  `docker compose -f capstone-taskforge/docker-compose.yml up --build`.
  All of `api`, `worker`, `scheduler` must be **healthy and staying up** —
  if they're crash-looping, you haven't finished the `src/main.rs` exercises
  yet (see the capstone README's "What you build" section).
- [k6](https://k6.io/docs/get-started/installation/) installed
  (`brew install k6`, `choco install k6`, or the Docker image
  `grafana/k6`).

## Running it

```sh
# defaults: BASE_URL=http://localhost:8080, API_TOKEN=dev-token
k6 run capstone-taskforge/loadtest/load.js

# or point it somewhere else / use a different token
BASE_URL=http://localhost:8080 API_TOKEN=dev-token \
  k6 run capstone-taskforge/loadtest/load.js
```

The script ramps virtual users 0 → 20 → 50 over ~2.5 minutes, enqueuing a
`send_email` job and reading it back each iteration. It **fails** (non-zero
exit) if it breaches the thresholds in `options`: <1% request errors and a
p95 request latency under 250 ms. Tighten or loosen those once you know your
own baseline — a threshold you never adjust is a threshold you don't
understand.

## What to look at

k6's end-of-run summary is the report. The lines that matter:

- **`http_req_duration` p95 / p99** — your tail latency. The average lies;
  the tail is what a real user feels. Watch how p95 moves between the steady
  20-VU stage and the 50-VU push.
- **`http_req_failed`** — anything above zero under this modest load means
  something is already the bottleneck.
- **`iterations` / `http_reqs` rate** — throughput. Compare it against
  `WORKER_CONCURRENCY` and the store's `max_connections(10)` pool cap.
- **The worker logs** (`docker compose logs -f worker`) — are jobs draining
  as fast as you enqueue them, or is the `Pending` backlog growing? Query it:
  `curl -H "Authorization: Bearer dev-token" localhost:8080/jobs | jq length`.

## The write-up (the actual deliverable)

Once you have numbers, answer these in a short `SCALE.md` of your own — this
is the "what I'd change at 10× scale" write-up the capstone asks for. Be
specific and tie each answer to something you *observed*, not something you
read:

1. **Where does job claiming contend?** The store claims jobs with
   `SELECT … FOR UPDATE SKIP LOCKED`. With N worker loops and a 10-connection
   pool, what's the real ceiling on claims/second — the lock, the pool, or
   Postgres round-trip latency? Which did you hit first?
2. **What saturates first as you turn up load** — the API's connection pool,
   `WORKER_CONCURRENCY`, or Postgres CPU/IO? How would you tell them apart
   from the metrics alone?
3. **At 10× the enqueue rate, what breaks first, and what's your fix?**
   (Bigger pool? More worker replicas — and does the `SKIP LOCKED` design let
   you just run more of them? Partition the queue by `job_type`?)
4. **Would you keep Postgres as the queue at 10×?** Name the point where you'd
   reach for Redis / NATS / Kafka instead, and the concrete symptom that would
   tell you you've reached it. (Revisit ADR 0002 — does its reasoning still
   hold at this scale?)
5. **What did you have to change about the load test itself** to trust its
   numbers (warm-up, think time, open vs. closed model)? A load test you
   didn't question is a number you can't defend.
