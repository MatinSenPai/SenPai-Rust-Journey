use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::time::Duration;

/// Builds a connection **pool**, not a single connection. `max_connections`
/// caps how many real TCP connections to Postgres this process ever holds
/// open at once; every `.await`ed query borrows one from the pool for the
/// duration of that query and returns it immediately after — many
/// concurrent async tasks time-share a small, fixed number of real
/// connections, rather than each opening (and paying the TCP+TLS+auth
/// handshake cost of) its own. `acquire_timeout` bounds how long a caller
/// will wait for a connection (new or freed-up) before giving up, instead
/// of hanging forever against an unreachable database.
pub async fn connect_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    todo!(
        "PgPoolOptions::new().max_connections(5).acquire_timeout(Duration::from_secs(3)) \
         .connect(database_url).await — connect() both opens the pool AND eagerly makes one \
         real connection, so a bad database_url fails here, not on the first query: {{}}"
    )
}

/// The smallest possible proof-of-life query: no table, no schema
/// assumptions, just "can I still reach Postgres and get a row back."
/// Load balancers and orchestrators (Kubernetes readiness probes, a
/// `docker-compose` healthcheck) call something shaped exactly like this.
pub async fn health_check(pool: &PgPool) -> Result<(), sqlx::Error> {
    todo!("sqlx::query(\"SELECT 1\").execute(pool).await.map(|_| ()) : {{}}")
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

    /// Proves pooling actually pools: 20 concurrent queries share a pool
    /// capped at 5 connections and all still complete correctly — nobody's
    /// query is dropped or corrupted just because more callers than
    /// connections existed at once, they just queue for a free connection.
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
