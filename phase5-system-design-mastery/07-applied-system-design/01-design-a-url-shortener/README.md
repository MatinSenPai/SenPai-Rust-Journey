# 07.1 — Design a URL shortener

The canonical first system-design interview question — small enough to
finish in an hour, rich enough to touch encoding, database design,
concurrency, and read/write-heavy tradeoffs. Every earlier "design X"
question you'll ever get borrows some piece of this one.

## Requirements, the way an interview actually starts

- `POST /shorten {url}` → a short code, e.g. `abc123`, that redirects to
  `url`.
- `GET /{code}` → **302/303 redirect** to the original URL (not a JSON
  body — a browser needs to actually follow it).
- Codes should be short (a handful of characters, not a UUID) and
  shouldn't collide.
- Bonus: track how many times each code has been visited.

## The naive design, and where it breaks

The naive idea: hash the URL (MD5, SHA-256, whatever) and take the first
6-8 characters as the code. Two real problems: **collisions** — two
different URLs can produce colliding truncated hashes, and now you need a
collision-resolution strategy anyway — and **duplicate shortenings** of
the exact same URL always produce the exact same code, which sounds
convenient until two different users shortening the same URL leaks
that they both did.

This lesson's design sidesteps both: assign an auto-increment-style **id**
first, then encode *that id* (not the URL) into a short string. Ids are
never reused, so no organic collisions; two shortenings of the same URL
get two different ids and two different codes, same as this lesson's own
`shortening_the_same_url_twice_produces_two_distinct_codes` test expects.

## `base62_encode`: why base62, not base64 or base16

```rust
const BASE62_ALPHABET: &[u8] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
```

Hex (base16) needs ~16 characters to represent a 64-bit id; base62 needs
at most ~11. Base64 would be even shorter, but its alphabet includes `+`
and `/`, which need URL-encoding to appear safely in a path segment —
exactly the kind of easy-to-get-wrong wrinkle base62 avoids by using only
alphanumeric characters, all of which are already URL-safe on their own.
`base62_encode`/`base62_decode` are plain, pure, synchronous functions —
no database, no async — deliberately, so they're trivially unit-testable
(see this lesson's own `#[cfg(test)]` module) independent of anything
Postgres-related.

## Why a `sequence`, not `BIGSERIAL`

Every earlier Postgres lesson in this repo used `BIGSERIAL` — insert the
row, let Postgres assign the id, read it back via `RETURNING id`. That
doesn't work here: the short code needs the id **before** the row exists,
since the code itself is derived from the id, not the other way around.
`SELECT nextval('p5_07_01_urls_id_seq')` pulls the next id from an
explicit sequence first, `base62_encode`s it, then a single `INSERT`
writes the row with both the id and its already-computed code in one
statement.

## `resolve`: one atomic statement, not two

```sql
UPDATE p5_07_01_urls SET click_count = click_count + 1
WHERE short_code = $1 RETURNING original_url
```

This does the lookup *and* the click-count increment in one round trip —
deliberately, not just for efficiency. A separate `SELECT` followed by a
separate `UPDATE click_count = click_count + 1` would be exactly the
read-then-write shape that Module 2's locking lesson (`06-locking-optimistic-vs-pessimistic`)
flags as a real race under concurrent traffic — two simultaneous clicks
on the same code could both read the same starting count and both write
back "+1," losing one increment. A single `UPDATE ... RETURNING` makes
Postgres do the read-modify-write atomically, with no gap for a second
request to land in between.

## Your task

Open `src/lib.rs`. Implement `base62_encode`, `base62_decode`,
`UrlShortenerStore::shorten`, `resolve`, and `stats`.

## Checkpoint

`cargo test -p p5-07-01-design-a-url-shortener` (the base62 unit tests run
with zero infrastructure). Then, with a live Postgres:

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p5-07-01-design-a-url-shortener -- --ignored
```

Then `CHECKPOINT.md`, then `solution/SOLUTION.md`.
