# 05.2 — Pagination: offset vs. keyset

Django's `Paginator` and DRF's `PageNumberPagination` are `OFFSET`/`LIMIT`
wearing a friendly API. DRF also ships `CursorPagination`, with a docstring
that quietly admits why: offset pagination has two failure modes that only
show up once a table is big and busy. This lesson builds both schemes over
raw SQL so you can see exactly where each one breaks.

## Failure mode 1: OFFSET does the work anyway

```sql
SELECT id, title, created_at FROM articles
ORDER BY created_at DESC, id DESC
LIMIT 20 OFFSET 100000;
```

Postgres cannot teleport to row 100,001. It walks the ordering — index or
not — *produces* the first 100,000 rows, throws them away, and only then
keeps 20. An index on the sort key makes each skipped row cheap, but never
free: page 1 is O(20), page 5,000 is O(100,020). Deep pages get slower the
deeper they are, which is exactly backwards from what a crawler (or a
"jump to last page" button) does to you in production.

## Failure mode 2: pages drift under writes

Offset means "skip N rows from the top *of the current table state*."
Between a user's page-1 and page-2 requests, a new row lands at the top —
now everything shifts down one, and page 2 re-serves the last row of
page 1. A delete shifts the other way and a row is *skipped* without
anyone noticing. One of this lesson's tests
(`offset_pages_drift_when_a_row_lands_between_requests_keyset_pages_do_not`)
reproduces the duplicate for real.

## Keyset (cursor) pagination

Instead of "skip N rows," remember **the sort key of the last row the
client saw** and continue strictly after it:

```sql
SELECT id, title, created_at FROM articles
WHERE (created_at, id) < ($1, $2)
ORDER BY created_at DESC, id DESC
LIMIT $3;
```

- The `WHERE` *seeks* to the continuation point — with an index on
  `(created_at, id)` every page costs O(limit), page 1 and page 5,000
  alike. (That's lesson 05.1's B-tree earning its keep again.)
- Rows inserted above the cursor can't shift anything: "strictly older
  than what I've seen" is a fact about the data, not about a count.
- `(created_at, id) < ($1, $2)` is Postgres **row-value comparison** —
  lexicographic, tuple-style, matching the two-column `ORDER BY` exactly.
  It's shorthand for `created_at < $1 OR (created_at = $1 AND id < $2)`.

**Why the composite key?** `created_at` alone isn't unique. With ties at
the cursor's timestamp, `<` skips the remaining tied rows and `<=`
re-serves the ones already seen — either way pagination corrupts silently.
Adding `id` (unique, never null) makes the sort key total, so "after this
exact row" is unambiguous. This is precisely DRF `CursorPagination`'s
requirement that `ordering` be a "unique, unchanging" field — you're now
implementing the reason.

## Cursor tokens

The client shouldn't parse, build, or reason about `(timestamp, id)` pairs
— it gets an **opaque token** to echo back. This lesson encodes
`"<timestamp_micros>:<id>"` (a plain delimited string, no new
dependencies; microseconds because Postgres `TIMESTAMPTZ` stores exactly
that precision, so the token survives the database round trip losslessly).
Production APIs typically base64-encode the payload and **sign** it
(HMAC), so clients can't hand-craft cursors or depend on their internals —
the format stays an implementation detail you can change. Decoding is a
trust boundary either way: `decode_cursor` returns typed errors for
garbage, never panics.

An API response then carries the token for the next request:

```json
{ "articles": [ ... ], "next_cursor": "1750000000000000:42" }
```

`next_cursor` here is just "encode the last row's sort key" — `None` once
a page comes back empty. (Many APIs also omit it when a page comes back
shorter than the limit; the client stops either way.)

**The tradeoff you accept:** keyset can only walk forward (or backward,
with the comparison flipped) from a known point. No "jump to page 37," no
cheap total-page count. For infinite scroll and API list endpoints —
most of what a backend serves — that's the right trade.

## Your task

Open `src/lib.rs`. Four `todo!()`s in two pairs:

1. **`encode_cursor` / `decode_cursor`** — pure functions, no database.
   Plain `cargo test -p p3-05-02-pagination` exercises them, including the
   malformed-token cases.
2. **`page_by_offset` / `page_by_keyset`** — the two queries above,
   `sqlx::query_as` into `(i64, String, DateTime<Utc>)` tuples, through
   the provided `rows_to_articles`.

Then run the `#[ignore]`d tests against the shared `taskforge` database
(they wipe and reseed `p3_05_02_articles`, hence
`#[serial(p3_05_02_pagination_db)]` on each — same reasoning as 05.1):

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p3-05-02-pagination -- --ignored
```

## Checkpoint

`cargo test -p p3-05-02-pagination`, then `CHECKPOINT.md`, then
`solution/SOLUTION.md`.
