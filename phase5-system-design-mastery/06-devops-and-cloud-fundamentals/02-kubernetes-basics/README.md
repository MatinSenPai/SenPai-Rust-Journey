# 06.2 — Kubernetes basics

No code in this lesson. This is a conceptual overview — no YAML to write or
read line-by-line. The goal is understanding what problem Kubernetes
solves, what its core objects mean, and — just as importantly — recognizing
when a project genuinely doesn't need it yet, using this repo's own
capstone as the concrete example either way.

## The problem: one container is easy, many containers on many machines is not

`01-docker-and-containers` covered what a single container is. Running
*one* container by hand is simple: `docker run my-image`. The hard problems
show up once you have more than a handful of containers, possibly spread
across more than one machine, and you need to keep answering questions like:

- A container crashed. Something needs to notice and restart it — right
  now, at 3am, without a human watching.
- You have 8 machines and 40 containers to run across them. Which container
  goes on which machine, given each machine has finite CPU/RAM and each
  container needs some amount of both?
- You're rolling out a new version of a service. How do you replace the old
  containers with new ones without a window where the service is
  completely down — and how do you roll back fast if the new version is
  broken?
- Container A needs to talk to container B. B might be restarted, rescheduled
  onto a different machine, or scaled from 3 replicas to 7. How does A find
  B's current address without being told about every change by hand?

