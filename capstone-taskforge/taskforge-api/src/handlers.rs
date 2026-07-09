use crate::error::ApiError;
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use taskforge_core::{Job, JobFilter, JobId, NewJob};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct EnqueueRequest {
    pub job_type: String,
    pub payload: serde_json::Value,
    pub max_attempts: Option<u32>,
}

pub async fn enqueue_job(
    State(state): State<AppState>,
    Json(req): Json<EnqueueRequest>,
) -> Result<(StatusCode, Json<Job>), ApiError> {
    if req.job_type.trim().is_empty() {
        return Err(ApiError::bad_request("job_type must not be empty"));
    }

    let mut new_job = NewJob::new(req.job_type, req.payload);
    if let Some(max_attempts) = req.max_attempts {
        new_job = new_job.with_max_attempts(max_attempts);
    }

    let job = state.store.enqueue(new_job).await?;
    metrics::counter!("taskforge_jobs_enqueued_total").increment(1);
    Ok((StatusCode::CREATED, Json(job)))
}

pub async fn get_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Job>, ApiError> {
    let job = state
        .store
        .get(JobId(id))
        .await?
        .ok_or_else(|| ApiError::not_found(format!("job not found: {id}")))?;
    Ok(Json(job))
}

#[derive(Debug, Deserialize)]
pub struct ListParams {
    pub job_type: Option<String>,
    pub limit: Option<usize>,
}

pub async fn list_jobs(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Job>>, ApiError> {
    let jobs = state
        .store
        .list(JobFilter {
            job_type: params.job_type,
            limit: params.limit.unwrap_or(0),
        })
        .await?;
    Ok(Json(jobs))
}

pub async fn cancel_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.store.cancel(JobId(id)).await?;
    metrics::counter!("taskforge_jobs_cancelled_total").increment(1);
    Ok(StatusCode::NO_CONTENT)
}

pub async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}

pub async fn health() -> &'static str {
    "ok"
}
