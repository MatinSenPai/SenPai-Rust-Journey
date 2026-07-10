# Solution

```rust
pub fn base62_encode(mut n: i64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut bytes = Vec::new();
    while n > 0 {
        bytes.push(BASE62_ALPHABET[(n % 62) as usize]);
        n /= 62;
    }
    bytes.reverse();
    String::from_utf8(bytes).unwrap()
}

pub fn base62_decode(s: &str) -> Option<i64> {
    let mut n = 0i64;
    for byte in s.bytes() {
        let index = BASE62_ALPHABET.iter().position(|&b| b == byte)?;
        n = n * 62 + index as i64;
    }
    Some(n)
}

pub async fn shorten(&self, original_url: &str) -> Result<ShortUrl, UrlShortenerError> {
    validate_url(original_url)?;

    let id: i64 = sqlx::query_scalar("SELECT nextval('p5_07_01_urls_id_seq')")
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UrlShortenerError::Storage(e.to_string()))?;
    let short_code = base62_encode(id);

    sqlx::query("INSERT INTO p5_07_01_urls (id, short_code, original_url) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(&short_code)
        .bind(original_url)
        .execute(&self.pool)
        .await
        .map_err(|e| UrlShortenerError::Storage(e.to_string()))?;

    Ok(ShortUrl { short_code, original_url: original_url.to_string() })
}
```

`resolve` and `stats` are single queries — see `src/lib.rs`, the `todo!()`
hints already spelled out the exact query text.

## Why `nextval()` before `INSERT`, not `BIGSERIAL` + `RETURNING`

`BIGSERIAL` + `RETURNING id` is the right call whenever the id only needs
to exist *after* the row does — every earlier lesson's id was purely an
opaque primary key, never fed back into anything else about the row. Here
the id **is** an input to another column (`short_code`) on the *same* row,
so it has to be known before that `INSERT` runs. Calling `nextval()` on an
explicit, named sequence first gets you that — `BIGSERIAL` is really just
Postgres syntax sugar for "create a sequence for me and default this
column to `nextval()` on insert," so this lesson is doing the same
underlying thing, just calling `nextval()` a step earlier than usual.

## The race a single `UPDATE ... RETURNING` avoids

A `SELECT original_url ... WHERE short_code = $1` followed by a separate
`UPDATE p5_07_01_urls SET click_count = click_count + 1 WHERE short_code =
$1` is a classic lost-update shape: two concurrent requests both `SELECT`
the same row (say, `click_count = 40` for both), both compute `40 + 1 =
41` in application code or in their own independent `UPDATE`, and both
write back `41` — one of the two real clicks vanishes, count ends at 41
instead of 42. `UPDATE ... SET click_count = click_count + 1 ... RETURNING
original_url` never has that gap: Postgres evaluates `click_count + 1`
against whatever the *current* row value is at the instant the row's lock
is acquired for that single statement, and a second concurrent `UPDATE` on
the same row simply waits for the first to commit before it runs — no
window where two requests could both read the same stale count.

## What `base62_decode` actually proves

It isn't part of the service's real request path at all — it exists
purely so `base62_round_trips_a_range_of_ids` can assert
`base62_decode(&base62_encode(id)) == Some(id)` across a spread of values
(0, 1, boundary values like 61/62, and large ids). That's a much stronger
correctness check on the *encoding* itself than testing `shorten`'s
end-to-end behavior alone would give you — a subtly wrong `base62_encode`
(say, an off-by-one in the alphabet index) could still happen to produce
*some* string and pass an end-to-end "did I get a non-empty code back"
test, while failing a round-trip test immediately.

## What sequential codes leak, and the fix

Comparing two consecutively-created codes reveals their *relative creation
order* (and, since the encoding is a pure deterministic function of a
monotonically increasing counter, roughly how many URLs have been
shortened in total, and how recently). For a public URL shortener where
users might not want "I made this link right after that other one" to be
inferable, or where you don't want competitors estimating your total
traffic from watching code growth over time, that's a real, if usually
minor, information leak. The fix: stop deriving the code directly from a
sequential id. Instead, generate a genuinely random string (5-7 characters
from the same base62 alphabet, checked against the table for a collision
and retried on the rare hit) OR keep the sequential id internally but
XOR/permute it through a reversible bijection (e.g. Feistel-network-based
id obfuscation) before base62-encoding — sequential internally, unordered
externally. Either avoids leaking creation order while keeping the same
external API shape.

## Does `nextval()` stay safe across 3 replicas?

Yes, with zero code changes — and this is the entire point of using a
database sequence rather than, say, an in-process `AtomicI64` counter.
`nextval()`'s "every caller gets a distinct value, no two ever collide"
guarantee comes from Postgres itself, not from anything about how many
application processes are calling it: the sequence's internal state lives
in the database, and Postgres serializes concurrent `nextval()` calls
against it regardless of which connection, process, or even which
*machine* issued them. This is the same underlying idea as Module 4's
unique-ID-generation lesson's discussion of why `taskforge-core`'s job ids
use `Uuid::new_v4()` instead of an auto-increment column — a coordinated,
shared source of truth (a database sequence here, no-coordination-needed
randomness there) is what makes id assignment safe across multiple
concurrent writers, versus an in-process counter, which would silently
double-assign ids the moment you ran a second replica.
