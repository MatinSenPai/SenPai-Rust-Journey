use sqlx::PgPool;

/// Runs every migration file under `./migrations/` that hasn't already
/// been applied to this database, in filename order. `sqlx::migrate!`
/// embeds the migration SQL into the compiled binary at compile time (via
/// `include_str!` internally) — a deployed binary carries its own
/// migrations with it, no separate SQL files need to ship alongside it.
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    todo!("sqlx::migrate!(\"./migrations\").run(pool).await : {{}}")
}

pub async fn insert_widget(pool: &PgPool, name: &str) -> Result<i64, sqlx::Error> {
    todo!(
        "sqlx::query_scalar(\"INSERT INTO p3_04_02_widgets (name) VALUES ($1) RETURNING id\") \
         .bind(name).fetch_one(pool).await : {{}}"
    )
}

pub async fn count_widgets(pool: &PgPool) -> Result<i64, sqlx::Error> {
    todo!(
        "sqlx::query_scalar(\"SELECT COUNT(*) FROM p3_04_02_widgets\").fetch_one(pool).await : {{}}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgPoolOptions;

    async fn test_pool() -> PgPool {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn migrating_creates_the_widgets_table() {
        let pool = test_pool().await;
        // Clean slate: this test may run after earlier runs already
        // created the table and its migration-tracking row.
        sqlx::query("DROP TABLE IF EXISTS p3_04_02_widgets")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("DELETE FROM _sqlx_migrations WHERE version = 1")
            .execute(&pool)
            .await
            .ok(); // fine if `_sqlx_migrations` itself doesn't exist yet

        run_migrations(&pool).await.unwrap();

        let count = count_widgets(&pool).await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn running_migrations_twice_is_a_safe_no_op() {
        let pool = test_pool().await;

        run_migrations(&pool).await.unwrap();
        // If this errored (e.g. "relation already exists"), the migrator
        // wouldn't be tracking applied migrations correctly.
        run_migrations(&pool).await.unwrap();
    }

    #[tokio::test]
    #[ignore = "requires a live Postgres reachable at DATABASE_URL"]
    async fn inserted_widgets_are_counted() {
        let pool = test_pool().await;
        run_migrations(&pool).await.unwrap();

        let before = count_widgets(&pool).await.unwrap();
        insert_widget(&pool, "test widget").await.unwrap();
        let after = count_widgets(&pool).await.unwrap();

        assert_eq!(after, before + 1);
    }
}
