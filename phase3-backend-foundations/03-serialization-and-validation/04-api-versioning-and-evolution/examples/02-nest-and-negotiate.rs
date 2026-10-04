//! Two ways to pick a version on one router: a URL prefix (`nest`) and the
//! `Accept` header. v1 also carries lifecycle headers, added by a middleware
//! that wraps only the v1 sub-router. Driven with `oneshot`, no socket.
//!
//!     cargo run -p p3-03-04-api-versioning-and-evolution --example 02-nest-and-negotiate

use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::header::{ACCEPT, VARY};
use axum::http::{HeaderMap, HeaderValue};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower::ServiceExt;

async fn v1() -> Json<Value> {
    Json(json!({"id": 1, "status": "watching"}))
}

async fn v2() -> Json<Value> {
    Json(json!({"id": 1, "watch_status": "watching", "episodes": 28}))
}

async fn negotiated(headers: HeaderMap) -> Response {
    let wants_v2 = headers
        .get(ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|a| a.contains("vnd.anime.v2+json"));
    let body = if wants_v2 { v2().await } else { v1().await };
    ([(VARY, "accept")], body).into_response()
}

async fn deprecated(req: Request, next: Next) -> Response {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert("deprecation", HeaderValue::from_static("@1767225600"));
    h.insert(
        "sunset",
        HeaderValue::from_static("Thu, 31 Dec 2026 23:59:59 GMT"),
    );
    res
}

#[tokio::main]
async fn main() {
    let v1_routes = Router::new()
        .route("/anime", get(v1))
        .layer(middleware::from_fn(deprecated));
    let app = Router::new()
        .nest("/v1", v1_routes)
        .nest("/v2", Router::new().route("/anime", get(v2)))
        .route("/anime", get(negotiated));
    for (uri, accept) in [
        ("/v1/anime", None),
        ("/v2/anime", None),
        ("/anime", None),
        ("/anime", Some("application/vnd.anime.v2+json")),
    ] {
        let mut req = Request::get(uri);
        if let Some(a) = accept {
            req = req.header(ACCEPT, a);
        }
        let res = app
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let h = |n: &str| {
            res.headers()
                .get(n)
                .map_or("-", |v| v.to_str().unwrap())
                .to_string()
        };
        let (dep, sun, vary) = (h("deprecation"), h("sunset"), h("vary"));
        let body = to_bytes(res.into_body(), usize::MAX).await.unwrap();
        println!("GET {uri} accept={accept:?}");
        println!("  deprecation={dep} sunset={sun} vary={vary}");
        println!("  {}", String::from_utf8_lossy(&body));
    }
}
