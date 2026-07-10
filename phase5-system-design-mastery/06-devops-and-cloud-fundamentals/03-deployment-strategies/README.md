# 06.3 — Deployment strategies

No code in this lesson. `02-kubernetes-basics` covered *what* orchestrates
rolling out a new version of a service; this lesson covers the actual menu
of *strategies* for doing that rollout, what each one costs and buys you,
and where this repo's own CI pipeline honestly stands relative to actually
shipping any of them.

## Rolling deployment

Replace old instances with new ones **gradually**, a few at a time, rather
than all at once. Take instance 1 down, bring up a new-version instance 1,
wait for it to report healthy, move to instance 2, repeat. This is
Kubernetes's Deployment default (`02-kubernetes-basics` covered what a
Deployment is): you tell it "run the new image," and it works through the
replica set incrementally, using the same kind of readiness signal
described in the previous lessons — a Pod that's started but not yet
answering "ready to serve traffic" isn't cut over into rotation yet.

- **Cost:** low — no extra infrastructure, you're reusing the same fleet
  you already have, just cycling instances through it one (or a few) at a
  time.
- **What it buys you:** zero-downtime deploys, and if the new version is
  visibly broken (crashes on startup, fails its readiness check), the
  rollout can pause or auto-abort before *every* instance has been
  replaced — so a bad deploy affects some fraction of capacity, not all of
  it, and default tooling can automatically pause a rollout on repeated
  failures.
- **What it doesn't buy you:** during the rollout, old and new versions are
  **both** serving live traffic simultaneously, for however long the
  rollout takes. If old and new aren't compatible in some way that
  matters — a changed API response shape a client can't handle yet, a
  database migration the old code can't tolerate — a rolling deploy is
  exactly the strategy that will surface that incompatibility as
  intermittent, confusing failures depending on which instance happened to
  answer a given request. It's also comparatively slow to *fully* roll
  back — reversing a rolling deploy is itself another rolling deploy, in
  the other direction, taking about as long as the forward one did.

## Blue-green deployment

Run **two complete environments** — call them "blue" (currently live) and
"green" (the new version) — at full production capacity simultaneously.
Deploy the new version entirely to green while blue keeps serving all live
traffic, untouched. Once green is verified healthy, flip whatever's routing
traffic (a load balancer, a DNS record) to point at green **all at once**.
Blue stays up, idle, for some window afterward specifically so rollback is
just flipping the switch back.

- **Cost:** high, and it's a specific, named cost — **you're running 2x
  the infrastructure** during the switch (both environments fully
  provisioned and running at the same time), even if only briefly. For a
  service with meaningfully-sized compute costs, that's a real line item,
  not a footnote.
- **What it buys you:** the fastest possible rollback of any of these
  strategies — reverting is a routing change, not a redeploy, so it's
  effectively instant. It also sidesteps the rolling deployment's
  "old and new both live at once, unpredictably" problem: at any given
  moment, *all* traffic is going to exactly one version, never a mix.
- **What it doesn't buy you:** it doesn't protect you from a bug that only
  shows up under real production traffic patterns or scale — green gets
  100% of traffic the instant it's live, with no gradual exposure. If the
  new version has a problem that only manifests under real load (a memory
  leak that takes an hour to matter, a race condition that only shows up
  at real concurrency), blue-green finds out at the same moment every user
  does.

## Canary deployment

Route a **small percentage** of live traffic (1%, 5%, 10% — the actual
number varies) to the new version first, while the rest keeps hitting the
old one. Watch error rates, latency, and whatever else you monitor on that
small slice. If it looks healthy, gradually increase the percentage — 5%,
then 25%, then 100% — pulling the new version fully live only once each
step has proven itself. If something looks wrong at any step, you route
back to 0% for the new version, having only ever exposed a small fraction
of users to the problem.

- **Cost:** the **most operational complexity** of the four — you need
  traffic-splitting infrastructure capable of routing by percentage (not
  just "old or new"), and you need automated (or at least fast, attentive)
  monitoring to actually decide whether to keep advancing the percentage —
  a canary strategy nobody is watching doesn't buy you anything over a
  rolling deploy.
- **What it buys you:** the **lowest blast radius** of any of these
  strategies. A bug that would affect 100% of users under blue-green
  affects at most whatever percentage you'd advanced to before catching
  it — potentially just 1% of traffic, for however long it takes your
  monitoring to notice and roll back.
- **When it's worth the complexity:** once you have real production
  traffic and something meaningful to lose from a bad deploy — canary is
  the strategy of choice specifically because it catches regressions
  before they hit *everyone*, at the cost of needing the traffic-splitting
  and monitoring machinery to make that percentage-based rollout
  meaningful in the first place. Below that traffic/stakes threshold, the
  operational overhead isn't buying you much a rolling deploy wouldn't
  already give you.

