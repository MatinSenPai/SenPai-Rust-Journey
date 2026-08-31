# Solution

```rust
pub fn encode_cursor(cursor: Cursor) -> String {
    format!("{}:{}", cursor.created_at.timestamp_micros(), cursor.id)
}

pub fn decode_cursor(token: &str) -> Result<Cursor, CursorError> {
    let malformed = || CursorError::Malformed(token.to_string());
    let (ts, id) = token.split_once(':').ok_or_else(malformed)?;
    let ts: i64 = ts.parse().map_err(|_| malformed())?;
    let id: i64 = id.parse().map_err(|_| malformed())?;
    let created_at =
        DateTime::from_timestamp_micros(ts).ok_or(CursorError::TimestampOutOfRange(ts))?;
    Ok(Cursor { created_at, id })
}
```

```rust
pub async fn page_by_keyset(
    pool: &PgPool,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    let rows: Vec<(i64, String, DateTime<Utc>)> = match after {
        Some(cursor) => sqlx::query_as(
            "SELECT id, title, created_at FROM p3_05_02_articles \
             WHERE (created_at, id) < ($1, $2) \
             ORDER BY created_at DESC, id DESC LIMIT $3",
        )
        .bind(cursor.created_at)
        .bind(cursor.id)
        .bind(limit)
        .fetch_all(pool)
        .await?,
        None => sqlx::query_as(
            "SELECT id, title, created_at FROM p3_05_02_articles \
             ORDER BY created_at DESC, id DESC LIMIT $1",
        )
        .bind(limit)
        .fetch_all(pool)
        .await?,
    };
    Ok(rows_to_articles(rows))
}
```

(`page_by_offset` is the same `SELECT` with `LIMIT $1 OFFSET $2` and no
`WHERE` — see `src/lib.rs`.)

## The cursor functions: small, but the edges matter

`split_once(':')` does the structural check in one call: no delimiter →
`None` → `Malformed`. Note that `"1:2:3"` still fails — `split_once`
yields `("1", "2:3")` and `"2:3"` doesn't parse as `i64` — without any
special-casing. The closure `malformed` exists because the error needs an
owned copy of the token in three places; building it once keeps the three
failure sites identical.

`DateTime::from_timestamp_micros` returning `Option` is chrono being
honest: not every `i64` is a representable instant. A hand-crafted (or
bit-rotted) token with `i64::MAX` micros must become a typed error, not a
panic — cursor decoding is a trust boundary, exactly like deserializing a
request body in the serde lessons. The same reasoning says *microseconds*,
not nanos: Postgres `TIMESTAMPTZ` stores microsecond precision, so a
nanosecond token would encode detail the database round trip silently
truncates — and then `WHERE (created_at, id) < (cursor...)` could disagree
with the row the cursor was built from, off-by-one-ing the page boundary.

A production API would wrap this exact string in base64 and an HMAC
signature. That changes *packaging*, not content: opaque so clients can't
couple to the format, signed so they can't forge a cursor pointing at
rows they shouldn't page through. The delimited string is the honest core
of every cursor scheme you'll meet.

## The row-value comparison

`(created_at, id) < ($1, $2)` is lexicographic:

```sql
created_at < $1 OR (created_at = $1 AND id < $2)
```

The tuple's column order matches `ORDER BY created_at DESC, id DESC`,
which is what makes "strictly after the cursor row in scan order" a
single comparison — including inside a run of equal `created_at` values,
where the `id` half takes over. That's the whole fix for ties: with
`created_at` alone, ending a page mid-tie forces a choice between `<`
(skips the rest of the tie group) and `<=` (re-serves all of it). The
`keyset_ties_..._without_skips_or_repeats` test walks five same-timestamp
rows two at a time to prove the composite key threads through a tie
correctly.

Postgres can also use an index on `(created_at, id)` to satisfy both the
row-value `WHERE` and the `ORDER BY` in one backward index scan — each
page is O(limit) work regardless of depth, versus OFFSET's
O(offset + limit). At lesson scale you won't feel it; `EXPLAIN ANALYZE`
(lesson 05.1) on a few million generated rows makes it unmissable.

## What the drift test proves

OFFSET's contract is positional: "skip 2 from the top, *whatever the top
is now*." The insert between requests redefines the top, so row `c` — last
on page 1 — reappears on page 2. The cursor's contract is relational:
"strictly older than `c`," and no insert above `c` can change what's older
than `c`. Same table, same moment, same page size: offset duplicates,
keyset doesn't. Deletes cause the mirror-image failure for offset (a row
silently *skipped*), while keyset just no longer serves the deleted row.

Worth knowing what keyset gives up: no random access ("page 37") and no
cheap "page 3 of 412" — both are inherently positional concepts, and
positions are exactly what a moving table doesn't have. DRF makes the
same split: `PageNumberPagination` when you need page numbers and accept
drift, `CursorPagination` when you need stable, deep, fast pages.