Doing all of this by hand — a person or a shell script watching container
health, deciding placement, coordinating rollouts, updating some config
file every time an IP changes — doesn't scale past a small number of
services before it becomes its own full-time job, and a fragile one.
**Kubernetes (k8s)** is a system that does all four of these things
automatically, declaratively: you describe the state you want ("3 replicas
of this container, each needing this much CPU/RAM"), and Kubernetes
continuously works to make the actual state match it — restarting what
crashed, scheduling onto machines with room, rolling out changes gradually,
and giving every service a stable way to find every other service.

## When you do *not* need it yet

This is the part that's easy to skip past in a "learn Kubernetes" lesson,
so it's worth stating plainly: **this repo's own capstone does not need
Kubernetes, and adding it right now would be a net loss.**
`capstone-taskforge/docker-compose.yml` today brings up exactly one
container — Postgres — for local development; the `taskforge-*` binaries
(`taskforge-api`, `taskforge-worker`, etc.) run directly via `cargo run`
against it. Even in a hypothetical production deployment where every
`taskforge-*` crate ships as its own container, that's still a small,
fixed number of services with no need for automatic bin-packing across a
fleet of machines you don't have.

The real threshold for reaching for Kubernetes isn't "I have a
`docker-compose.yml`, what's next" — it's specific and structural:

- **You have multiple services that need to scale independently.** If
  `taskforge-api` needs 10 replicas under load but `taskforge-scheduler`
  only ever needs 1, and you want that to flex automatically, you need
  something that can scale each service on its own axis. `docker compose
  up --scale` exists but doesn't do this *automatically* based on load, and
  doesn't span more than one machine.
- **You need failure isolation and self-healing across a fleet of
  machines**, not just one. `docker-compose.yml` describes containers on a
  single Docker host — if that host goes down, everything on it goes down
  with it, and nothing brings it back automatically. Kubernetes schedules
  across a cluster of nodes, so a node dying doesn't take the whole system
  down and the scheduler places replacement Pods on the nodes that survive.
- **You need multi-machine orchestration**, full stop — once "where do
  these 40 containers run" stops fitting on one box, something has to make
  placement decisions, and that's the actual core job Kubernetes does.

A single `docker-compose.yml` running one app plus its databases — which is
exactly this repo's capstone today — is a completely legitimate, genuinely
sufficient production shape for a huge range of real services. Reaching for
Kubernetes before you have the problem it solves buys you a large amount of
operational complexity (a cluster to run, YAML to maintain, new failure
modes of Kubernetes itself to learn) in exchange for solving a problem you
don't have yet. This lesson is about understanding the tool, not about
"you should be using this."

## The core objects, conceptually

No YAML here — just what each concept *means*. When you do need Kubernetes,
these are the vocabulary you build everything else out of:

- **Pod** — the smallest deployable unit. One or more containers that are
  always scheduled together, onto the same machine, sharing a network
  namespace (so they can reach each other over `localhost`) and optionally
  storage. Most Pods run exactly one container; a Pod holding more than one
  is for cases where a second container is tightly coupled to the first
  (a log-shipping sidecar, say) and genuinely needs to live and die with
  it. A Pod is disposable — Kubernetes never "heals" a broken Pod in place,
  it kills it and schedules a brand new one.
- **Deployment** — manages a *set* of identical replica Pods and their
  lifecycle: "keep 3 healthy Pods of this image running at all times," plus
  the logic for rolling updates (replace old Pods with new ones gradually —
  this is the mechanism behind the rolling deployment strategy covered in
  `03-deployment-strategies`) and rollbacks. If a Pod crashes, the
  Deployment notices and schedules a replacement. This is the answer to
  "something crashed, who restarts it."
- **Service** — a stable network identity (a fixed DNS name / virtual IP)
  that load-balances across whichever Pods a Deployment currently has
  running. Pods come and go — crash, get replaced, get rescheduled to a
  different node during a rollout — and each new Pod gets a new internal
  IP. A Service is what lets every *other* Pod keep addressing "the API"
  by one stable name instead of tracking individual Pod IPs by hand. This
  is the answer to "how does A find B without being told about every
  change."
- **ConfigMap / Secret** — externalized configuration, injected into Pods
  as environment variables or mounted files rather than baked into the
  container image. A `ConfigMap` holds non-sensitive config (a log level, a
  feature flag); a `Secret` holds the same shape of thing for sensitive
  values (a database password, an API key) with slightly different handling
  (base64-encoded at rest, access-controlled separately). Both exist for
  exactly the reason `04-the-twelve-factor-app`'s Config factor argues for:
  the same container image should run unmodified in every environment,
  with only the externally-injected config differing between them.

## Mapping this repo's capstone onto Kubernetes objects

`capstone-taskforge`'s crates map cleanly onto this vocabulary, which is a
useful way to build intuition even though this repo doesn't actually deploy
any of it to a cluster:

| TaskForge piece | Kubernetes shape |
|---|---|
| `taskforge-api` | A Deployment (its own replica count, independently scalable — it's the stateless HTTP surface, see `04-the-twelve-factor-app`'s Processes factor) fronted by a Service, so other Pods and any external load balancer reach it by one stable name regardless of which replica actually answers. |
| `taskforge-worker` | A Deployment, but specifically one you'd want **multiple replicas** of on purpose — it's designed from the ground up to scale horizontally by having independent worker processes race to claim jobs via `SELECT ... FOR UPDATE SKIP LOCKED` (see `taskforge-storage`'s `claim_next`, and the distributed-locking section of `phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`). More replicas of `taskforge-worker` directly means more job throughput, with zero coordination needed beyond what Postgres already provides — this is close to the ideal Kubernetes horizontal-scaling case. |
| `taskforge-scheduler` | A Deployment, but one you'd deliberately keep at a **single** replica (or use a `CronJob`-style object for) — if it fires recurring jobs on a timer, multiple uncoordinated replicas would each fire the same recurring job redundantly unless it also has its own claiming logic. |
| `taskforge-admin-bot` | A Deployment with 1 replica — a Telegram bot maintaining a long-lived connection to Telegram's API doesn't benefit from being horizontally replicated the way a stateless HTTP API does. |
| `postgres` | In a real production cluster, **not** typically run as a plain Deployment yourself — a stateful database usually runs as a managed service (RDS, Cloud SQL) or, if self-hosted in-cluster, a `StatefulSet` with persistent volumes, because Pods are meant to be freely interchangeable and disposable, and a database's on-disk state is the opposite of disposable. `capstone-taskforge/docker-compose.yml`'s single Postgres container with a named volume is the local-dev analogue of this same idea — the volume, not the container, is what actually matters. |
| `DATABASE_URL`, connection strings, auth tokens | A ConfigMap (non-sensitive bits) and a Secret (the Postgres password, any API keys) — injected into every Deployment above as environment variables, exactly matching how `taskforge-storage`/`taskforge-api` already read `DATABASE_URL` from the environment rather than a hardcoded value. |

The point of this table isn't "go containerize the whole capstone" — per
the previous section, this repo's actual capstone doesn't need any of this
yet. It's that once a project *does* cross that threshold, the objects
above aren't abstract — each one has a direct, obvious counterpart to a
service you've already designed and built.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
