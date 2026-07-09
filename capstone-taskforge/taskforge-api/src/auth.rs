use crate::error::ApiError;
use crate::AppState;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

/// Simple bearer-token check — good enough for an admin/internal API,
/// nowhere near enough for a multi-tenant public one (no per-user
/// identity, no scopes, no expiry). `phase3-backend-foundations/
/// 06-auth-and-security/02-jwt-and-tower-middleware` covers a more
/// realistic JWT-based version of this same idea; TaskForge keeps it
/// simple since job-queue admin access is usually a small, trusted set of
/// callers (other backend services, an ops dashboard, this project's own
/// admin bot/CLI).
pub async fn require_bearer_token(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match header {
        Some(value) if value == format!("Bearer {}", state.auth_token) => {
            Ok(next.run(request).await)
        }
        _ => Err(ApiError {
            status: axum::http::StatusCode::UNAUTHORIZED,
            message: "missing or invalid bearer token".to_string(),
        }),
    }
}
