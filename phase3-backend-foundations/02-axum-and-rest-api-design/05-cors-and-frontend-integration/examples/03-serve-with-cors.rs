//! The lesson's API on a real socket, behind a CORS policy for one frontend
//! origin, so you can send preflights at it with `curl` exactly the way a
//! browser would. Stop it with Ctrl+C.
//!
//!     cargo run -p p3-02-05-cors-and-frontend-integration --example 03-serve-with-cors

use axum::http::{header, HeaderValue, Method};
use tower_http::cors::{AllowOrigin, CorsLayer};

use p3_02_05_cors_and_frontend_integration::app;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::list([HeaderValue::from_static(
            "http://localhost:5173",
        )]))
        .allow_methods([Method::GET, Method::POST])
        .allow_headers([header::CONTENT_TYPE]);

    let addr = "127.0.0.1:3002";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");
    println!("anime API listening on http://{addr}, CORS for http://localhost:5173 only");

    axum::serve(listener, app(cors))
        .await
        .expect("server error");
}
