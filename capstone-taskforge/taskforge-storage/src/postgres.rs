use crate::row::{status_columns, JobRow};
use async_trait::async_trait;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use taskforge_core::{Job, JobError, JobFilter, JobId, JobStatus, JobStore, NewJob};

/// The real, Postgres-backed `JobStore`. See
/// `../docs/adr/0002-postgres-backed-queue.md` for why Postgres, and
/// `README.md` for why every query here is written with the runtime
/// `sqlx::query`/`query_as` API rather than the compile-time-checked
/// `query!`/`query_as!` macros.
pub struct PostgresJobStore {
    pool: PgPool,
}

impl PostgresJobStore {
    pub fn new(pool: PgPool) -> Self {
        PostgresJobStore { pool }
    }

    /// Connects and runs migrations. Only ever called from `#[ignore]`d
    /// integration tests or a real `main.rs` — never from a test that runs
    /// as part of the default `cargo test` (see `README.md`).
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await?;
        sqlx::migrate!("./migrations").run(&pool).await?;
        Ok(PostgresJobStore { pool })
    }
}

#[async_trait]
impl JobStore for PostgresJobStore {
    async fn enqueue(&self, new_job: NewJob) -> Result<Job, JobError> {
        let job = Job::from_new(new_job);
        let (status, attempt, next_attempt_at, error) = status_columns(&job.status);

        sqlx::query(
            "INSERT INTO jobs (id, job_type, payload, status, attempt, next_attempt_at, error, \
             attempts, max_attempts, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
        )
        .bind(job.id.0)
        .bind(&job.job_type)
        .bind(&job.payload)
        .bind(status)
        .bind(attempt)
        .bind(next_attempt_at)
        .bind(&error)
        .bind(job.attempts as i32)
        .bind(job.max_attempts as i32)
        .bind(job.created_at)
        .bind(job.updated_at)
        .execute(&self.pool)
        .await
        .map_err(|e| JobError::Storage(e.to_string()))?;

        Ok(job)
    }

    async fn claim_next(&self) -> Result<Option<Job>, JobError> {
        // The CTE selects one claimable row (pending, or retrying whose
        // next_attempt_at has passed), locking it with FOR UPDATE SKIP
        // LOCKED so concurrent workers never claim the same row — see
        // phase4-backend-advanced/03-background-jobs-and-message-queues/
        // 01-postgres-skip-locked-toy-queue for the small-scale rehearsal
        // of exactly this pattern.
        let row: Option<JobRow> = sqlx::query_as(
            "WITH claimed AS ( \
                SELECT id FROM jobs \
                WHERE status = 'pending' \
                   OR (status = 'retrying' AND next_attempt_at <= now()) \
                ORDER BY created_at \
                FOR UPDATE SKIP LOCKED \
                LIMIT 1 \
             ) \
             UPDATE jobs SET status = 'running', updated_at = now() \
             WHERE id IN (SELECT id FROM claimed) \
             RETURNING *",
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| JobError::Storage(e.to_string()))?;

        Ok(row.map(JobRow::into_job))
    }

    async fn mark_succeeded(&self, id: JobId) -> Result<(), JobError> {
        sqlx::query("UPDATE jobs SET status = 'succeeded', updated_at = now() WHERE id = $1")
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| JobError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn mark_failed(
        &self,
        id: JobId,
        error: String,
        next_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<(), JobError> {
        let job = self
            .get(id)
            .await?
            .ok_or_else(|| JobError::NotFound(id.to_string()))?;
        let attempts = job.attempts + 1;

        let status = if let Some(next_attempt_at) = next_attempt_at {
            if attempts < job.max_attempts {
                JobStatus::Retrying {
                    attempt: attempts,
                    next_attempt_at,
                }
            } else {
                JobStatus::DeadLetter {
                    error: error.clone(),
                }
            }
        } else {
            JobStatus::DeadLetter {
                error: error.clone(),
            }
        };
        let (status_str, attempt, next_attempt_at, err_col) = status_columns(&status);

        sqlx::query(
            "UPDATE jobs SET status = $1, attempt = $2, next_attempt_at = $3, error = $4, \
             attempts = $5, updated_at = now() WHERE id = $6",
        )
        .bind(status_str)
        .bind(attempt)
        .bind(next_attempt_at)
        .bind(err_col)
        .bind(attempts as i32)
        .bind(id.0)
        .execute(&self.pool)
        .await
        .map_err(|e| JobError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn cancel(&self, id: JobId) -> Result<(), JobError> {
        let job = self
            .get(id)
            .await?
            .ok_or_else(|| JobError::NotFound(id.to_string()))?;

        if !job.status.is_cancellable() {
            return Err(JobError::NotCancellable(format!("{:?}", job.status)));
        }

        let (status, _, _, error) = status_columns(&JobStatus::Failed {
            error: "cancelled by user".to_string(),
        });
        sqlx::query("UPDATE jobs SET status = $1, error = $2, updated_at = now() WHERE id = $3")
            .bind(status)
            .bind(error)
            .bind(id.0)
            .execute(&self.pool)
            .await
            .map_err(|e| JobError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn get(&self, id: JobId) -> Result<Option<Job>, JobError> {
        let row: Option<JobRow> = sqlx::query_as("SELECT * FROM jobs WHERE id = $1")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| JobError::Storage(e.to_string()))?;
        Ok(row.map(JobRow::into_job))
    }

    async fn list(&self, filter: JobFilter) -> Result<Vec<Job>, JobError> {
        let limit = if filter.limit == 0 { 50 } else { filter.limit };

        let rows: Vec<JobRow> = if let Some(job_type) = filter.job_type {
            sqlx::query_as(
                "SELECT * FROM jobs WHERE job_type = $1 ORDER BY created_at DESC LIMIT $2",
            )
            .bind(job_type)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as("SELECT * FROM jobs ORDER BY created_at DESC LIMIT $1")
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await
        }
        .map_err(|e| JobError::Storage(e.to_string()))?;

        Ok(rows.into_iter().map(JobRow::into_job).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Requires a real local Postgres. Run with:
    ///   docker compose -f ../docker-compose.yml up -d postgres
    ///   DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
    ///     cargo test -p taskforge-storage -- --ignored
    #[tokio::test]
    #[ignore]
    async fn enqueue_and_claim_round_trip() {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        let store = PostgresJobStore::connect(&database_url).await.unwrap();

        let job = store
            .enqueue(NewJob::new(
                "send_email",
                serde_json::json!({"to": "a@b.com"}),
            ))
            .await
            .unwrap();
        assert_eq!(job.status, JobStatus::Pending);

        let claimed = store.claim_next().await.unwrap().unwrap();
        assert_eq!(claimed.id, job.id);
        assert_eq!(claimed.status, JobStatus::Running);

        store.mark_succeeded(job.id).await.unwrap();
        let fetched = store.get(job.id).await.unwrap().unwrap();
        assert_eq!(fetched.status, JobStatus::Succeeded);
    }
}
