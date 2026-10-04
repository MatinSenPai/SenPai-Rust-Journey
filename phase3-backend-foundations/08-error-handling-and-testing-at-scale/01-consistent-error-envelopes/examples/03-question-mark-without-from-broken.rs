//! DELIBERATELY BROKEN — expected: E0277
//! Run `cargo run -p p3-08-01-consistent-error-envelopes --example 03-question-mark-without-from-broken --features broken`
//! and read the error. `?` needs a `From` impl to convert the rejection.

use axum::extract::rejection::JsonRejection;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;

#[derive(Deserialize)]
struct NewShow {
    title: String,
}

struct ApiError;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        StatusCode::BAD_REQUEST.into_response()
    }
}

async fn create(body: Result<Json<NewShow>, JsonRejection>) -> Result<String, ApiError> {
    let Json(show) = body?;
    Ok(show.title)
}

fn main() {
    let _app: Router = Router::new().route("/shows", post(create));
}
