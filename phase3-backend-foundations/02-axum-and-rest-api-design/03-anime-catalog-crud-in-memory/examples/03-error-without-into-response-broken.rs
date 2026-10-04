//! DELIBERATELY BROKEN — expected: E0277
//!
//! A handler returns `Result<_, ShowError>`, but `ShowError` never
//! implements `IntoResponse`, so axum cannot turn the `Err` into a response.
//!
//!     cargo build -p p3-02-03-anime-catalog-crud-in-memory --example 03-error-without-into-response-broken --features broken

use axum::routing::get;
use axum::Router;

enum ShowError {
    NotFound,
}

async fn show() -> Result<&'static str, ShowError> {
    Err(ShowError::NotFound)
}

fn main() {
    let _app: Router = Router::new().route("/show", get(show));
}
