# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can name the exact place in this repo each
factor shows up (or honestly doesn't), not whether a test passes.

1. Pick three factors from this lesson and, for each, name the specific
   file or crate in this repo that already follows it, and explain *how*
   in one or two sentences — don't just restate the factor's name.
2. `taskforge-core`'s `JobStore` trait, with `PostgresJobStore` and
   `InMemoryJobStore` as two implementations, is this lesson's example for
   the Backing Services factor. Explain specifically what would have to
   change in `taskforge-worker` or `taskforge-api`'s code if you swapped
   which implementation was wired in — and why the answer is "nothing."
3. The Processes factor (stateless processes) is described in this lesson
   as "a direct rerun of a lesson you already had." Which earlier lesson,
   and what specific claim about `AppState` does it make that this lesson
   just re-names?
4. This lesson names two honest gaps: Port binding and the last mile of
   Disposability. For each one, explain precisely what's missing — not
   "it's not done," but the specific line of code or wiring that doesn't
   exist yet — and what the minimal fix would look like.
5. Explain the Logs factor ("treat logs as event streams") in your own
   words, then explain why `tracing_subscriber::fmt::init()` in
   `taskforge-admin-bot/src/main.rs` satisfies it while a hypothetical
   version that opened a file and wrote log lines to it by hand would not.
6. Build/release/run separation says you should never be able to change
   code as part of "running" it. Walk through `phase4-backend-advanced/07-deployment-and-operations/01-docker-compose-and-ci`'s
   `Dockerfile` and identify exactly which stage is "build," which
   artifact represents "release," and what command constitutes "run" —
   and explain what it would mean to violate this separation (give a
   concrete example of what *not* separating them would look like).
