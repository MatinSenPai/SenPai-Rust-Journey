# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can walk through the actual race and fix
it two different ways, not recite "optimistic vs pessimistic."

1. Walk through the concrete A/B scenario in this lesson (rating bump vs.
   title change) step by step, and explain exactly which round trip each
   request makes and in what order, such that B's write silently reverts
   A's rating change. Where, specifically, is the "gap" that makes this
   possible?

2. Rewrite the scenario assuming `AnimeStore::update` used `SELECT ... FOR
   UPDATE` instead. Walk through the same two concurrent requests again —
   what does B actually experience differently, and why does that close
   the race?

3. Rewrite the scenario again assuming `AnimeStore::update` used a
   `version` column instead. What does B's `UPDATE` actually return in
   this scenario, and what should the calling code do in response? Is a
   silent lost update still possible with this fix — why or why not?

4. This lesson argues optimistic locking is the better choice for the
   anime catalog but pessimistic locking (specifically `FOR UPDATE SKIP
   LOCKED`) is the better choice for `taskforge-storage`'s `claim_next`.
   Explain the contention argument behind that difference in your own
   words — what's different about how often conflicts actually happen in
   each system?

5. `FOR UPDATE` (blocking) and `FOR UPDATE SKIP LOCKED` (non-blocking, skip
   to the next candidate) are both pessimistic locking. Explain why
   `AnimeStore::update`'s fix needs the blocking variant while
   `claim_next` specifically wants the skipping variant — what's different
   about what each caller actually wants when it loses the race for a row?

6. `solution/SOLUTION.md`'s closing section names this exact race as a
   deliberate, undocumented-until-now gap rather than something silently
   patched. Why do you think a curriculum would leave a real bug in
   intentionally rather than just fixing it in the starter code? What
   would be lost, pedagogically, if `AnimeStore::update` had shipped
   already using `FOR UPDATE` from the start?