## Feature flags: an orthogonal technique

Everything above is about **how you deploy** — moving code from "not
running in production" to "running in production." Feature flags solve a
different, related problem: they **decouple deploy from release**. Code
ships to production *dark* — merged, built, deployed, running on every
instance — behind a flag that's off by default. Turning the feature on for
some users, all users, or nobody is then a config change (flip the flag),
completely independent of any of the deployment strategies above, and
reversible in the time it takes that config change to propagate, not the
time it takes to redeploy.

This matters because "deploy" and "release" are genuinely different
events, and conflating them is the source of a lot of deployment risk:
without a flag, "ship the code" and "expose users to the new behavior"
happen at exactly the same moment, so if the new behavior is wrong, your
*only* lever is redeploying (or rolling back a deploy) to fix it. With a
flag, you can deploy the new code during low-risk hours, days before
anyone sees it, verify nothing about the deploy itself broke, and *then*
separately decide when and for whom to turn the behavior on — and turn it
back off instantly if it's wrong, without touching the deployment pipeline
at all. Feature flags compose with any of the three strategies above: a
canary rollout can *also* gate the new behavior behind a flag, giving you
two independent dials (which instances are running the new binary; which
users see the new behavior) instead of one.

## What this repo's CI actually is — and isn't

`.github/workflows/ci.yml` runs three things on every push: `cargo fmt --all
-- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and
`cargo test --workspace`. Read plainly, that's a **correctness gate** — it
answers "is this code well-formatted, free of clippy's known footguns, and
behaviorally correct against its tests," and it blocks a merge if the
answer is no. That is genuinely valuable, and it's real CI (continuous
*integration* — verifying every change is good *before* it merges) — but
it is honestly **not** a deployment pipeline. Nothing in that file builds a
container, pushes an image anywhere, or causes a single running instance
of `taskforge-api` (or anything else in this repo) to change what code it's
running. `check` passing means "this commit is safe to build," not "this
commit is now live."

`phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`'s
README already sketches exactly what you'd bolt on top to close that gap: a
second job, gated with `needs: check` and `if: github.ref ==
'refs/heads/main'`, that builds the Docker image and pushes it to a
registry tagged by commit SHA:

```yaml
build-and-push:
  needs: check
  if: github.ref == 'refs/heads/main'
  steps:
    - uses: docker/build-push-action@v6
      with:
        push: true
        tags: ghcr.io/your-org/your-service:${{ github.sha }}
```

That job gets you a built, pushed, addressable artifact — "this exact
commit, as a runnable image" — but *still* isn't deployment on its own.
Deployment is the step after that: something (a human running `kubectl
set image`, or a CD tool watching the registry, or a Kubernetes Deployment
manifest referencing the new tag) actually causing that image to replace
what's currently running, using one of the strategies above. This repo
stops at "build a known-good, addressable image" deliberately — actually
wiring up a deploy target is out of scope for a curriculum repo with no
real cluster or cloud account behind it, but the shape of the missing piece
is exactly the CD job above plus one of rolling/blue-green/canary applied
to whatever's actually running the image.

## Which strategy for `taskforge-api`?

**Rolling is the sane default.** It's the lowest-cost of the three (no 2x
infrastructure, no traffic-splitting layer to build), it's what Kubernetes
gives you for free as the out-of-the-box Deployment behavior covered in
`02-kubernetes-basics`, and `taskforge-api` is specifically well-suited to
it: it's stateless (every handler re-derives everything from `Arc<dyn
JobStore>`, per the horizontal-scaling argument in
`phase4-backend-advanced/06-system-design-fundamentals/01-cap-scaling-lb-idempotency-locking`),
so old and new instances serving traffic side-by-side during the rollout
isn't a coordination problem the way it would be for a service holding
in-memory state that old and new instances might disagree about.

**Canary earns its complexity later** — specifically once there's real
production traffic and something to actually lose from a bad deploy. The
whole value of canary is catching a regression on 1% of traffic before it
hits 100%, which only matters once "100%" is a number worth protecting.
For a service that's mid-development, low-traffic, or still finding its
actual usage pattern, the traffic-splitting and monitoring machinery canary
needs is overhead you'd be paying for without yet getting the benefit it's
designed to provide. The sane order is: ship on rolling deploys first, and
reach for canary once "a bad deploy might hit real users at real scale" is
a genuine, not hypothetical, risk.

## Checkpoint

No `cargo test` for this lesson — go straight to `CHECKPOINT.md`.
