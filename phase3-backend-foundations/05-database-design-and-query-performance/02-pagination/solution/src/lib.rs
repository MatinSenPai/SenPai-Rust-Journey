use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Article {
    pub id: i64,
    pub title: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cursor {
    pub created_at: DateTime<Utc>,
    pub id: i64,
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CursorError {
    #[error("cursor is not in `<timestamp_micros>:<id>` form: {0:?}")]
    Malformed(String),
    #[error("cursor timestamp {0} is out of range")]
    TimestampOutOfRange(i64),
}

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

pub fn next_cursor(page: &[Article]) -> Option<String> {
    page.last().map(|last| {
        encode_cursor(Cursor {
            created_at: last.created_at,
            id: last.id,
        })
    })
}

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

pub fn rows_to_articles(rows: Vec<(i64, String, DateTime<Utc>)>) -> Vec<Article> {
    rows.into_iter()
        .map(|(id, title, created_at)| Article {
            id,
            title,
            created_at,
        })
        .collect()
}

pub async fn page_by_offset(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    let rows: Vec<(i64, String, DateTime<Utc>)> = sqlx::query_as(
        "SELECT id, title, created_at FROM p3_05_02_articles \
         ORDER BY created_at DESC, id DESC LIMIT $1 OFFSET $2",
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;
    Ok(rows_to_articles(rows))
}

pub async fn page_by_keyset(
    pool: &PgPool,
    after: Option<Cursor>,
    limit: i64,
) -> Result<Vec<Article>, sqlx::Error> {
    let rows: Vec<(i64, String, DateTime<Utc>)> = match after {
        Some(cursor) => {
            sqlx::query_as(
                "SELECT id, title, created_at FROM p3_05_02_articles \
                 WHERE (created_at, id) < ($1, $2) \
                 ORDER BY created_at DESC, id DESC LIMIT $3",
            )
            .bind(cursor.created_at)
            .bind(cursor.id)
            .bind(limit)
            .fetch_all(pool)
            .await?
        }
        None => {
            sqlx::query_as(
                "SELECT id, title, created_at FROM p3_05_02_articles \
                 ORDER BY created_at DESC, id DESC LIMIT $1",
            )
            .bind(limit)
            .fetch_all(pool)
            .await?
        }
    };
    Ok(rows_to_articles(rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use sqlx::postgres::PgPoolOptions;

    fn ts(secs_after_epoch: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs_after_epoch, 0).unwrap()
    }

    const BASE: i64 = 1_750_000_000;

    // ------------------------------------------------------------------
    // Always-on tests: no database required.
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
    // Database tests: `#[ignore]`-gated and `#[serial]` (shared table).
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

        let offset_page_1 = page_by_offset(&pool, 2, 0).await.unwrap();
        let keyset_page_1 = page_by_keyset(&pool, None, 2).await.unwrap();
        assert_eq!(offset_page_1, keyset_page_1);
        assert_eq!(
            offset_page_1.iter().map(|x| x.id).collect::<Vec<_>>(),
            vec![d, c]
        );

        insert_article(&pool, "breaking news", ts(BASE + 100))
            .await
            .unwrap();

        let offset_page_2 = page_by_offset(&pool, 2, 2).await.unwrap();
        assert!(
            offset_page_2.iter().any(|x| x.id == c),
            "offset page 2 should contain the duplicate row c, got {offset_page_2:?}"
        );

        let cursor = decode_cursor(&next_cursor(&keyset_page_1).unwrap()).unwrap();
        let keyset_page_2 = page_by_keyset(&pool, Some(cursor), 2).await.unwrap();
        assert_eq!(
            keyset_page_2.iter().map(|x| x.id).collect::<Vec<_>>(),
            vec![b, a]
        );
    }
}
