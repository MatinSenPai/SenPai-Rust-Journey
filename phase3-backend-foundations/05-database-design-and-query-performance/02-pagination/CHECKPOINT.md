# Checkpoint

1. `LIMIT 20 OFFSET 100000` with a perfect index on the sort key: what
   does Postgres still have to do before returning the 20 rows, and why
   does an index make it cheaper but not O(limit)?
2. Walk through the drift test's timeline yourself: 4 rows, offset page 1
   of 2, one insert at the top, offset page 2. Which exact row gets served
   twice and why? Then explain what a *delete* between pages would cause
   instead.
3. Suppose the cursor were `created_at` alone and five rows share one
   timestamp. Show concretely what goes wrong with `WHERE created_at < $1`
   after a page ends mid-tie — and what goes wrong with `<=` instead. What
   property does adding `id` restore?
4. `(created_at, id) < ($1, $2)` — expand this row-value comparison into
   its `OR`/`AND` form, and explain why the comparison's column order must
   match the `ORDER BY`'s column order.
5. The cursor encodes `timestamp_micros`, not nanoseconds, and the tests
   only ever use whole-second timestamps. What could break if the token
   carried nanosecond precision through a Postgres `TIMESTAMPTZ` round
   trip?
6. Your product manager asks for "jump to page 37" and "showing page 3 of
   412" on a keyset-paginated endpoint. What do you tell them, and which
   scheme would each feature force you back to?
