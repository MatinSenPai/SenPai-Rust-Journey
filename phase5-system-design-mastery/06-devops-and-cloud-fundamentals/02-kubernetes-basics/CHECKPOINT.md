# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea using a real example
from this repo, not whether a test passes.

1. Name the four concrete problems this lesson says Kubernetes solves (the
   "3am crash," the placement problem, the rollout problem, and the
   service-discovery problem). For each one, explain in one sentence why
   doing it by hand doesn't scale past a small number of services.
2. `capstone-taskforge/docker-compose.yml` runs one container today
   (Postgres). Explain, specifically, why this repo's capstone does not
   need Kubernetes right now — name the real threshold from this lesson
   (not "it has a docker-compose.yml, so it's ready for the next step")
   that would actually justify adopting it.
3. Explain the difference between a Pod, a Deployment, and a Service in one
   sentence each, using your own words rather than the lesson's.
4. Why would you deliberately run `taskforge-worker` with *many* replicas
   in Kubernetes, but `taskforge-scheduler` with deliberately *one*? What
   specific property of each service's design makes multiple replicas safe
   for one and risky for the other?
5. A ConfigMap and a Secret hold the same *shape* of thing — externalized
   configuration values injected into a Pod. Why do they exist as two
   separate object types instead of one? Connect your answer to how
   `taskforge-storage`/`taskforge-api` already read `DATABASE_URL` from the
   environment in this repo today.
6. Why does the table in this lesson recommend running Postgres as a
   managed service or `StatefulSet` rather than a plain Deployment, when
   every other `taskforge-*` service is a plain Deployment? What property
   of a Pod (stated earlier in the lesson) is in direct tension with what a
   database needs?
