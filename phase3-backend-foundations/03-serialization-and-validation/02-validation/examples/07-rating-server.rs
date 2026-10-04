//! A tiny real server: 3.2.3's "a bad rating is a 422", now with the rule on
//! the type and a field-keyed JSON body.
//! Run: `cargo run -p p3-03-02-validation --example 07-rating-server`
//! then `curl -i -X POST -H 'content-type: application/json' -d '{"score":15}' 127.0.0.1:3110/ratings`

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::post;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use validator::{Validate, ValidationErrors};

#[derive(Deserialize, Validate)]
struct NewRating {
    #[validate(range(min = 1, max = 10, message = "score must be between 1 and 10"))]
    score: u8,
}

struct ApiError(ValidationErrors);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let mut errors = serde_json::Map::new();
        for (field, list) in self.0.field_errors() {
            let msgs: Vec<_> = list
                .iter()
                .map(|e| e.message.as_deref().unwrap_or(&e.code))
                .collect();
            errors.insert(field.to_string(), json!(msgs));
        }
        (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(json!({ "errors": errors })),
        )
            .into_response()
    }
}

impl From<ValidationErrors> for ApiError {
    fn from(e: ValidationErrors) -> Self {
        ApiError(e)
    }
}

async fn create(Json(input): Json<NewRating>) -> Result<(StatusCode, String), ApiError> {
    input.validate()?;
    Ok((StatusCode::CREATED, format!("saved {}", input.score)))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ratings", post(create));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3110")
        .await
        .unwrap();
    println!("listening on 127.0.0.1:3110");
    axum::serve(listener, app).await.unwrap();
}
