use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

pub async fn connect_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await
}

pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await.map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn database_url() -> String {
        std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test")
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn connects_and_passes_a_health_check() {
        let pool = connect_pool(&database_url()).await.unwrap();
        health_check(&pool).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "requires network access to attempt (and fail) a real connection"]
    async fn a_bad_url_fails_to_connect_instead_of_hanging() {
        let result = connect_pool("postgres://nobody:nothing@localhost:1/does-not-exist").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn many_concurrent_queries_share_a_small_pool_safely() {
        let pool = connect_pool(&database_url()).await.unwrap();

        let mut handles = Vec::new();
        for i in 0..20i64 {
            let pool = pool.clone();
            handles.push(tokio::spawn(async move {
                let row: (i64,) = sqlx::query_as("SELECT $1::bigint")
                    .bind(i)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
                row.0
            }));
        }

        let mut results: Vec<i64> = Vec::new();
        for handle in handles {
            results.push(
                tokio::time::timeout(Duration::from_secs(5), handle)
                    .await
                    .expect("query should not hang waiting for a pooled connection")
                    .unwrap(),
            );
        }
        results.sort_unstable();
        assert_eq!(results, (0..20).collect::<Vec<_>>());
    }
}
