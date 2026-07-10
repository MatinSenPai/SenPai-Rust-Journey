# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can attach each term to a concrete
scenario, not recite the acronym.

1. For each letter in ACID, state the guarantee in one sentence and give a
   concrete example (can be from this lesson or invented) of what would go
   wrong if that specific letter were violated. Don't just expand the
   acronym — say what breaks.

2. Explain the difference between a non-repeatable read and a phantom read
   using two different queries against `taskforge-storage`'s `jobs` table
   as your examples (one query per anomaly). Why does Postgres's
   Repeatable Read level prevent both, even though the SQL standard
   technically only requires it to prevent the first?

3. `taskforge-storage/src/postgres.rs`'s `claim_next` runs at Postgres's
   default Read Committed level. Explain, specifically, why bumping it to
   Serializable would not close any real gap in `claim_next`'s
   correctness — what property is `FOR UPDATE SKIP LOCKED` already
   providing that Serializable would be redundant with here?

4. `AnimeStore::update` (in
   `phase3-backend-foundations/04-postgres-and-sqlx/03-anime-catalog-postgres-backed`)
   does a `self.get(id)` read, then a separate `UPDATE` later. Which of the
   three anomalies in this lesson's table (dirty read, non-repeatable read,
   phantom read) most directly describes the gap between those two
   statements? Would raising `AnimeStore::update`'s isolation level to
   Serializable fix it on its own, or would something else about how the
   method is structured also need to change? (You don't need the full
   answer yet — lesson 06 covers the fix — just reason about what
   Serializable alone would and wouldn't buy you.)

5. Someone claims "Read Committed is basically unsafe, you should always
   use Serializable in production." Using the throughput/retry trade-off
   discussed in this lesson, explain what's wrong with that as a blanket
   rule, and describe one kind of operation where Serializable genuinely
   is worth its cost.
