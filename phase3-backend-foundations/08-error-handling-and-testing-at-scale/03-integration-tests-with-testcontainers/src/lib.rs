//! A tiny Postgres-backed repository, exercised two ways: this crate's own
//! `#[cfg(test)]` unit tests (none needed here — there's no pure logic to
//! isolate, every method is a SQL query) and `tests/integration_test.rs`'s
//! real, ephemeral-Postgres integration tests. See `README.md` for why both
//! matter and neither replaces the other.

use sqlx::PgPool;

#[derive(Debug, Clone, PartialEq, sqlx::FromRow)]
pub struct Widget {
    pub id: i64,
    pub name: String,
    pub quantity: i32,
}

/// Creates the table this repository needs. A real project would use
/// `sqlx::migrate!` (previous module) for this; a single inline
/// `CREATE TABLE` is enough for one lesson's throwaway container and keeps
/// the focus on `testcontainers`, not migration file management.
pub async fn run_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    todo!(
        "sqlx::query(\"CREATE TABLE IF NOT EXISTS p3_07_02_widgets (id BIGSERIAL PRIMARY KEY, \
         name TEXT NOT NULL, quantity INT NOT NULL)\").execute(pool).await.map(|_| ()) : {{}}"
    )
}

pub struct WidgetRepository {
    pool: PgPool,
}

impl WidgetRepository {
    pub fn new(pool: PgPool) -> Self {
        WidgetRepository { pool }
    }

    pub async fn create(&self, name: &str, quantity: i32) -> Result<Widget, sqlx::Error> {
        todo!(
            "sqlx::query_as(\"INSERT INTO p3_07_02_widgets (name, quantity) VALUES ($1, $2) \
             RETURNING id, name, quantity\").bind(name).bind(quantity).fetch_one(&self.pool)\
             .await — Widget needs #[derive(sqlx::FromRow)] added above for query_as to decode \
             directly into it: {{}}"
        )
    }

    pub async fn get(&self, id: i64) -> Result<Option<Widget>, sqlx::Error> {
        todo!(
            "sqlx::query_as(\"SELECT id, name, quantity FROM p3_07_02_widgets WHERE id = $1\") \
             .bind(id).fetch_optional(&self.pool).await : {{}}"
        )
    }

    pub async fn list(&self) -> Result<Vec<Widget>, sqlx::Error> {
        todo!(
            "sqlx::query_as(\"SELECT id, name, quantity FROM p3_07_02_widgets ORDER BY id\") \
             .fetch_all(&self.pool).await : {{}}"
        )
    }
}
