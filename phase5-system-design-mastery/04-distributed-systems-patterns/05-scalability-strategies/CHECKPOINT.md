# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea using a real
example from this repo, not whether a test passes.

1. `capstone-taskforge` is split into seven crates rather than one
   monolithic binary. Explain, specifically in terms of scaling, why
   `taskforge-api` and `taskforge-worker` being separate deployable units
   matters — what could you do with that split that you couldn't do if
   both lived in one process?
2. Name three signals you could auto-scale `taskforge-worker` on (CPU,
   queue depth, request rate) and explain which one is the most honest
   signal for this specific service, and why the other two would mislead
   an auto-scaler here.
3. Explain why adding Postgres read replicas would do nothing to help if
   `taskforge-storage`'s *write* throughput (not read throughput) became
   the bottleneck. What's the actual fix once that happens, and which
   lesson in this repo covers it in depth?
4. `taskforge-worker`'s `WorkerPool` has both an in-process `concurrency`
   setting and the ability to run as multiple separate OS processes.
   Explain the difference between these two forms of scaling, and why
   `claim_next`'s `FOR UPDATE SKIP LOCKED` is what makes the
   multiple-processes form safe with zero coordination code in
   `taskforge-worker` itself.
5. A read replica lagging behind its primary means a read against that
   replica can return stale data. Connect this back to the first lesson in
   this module — which consistency model does a read-replica setup
   actually provide, and why isn't it linearizable?
