//! Exercises for 3.8.2 — Request tracing and correlation IDs.
//!
//! One id per request: accepted from the caller if it is sane, generated
//! otherwise; stored in the request extensions, attached to a tracing span so
//! every log line carries it, echoed on the response, and written into the
//! error body.

use std::io;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::{Extension, Path, Request};
use axum::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::middleware::{from_fn, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tracing::Instrument;
use tracing_subscriber::fmt::MakeWriter;

/// The header that carries the id, in both directions.
pub const REQUEST_ID_HEADER: &str = "x-request-id";

/// The longest id this service accepts from a caller, in bytes.
pub const MAX_REQUEST_ID_LEN: usize = 64;

/// The id of the request being served. The middleware puts one of these in the
/// request's extensions, so a handler can ask for it with `Extension<RequestId>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequestId(pub String);

/// A fresh id: a random UUID in its hyphenated form, 36 characters long
/// (`67e55044-10b1-426f-9247-bb680e5fe0c8`).
pub fn new_request_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ------------------------------------------------------------------ LogBuffer
//
// Given, complete. A tracing subscriber that writes into memory instead of the
// terminal, so a test can read the log back and assert on it.

/// A shared in-memory byte buffer that tracing can write log lines into.
#[derive(Clone, Default)]
pub struct LogBuffer(Arc<Mutex<Vec<u8>>>);

impl LogBuffer {
    pub fn new() -> Self {
        LogBuffer::default()
    }

    /// Everything logged so far, as text.
    pub fn contents(&self) -> String {
        String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
    }

    /// A subscriber that writes plain lines (no colours, no timestamps, no
    /// target) into this buffer. Install it for a test with
    /// `tracing::subscriber::set_default`.
    pub fn subscriber(&self) -> impl tracing::Subscriber + Send + Sync + 'static {
        tracing_subscriber::fmt()
            .with_ansi(false)
            .without_time()
            .with_target(false)
            .with_writer(self.clone())
            .finish()
    }
}

impl io::Write for LogBuffer {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for LogBuffer {
    type Writer = LogBuffer;

    fn make_writer(&'a self) -> LogBuffer {
        self.clone()
    }
}

// ---------------------------------------------------------------- the error

/// What an [`ApiError`] leaves on its response for the middleware to find.
#[derive(Clone, Debug)]
pub struct ErrorInfo {
    pub code: String,
    pub message: String,
}

/// A failure a handler can return. On its own it renders the plain envelope
/// `{"error":{"code":...,"message":...}}`; the request-id middleware rewrites
/// that body to add the id, so no handler ever has to know the id.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    pub fn not_found(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::NOT_FOUND,
            code: "not_found",
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code: "internal_error",
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let info = ErrorInfo {
            code: self.code.to_string(),
            message: self.message.clone(),
        };
        let body = json!({"error": {"code": self.code, "message": self.message}});
        let mut response = (self.status, Json(body)).into_response();
        response.extensions_mut().insert(info);
        response
    }
}

// ------------------------------------------------------------ the app (given)

async fn get_anime(Path(id): Path<u32>) -> Result<Json<Value>, ApiError> {
    tracing::info!(id, "looking up anime");
    match id {
        1 => Ok(Json(json!({"id": 1, "title": "Cowboy Bebop"}))),
        _ => Err(ApiError::not_found(format!("anime {id} not found"))),
    }
}

async fn boom() -> ApiError {
    tracing::error!("the database connection dropped");
    ApiError::internal("something went wrong on our side")
}

async fn whoami(Extension(id): Extension<RequestId>) -> String {
    id.0
}

/// The routes, with no middleware. `GET /anime/{id}` (only id 1 exists),
/// `GET /boom` (always a 500) and `GET /whoami` (echoes the request id).
pub fn routes() -> Router {
    Router::new()
        .route("/anime/{id}", get(get_anime))
        .route("/boom", get(boom))
        .route("/whoami", get(whoami))
}

/// The routes wrapped in [`request_id_middleware`].
pub fn app() -> Router {
    routes().layer(from_fn(request_id_middleware))
}

// ------------------------------------------------------------- Implement

/// Is `candidate` an id this service is willing to accept from a caller?
///
/// Yes when it is 1 to [`MAX_REQUEST_ID_LEN`] (64) bytes long and every
/// character is an ASCII letter, an ASCII digit, `-`, `_` or `.`. Anything else
/// is no: the empty string, 65 bytes, a space, a quote, `/`, a non-ASCII
/// letter. (A UUID, `67e55044-10b1-426f-9247-bb680e5fe0c8`, passes.)
pub fn is_valid_request_id(candidate: &str) -> bool {
    todo!("say whether the candidate is an id this service accepts")
}

/// Pick the id for one request.
///
/// `incoming` is the caller's `x-request-id` header as text, or `None` when
/// the header was absent or was not text. If it is `Some` and
/// [`is_valid_request_id`] accepts it, return it unchanged. In every other
/// case return `generate()`, and call `generate` only in those cases. An
/// invalid id is replaced, never trimmed or repaired.
pub fn resolve_request_id(incoming: Option<&str>, generate: impl FnOnce() -> String) -> String {
    todo!(
        "pick the id for this request: the caller's if it is acceptable, otherwise a generated one"
    )
}

/// The JSON error body, with the request id in it. Exactly this shape:
///
/// ```json
/// {"error": {"code": "<code>", "message": "<message>", "request_id": "<request_id>"}}
/// ```
///
/// All three values are JSON strings, taken as given.
pub fn error_body(request_id: &str, code: &str, message: &str) -> Value {
    todo!("build the JSON error body with the code, the message and the request id")
}

// ----------------------------------------------------------------- Build

/// Middleware that gives every request an id and carries it everywhere.
///
/// 1. Read the `x-request-id` request header as text (a header that is not
///    valid text counts as absent) and choose the id with
///    [`resolve_request_id`], generating with [`new_request_id`].
/// 2. Insert `RequestId(id)` into the request's extensions.
/// 3. Open a tracing span named `request` at INFO level, with three fields:
///    `request_id` (the id, recorded with `Display`, so it prints without
///    quotes), `method` (`GET`) and `path` (the URI path only, no query).
///    Everything after this point, including the inner app, runs inside it.
/// 4. After the inner app answers, log one INFO event whose message is
///    `finished` and whose field `status` is the numeric status code.
/// 5. Set the `x-request-id` header on the response to the id, replacing any
///    value already there.
/// 6. If the response carries an [`ErrorInfo`] extension, replace its body with
///    [`error_body`] (using the id and the info's code and message), keep the
///    status, make the content type `application/json`, drop any
///    `content-length`, and first log one event whose message is
///    `request failed` and whose field `code` is the info's code: at WARN for a
///    4xx status, at ERROR for a 5xx.
///
/// Any other response is returned as the inner app made it, plus the header.
pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    todo!("give the request an id, run the rest of the app inside a span that carries it, and put the id on the response and in any error body")
}
