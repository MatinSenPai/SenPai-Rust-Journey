# Checkpoint

1. Why does `shorten` pull an id from `nextval('p5_07_01_urls_id_seq')`
   *before* inserting the row, instead of using a `BIGSERIAL` column and
   reading the id back afterward with `RETURNING id` — the pattern every
   earlier Postgres lesson in this repo used?
2. `resolve` increments `click_count` inside the same `UPDATE ...
   RETURNING` statement that looks up `original_url`. Walk through
   concretely what could go wrong if `resolve` instead ran a `SELECT` to
   get `original_url` and a separate `UPDATE click_count = click_count +
   1` — under what condition would a click get lost?
3. `base62_decode` is never called anywhere in `shorten`/`resolve`/`stats`
   — lookups go straight to the database by the `short_code` string. Why
   write it at all? What's it actually testing?
4. This design assigns short codes sequentially (id 1 → code "1", id 2 →
   code "2", ...). What does that leak to anyone who shortens two URLs in
   a row and compares their codes? Is that a real problem for this
   service, and if you wanted codes that didn't reveal creation order,
   what would you change (you don't need to implement it — just describe
   the shape of the fix)?
5. If this service needed to run as 3 replicas behind a load balancer
   (Module 4's scalability-strategies lesson), would the `nextval()`-based
   id assignment still be safe with zero code changes? Why or why not —
   what's actually providing the "no two callers ever get the same id"
   guarantee here, and does it care how many application processes are
   calling it?
