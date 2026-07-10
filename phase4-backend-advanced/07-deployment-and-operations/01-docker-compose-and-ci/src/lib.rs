use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

/// The shape `GET /health` returns. Kept deliberately tiny — a real
/// health endpoint's job is "did the process start and can it serve
/// requests," not "is every downstream dependency happy" (that's what a
/// separate `/ready` / readiness probe is for in a real deployment, see
/// the README).
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: &'static str,
}

/// No extractors, no state — this handler's entire job is to prove the
/// process is up and answering HTTP requests. `docker-compose.yml`'s
/// healthcheck (and any real orchestrator's liveness probe) polls exactly
/// this endpoint.
pub async fn health() -> Json<HealthResponse> {
    todo!("return Json(HealthResponse {{ status: \"ok\" }})")
}

/// Wires `health` onto `GET /health`. Split out from `main` (see
/// `src/main.rs`) so tests can build the router directly with no bound
/// TCP socket, the same `tower::ServiceExt::oneshot` pattern used
/// throughout Phase 3.
pub fn app() -> Router {
    todo!("Router::new().route(\"/health\", get(health))")
}

#[cfg(test)]
mod tests {
    use axum::body::{to_bytes, Body};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    use super::app;

    #[tokio::test]
    async fn health_returns_200_ok_status_json() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "ok");
    }

    #[tokio::test]
    async fn unknown_route_returns_404() {
        let response = app()
            .oneshot(Request::builder().uri("/nope").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
