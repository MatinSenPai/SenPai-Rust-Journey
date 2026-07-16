//! Offset vs. keyset pagination over one `articles` table. The cursor
//! encode/decode functions are pure — no database, always-on tests — while
//! the two query functions are `#[ignore]`-gated like every Postgres
//! lesson in this repo. See `README.md` for why OFFSET falls apart at
//! depth (and under concurrent writes) before touching any `todo!()`.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

/// One row of the thing being paginated. `created_at` + `id` together form
/// the **sort key** for every query in this lesson: `ORDER BY created_at
/// DESC, id DESC`. The `id` tiebreaker is load-bearing — see README.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

/// A decoded cursor: the sort key of the last row the client has already
/// seen. "Give me the page *after* this point."
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub created_at: DateTime<Utc>,
    pub id: i64,
}

/// Cursor tokens come from clients, which makes them a trust boundary:
/// anything — truncated, hand-edited, from a three-month-old bookmark —
/// can show up here, and it must fail with a typed error, never a panic.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CursorError {
    #[error("cursor is not in `<timestamp_micros>:<id>` form: {0:?}")]
    Malformed(String),
    #[error("cursor timestamp {0} is out of range")]
    TimestampOutOfRange(i64),
}

/// Encodes a cursor as `"<timestamp_micros>:<id>"` — e.g.
/// `"1750000000000000:42"`.
///
/// Microseconds, not nanoseconds, deliberately: Postgres `TIMESTAMPTZ`
/// stores microsecond precision, so a nanosecond cursor would promise
/// precision that doesn't survive the database round trip. (Production
/// APIs usually base64-encode and sign this string — see README — but the
/// delimited form is the actual content either way.)
pub fn encode_cursor(cursor: Cursor) -> String {
    todo!("format! the pair as {{timestamp_micros}}:{{id}} — DateTime<Utc> has .timestamp_micros()")
}

/// Parses what `encode_cursor` produced. Everything else is an error:
/// no `:` at all, either half not an integer, or a timestamp value
/// `chrono` can't represent.
pub fn decode_cursor(token: &str) -> Result<Cursor, CursorError> {
    todo!(
        "token.split_once(':') gives Option<(&str, &str)> — None is Malformed; parse both \
         halves as i64 (parse errors are Malformed too); then \
         DateTime::from_timestamp_micros(ts) returns Option — None means TimestampOutOfRange"
    )
}

/// What an API handler calls to build the response envelope:
/// `{"articles": [...], "next_cursor": "1750000000000000:42"}`.
/// An empty page has no next cursor — the client has reached the end.
pub fn next_cursor(page: &[Article]) -> Option<String> {
    page.last().map(|last| {
        encode_cursor(Cursor {
            created_at: last.created_at,
            id: last.id,
        })
    })
}

/// Creates the articles table if needed. A real system would use
/// `sqlx::migrate!` (lesson 04.2) — this toy inlines `CREATE TABLE IF NOT
/// EXISTS` to keep the lesson to a single file, same as lesson 05.1.
pub async fn setup(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS p3_05_02_articles ( \
            id BIGSERIAL PRIMARY KEY, \
            title TEXT NOT NULL, \
            created_at TIMESTAMPTZ NOT NULL DEFAULT now() \
         )",
    )
    .execute(pool)
    .await
    .map(|_| ())
}

/// Test/seed helper: inserts an article with an explicit `created_at`, so
/// tests control the ordering instead of racing the clock.
pub async fn insert_article(
    pool: &PgPool,
    title: &str,
    created_at: DateTime<Utc>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "INSERT INTO p3_05_02_articles (title, created_at) VALUES ($1, $2) RETURNING id",
    )
    .bind(title)
    .bind(created_at)
    .fetch_one(pool)
    .await
}

/// Turns raw `query_as` tuples into `Article`s — shared by both paging
/// functions below.
pub fn rows_to_articles(rows: Vec<(i64, String, DateTime<Utc>)>) -> Vec<Article> {
    rows.into_iter()
        .map(|(id, title, created_at)| Article {
            id,
            title,
            created_at,
        })
        .collect()
}

