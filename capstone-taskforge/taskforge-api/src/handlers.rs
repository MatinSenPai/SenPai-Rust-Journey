use crate::error::{ApiError, ErrorBody};
use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;
use taskforge_core::{Job, JobFilter, JobId, NewJob};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Deserialize, ToSchema)]
pub struct EnqueueRequest {
    /// Which registered handler should run this job (e.g. `send_email`) —
    /// the worker dead-letters jobs whose type nothing handles.
    pub job_type: String,
    /// Arbitrary JSON, handed to the handler untouched.
    pub payload: serde_json::Value,
    /// Retry budget; defaults to 5 when omitted.
    pub max_attempts: Option<u32>,
}

#[utoipa::path(
    post,
    path = "/jobs",
    tag = "jobs",
    request_body = EnqueueRequest,
    responses(
        (status = 201, description = "Job persisted in `Pending` status", body = Job),
        (status = 400, description = "`job_type` is empty", body = ErrorBody),
        (status = 401, description = "Missing or invalid bearer token", body = ErrorBody)
    ),
    security(("bearer_token" = []))
)]
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

#[utoipa::path(
    get,
    path = "/jobs/{id}",
    tag = "jobs",
    params(("id" = Uuid, Path, description = "Job id")),
    responses(
        (status = 200, description = "The job", body = Job),
        (status = 404, description = "No job with this id", body = ErrorBody),
        (status = 401, description = "Missing or invalid bearer token", body = ErrorBody)
    ),
    security(("bearer_token" = []))
)]
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

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListParams {
    /// Only return jobs of this type.
    pub job_type: Option<String>,
    /// Cap the result count; 0 or omitted means the store default (50).
    pub limit: Option<usize>,
}

#[utoipa::path(
    get,
    path = "/jobs",
    tag = "jobs",
    params(ListParams),
    responses(
        (status = 200, description = "Matching jobs, newest first", body = Vec<Job>),
        (status = 401, description = "Missing or invalid bearer token", body = ErrorBody)
    ),
    security(("bearer_token" = []))
)]
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

#[utoipa::path(
    post,
    path = "/jobs/{id}/cancel",
    tag = "jobs",
    params(("id" = Uuid, Path, description = "Job id")),
    responses(
        (status = 204, description = "Cancelled — recorded as `Failed` with error \"cancelled by user\""),
        (status = 404, description = "No job with this id", body = ErrorBody),
        (status = 409, description = "Job is `Running` or already terminal — not cancellable", body = ErrorBody),
        (status = 401, description = "Missing or invalid bearer token", body = ErrorBody)
    ),
    security(("bearer_token" = []))
)]
pub async fn cancel_job(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.store.cancel(JobId(id)).await?;
    metrics::counter!("taskforge_jobs_cancelled_total").increment(1);
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/metrics",
    tag = "ops",
    responses(
        (status = 200, description = "Prometheus text exposition format", body = String, content_type = "text/plain")
    )
)]
pub async fn metrics_handler(State(state): State<AppState>) -> String {
    state.metrics_handle.render()
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "ops",
    responses(
        (status = 200, description = "Service is up — body is literally `ok`", body = str, content_type = "text/plain")
    )
)]
pub async fn health() -> &'static str {
    "ok"
}
