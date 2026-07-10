# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea precisely and apply
it to this repo, not whether a test passes.

1. Rolling, blue-green, and canary all achieve zero-downtime deploys. What
   is the one specific failure mode each of the other two protects against
   that rolling deployment does *not*? Be concrete, not just "it's safer."
2. Blue-green's headline cost is "2x infrastructure during the switch."
   Explain exactly when that 2x actually applies — is it 2x forever, or 2x
   for some window? What determines how long that window is?
3. Canary deployment is described as having "the most operational
   complexity" of the three. Name the two concrete pieces of
   infrastructure/process it requires that rolling deployment doesn't.
4. Explain, precisely, how feature flags are *orthogonal* to the three
   deployment strategies rather than a fourth alternative to them. Could
   you use a feature flag together with a canary deployment? What would
   each one be controlling independently in that combination?
5. `.github/workflows/ci.yml` is described in this lesson as "a correctness
   gate, not a deployment pipeline." Defend that claim: walk through
   exactly what happens (and doesn't happen) to a running instance of
   `taskforge-api` when a commit passes `cargo fmt`/`clippy`/`test` on
   `main` today, versus what would need to be added for that same event to
   actually cause new code to run in production.
6. Why does this lesson recommend rolling deployment as the default for
   `taskforge-api` specifically, rather than starting with canary "to be
   safe"? Connect your answer to a specific property of `taskforge-api`'s
   design covered in an earlier lesson.