/// Page N the Django-`Paginator` way: newest first, skip `offset` rows,
/// take `limit`. Works — and quietly does O(offset + limit) work on the
/// database side, because the skipped rows are still produced and thrown
/// away. See README for why this also duplicates/skips rows when writes
/// land between page requests.
pub async fn page_by_offset(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    todo!(
        "sqlx::query_as::<_, (i64, String, DateTime<Utc>)> over \
         SELECT id, title, created_at FROM p3_05_02_articles \
         ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2 \
         — bind limit then offset, fetch_all(pool), then rows_to_articles"
    )
}

/// The keyset version: `after` is the sort key of the last row already
/// seen (`None` for the first page). Instead of counting skipped rows,
/// the WHERE clause seeks straight to the continuation point:
///
/// ```sql
/// WHERE (created_at, id) < ($1, $2)
/// ORDER BY created_at DESC, id DESC
/// LIMIT $3
/// ```
///
/// `(a, b) < (x, y)` is Postgres row-value comparison — lexicographic,
/// exactly matching the ORDER BY, so "strictly after the cursor" is one
/// comparison, ties included.
pub async fn page_by_keyset(
    pool: &PgPool,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    todo!(
        "match on `after`: Some(cursor) runs the WHERE (created_at, id) < ($1, $2) query \
         binding cursor.created_at, cursor.id, limit; None runs the same query without the \
         WHERE clause binding only limit; both ORDER BY created_at DESC, id DESC, then \
         rows_to_articles"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use sqlx::postgres::PgPoolOptions;

    /// Whole seconds only: keeps every timestamp exactly representable in
    /// both chrono and Postgres microseconds, so cursor round trips are
    /// exact.
    fn ts(secs_after_epoch: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs_after_epoch, 0).unwrap()
    }

    const BASE: i64 = 1_750_000_000;

    // ------------------------------------------------------------------
    // Always-on tests: the cursor functions are pure — no database, no
    // #[ignore], they run on plain `cargo test -p p3-05-02-pagination`.
    // ------------------------------------------------------------------

    #[test]
    fn cursor_round_trips_through_encode_and_decode() {
        let cursor = Cursor {
            created_at: ts(1_750_000_000),
            id: 42,
        };
        let token = encode_cursor(cursor);
        assert_eq!(token, "1750000000000000:42");
        assert_eq!(decode_cursor(&token).unwrap(), cursor);
    }

    #[test]
    fn decode_rejects_tokens_that_are_not_two_integers() {
        for bad in ["", "12345", "abc:42", "1750000000000000:xyz", ":", "1:2:3"] {
            assert!(
                matches!(decode_cursor(bad), Err(CursorError::Malformed(_))),
                "should reject {bad:?}"
            );
        }
    }

    #[test]
    fn decode_rejects_a_timestamp_chrono_cannot_represent() {
        let token = format!("{}:1", i64::MAX);
        assert_eq!(
            decode_cursor(&token),
            Err(CursorError::TimestampOutOfRange(i64::MAX))
        );
    }

    #[test]
    fn next_cursor_is_none_for_an_empty_page_and_encodes_the_last_row_otherwise() {
        assert_eq!(next_cursor(&[]), None);

        let page = vec![
            Article {
                id: 3,
                title: "newer".to_string(),
                created_at: ts(300),
            },
            Article {
                id: 2,
                title: "older".to_string(),
                created_at: ts(200),
            },
        ];
        assert_eq!(
            next_cursor(&page),
            Some(encode_cursor(Cursor {
                created_at: ts(200),
                id: 2
            }))
        );
    }

    // ------------------------------------------------------------------
    // Database tests: `#[ignore]`-gated, and `#[serial]` because they all
    // wipe and reseed the one shared p3_05_02_articles table (same reason
    // as lessons 04.2/04.3 and 05.1).
    // ------------------------------------------------------------------

    async fn test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap()
    }

    async fn reset(pool: &PgPool) {
        setup(pool).await.unwrap();
        sqlx::query("DELETE FROM p3_05_02_articles")
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    #[serial(p3_05_02_pagination_db)]
    async fn offset_and_keyset_agree_when_nothing_changes_between_pages() {
        let pool = test_pool().await;
        reset(&pool).await;
        for i in 0..7 {
            insert_article(&pool, &format!("article {i}"), ts(BASE + i))
                .await
                .unwrap();
        }

        let offset_page_1 = page_by_offset(&pool, 3, 0).await.unwrap();
        let keyset_page_1 = page_by_keyset(&pool, None, 3).await.unwrap();
        assert_eq!(offset_page_1, keyset_page_1);
        assert_eq!(offset_page_1.len(), 3);
        assert_eq!(offset_page_1[0].title, "article 6"); // newest first

        let cursor = decode_cursor(&next_cursor(&keyset_page_1).unwrap()).unwrap();
        let offset_page_2 = page_by_offset(&pool, 3, 3).await.unwrap();
        let keyset_page_2 = page_by_keyset(&pool, Some(cursor), 3).await.unwrap();
        assert_eq!(offset_page_2, keyset_page_2);

        let cursor = decode_cursor(&next_cursor(&keyset_page_2).unwrap()).unwrap();
        let keyset_page_3 = page_by_keyset(&pool, Some(cursor), 3).await.unwrap();
        assert_eq!(keyset_page_3.len(), 1); // 7 rows = 3 + 3 + 1
        assert_eq!(keyset_page_3[0].title, "article 0");
    }

    /// Five articles share one `created_at`. If the cursor were
    /// `created_at` alone, "strictly older than the cursor" would skip
    /// every remaining same-timestamp row; `<=` would re-serve the whole
    /// tie group instead. The composite `(created_at, id)` key walks the
    /// tie exactly once.
    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    #[serial(p3_05_02_pagination_db)]
    async fn keyset_ties_on_created_at_are_broken_by_id_without_skips_or_repeats() {
        let pool = test_pool().await;
        reset(&pool).await;
        let same_instant = ts(BASE);
        let mut expected: Vec<i64> = Vec::new();
        for i in 0..5 {
            expected.push(
                insert_article(&pool, &format!("same instant {i}"), same_instant)
                    .await
                    .unwrap(),
            );
        }
        expected.sort_unstable();
        expected.reverse(); // id DESC

        let mut seen: Vec<i64> = Vec::new();
        let mut cursor: Option<Cursor> = None;
        loop {
            let page = page_by_keyset(&pool, cursor, 2).await.unwrap();
            if page.is_empty() {
                break;
            }
            cursor = Some(decode_cursor(&next_cursor(&page).unwrap()).unwrap());
            seen.extend(page.iter().map(|a| a.id));
        }

        assert_eq!(seen, expected);
    }

    /// The consistency failure, reproduced: a row inserted between a
    /// user's page-1 and page-2 requests shifts every OFFSET down by one,
    /// re-serving a row they already saw. The keyset cursor says "strictly
    /// older than the last row I saw," which no newer insert can disturb.
    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    #[serial(p3_05_02_pagination_db)]
    async fn offset_pages_drift_when_a_row_lands_between_requests_keyset_pages_do_not() {
        let pool = test_pool().await;
        reset(&pool).await;
        let a = insert_article(&pool, "a", ts(BASE)).await.unwrap();
        let b = insert_article(&pool, "b", ts(BASE + 1)).await.unwrap();
        let c = insert_article(&pool, "c", ts(BASE + 2)).await.unwrap();
        let d = insert_article(&pool, "d", ts(BASE + 3)).await.unwrap();

        // Page 1, both schemes: [d, c].
        let offset_page_1 = page_by_offset(&pool, 2, 0).await.unwrap();
        let keyset_page_1 = page_by_keyset(&pool, None, 2).await.unwrap();
        assert_eq!(offset_page_1, keyset_page_1);
        assert_eq!(
            offset_page_1.iter().map(|x| x.id).collect::<Vec<_>>(),
            vec![d, c]
        );

        // A new article lands between the two page requests.
        insert_article(&pool, "breaking news", ts(BASE + 100))
            .await
            .unwrap();

        // OFFSET 2 now skips [breaking, d] — so page 2 re-serves c.
        let offset_page_2 = page_by_offset(&pool, 2, 2).await.unwrap();
        assert!(
            offset_page_2.iter().any(|x| x.id == c),
            "offset page 2 should contain the duplicate row c, got {offset_page_2:?}"
        );

        // Keyset continues strictly after c: [b, a]. No duplicate.
        let cursor = decode_cursor(&next_cursor(&keyset_page_1).unwrap()).unwrap();
        let keyset_page_2 = page_by_keyset(&pool, Some(cursor), 2).await.unwrap();
        assert_eq!(
            keyset_page_2.iter().map(|x| x.id).collect::<Vec<_>>(),
            vec![b, a]
        );
    }
}
