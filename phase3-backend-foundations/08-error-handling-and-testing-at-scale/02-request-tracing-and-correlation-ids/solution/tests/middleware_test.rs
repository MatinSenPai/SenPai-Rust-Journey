use axum::body::{to_bytes, Body};
use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::Response;
use p3_08_02_request_tracing_and_correlation_ids_solution::{app, LogBuffer};
use serde_json::Value;
use tower::ServiceExt;

/// Send one GET, optionally with an `x-request-id` header.
async fn get(path: &str, id: Option<&str>) -> Response {
    let mut builder = Request::builder().uri(path);
    if let Some(id) = id {
        builder = builder.header("x-request-id", id);
    }
    app()
        .oneshot(builder.body(Body::empty()).unwrap())
        .await
        .unwrap()
}

async fn body_text(response: Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn header(response: &Response, name: &str) -> String {
    response.headers()[name].to_str().unwrap().to_string()
}

// Each test installs a subscriber for its own thread. `#[tokio::test]` runs the
// test on one thread, so the guard covers the whole request.

#[tokio::test]
async fn a_valid_incoming_id_is_kept_and_echoed() {
    let response = get("/whoami", Some("abc-123")).await;
    assert_eq!(header(&response, "x-request-id"), "abc-123");
    assert_eq!(body_text(response).await, "abc-123");
}

#[tokio::test]
async fn a_missing_id_is_generated_and_echoed() {
    let response = get("/whoami", None).await;
    let echoed = header(&response, "x-request-id");
    assert_eq!(echoed.len(), 36);
    assert_eq!(body_text(response).await, echoed);
}

#[tokio::test]
async fn a_malformed_id_is_replaced() {
    let response = get("/whoami", Some("not valid!")).await;
    let echoed = header(&response, "x-request-id");
    assert_ne!(echoed, "not valid!");
    assert_eq!(echoed.len(), 36);
}

#[tokio::test]
async fn an_oversized_id_is_replaced() {
    let huge = "a".repeat(5000);
    let response = get("/whoami", Some(&huge)).await;
    assert_eq!(header(&response, "x-request-id").len(), 36);
}

#[tokio::test]
async fn handler_log_lines_carry_the_id_through_the_span() {
    let buffer = LogBuffer::new();
    let _guard = tracing::subscriber::set_default(buffer.subscriber());

    get("/anime/1", Some("abc-123")).await;

    let log = buffer.contents();
    let handler_line = log
        .lines()
        .find(|line| line.contains("looking up anime"))
        .expect("the handler's line is in the log");
    assert!(
        handler_line.contains("request_id=abc-123"),
        "{handler_line}"
    );
    assert!(handler_line.contains("method=GET"), "{handler_line}");
    assert!(handler_line.contains("path=/anime/1"), "{handler_line}");
    let finished = log.lines().find(|l| l.contains("finished")).unwrap();
    assert!(finished.contains("request_id=abc-123"), "{finished}");
    assert!(finished.contains("status=200"), "{finished}");
}

#[tokio::test]
async fn the_path_in_the_span_has_no_query_string() {
    let buffer = LogBuffer::new();
    let _guard = tracing::subscriber::set_default(buffer.subscriber());

    get("/anime/1?secret=hunter2", Some("q-1")).await;

    let log = buffer.contents();
    assert!(log.contains("path=/anime/1"));
    assert!(!log.contains("hunter2"));
}

#[tokio::test]
async fn a_generated_id_is_the_one_in_the_log() {
    let buffer = LogBuffer::new();
    let _guard = tracing::subscriber::set_default(buffer.subscriber());

    let response = get("/anime/1", None).await;

    let echoed = header(&response, "x-request-id");
    assert!(buffer.contents().contains(&format!("request_id={echoed}")));
}

#[tokio::test]
async fn a_hostile_id_never_reaches_the_log() {
    let buffer = LogBuffer::new();
    let _guard = tracing::subscriber::set_default(buffer.subscriber());

    get("/anime/1", Some("x status=200 injected=true")).await;

    assert!(!buffer.contents().contains("injected"));
}

#[tokio::test]
async fn an_error_body_carries_the_id_and_keeps_the_status() {
    let response = get("/anime/99", Some("abc-123")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(header(&response, "x-request-id"), "abc-123");
    assert_eq!(header(&response, "content-type"), "application/json");

    let body: Value = serde_json::from_str(&body_text(response).await).unwrap();
    assert_eq!(body["error"]["code"], "not_found");
    assert_eq!(body["error"]["message"], "anime 99 not found");
    assert_eq!(body["error"]["request_id"], "abc-123");
}

#[tokio::test]
async fn failures_are_logged_with_their_code_at_the_right_level() {
    let buffer = LogBuffer::new();
    let _guard = tracing::subscriber::set_default(buffer.subscriber());

    get("/anime/99", Some("a-404")).await;
    get("/boom", Some("a-500")).await;

    let log = buffer.contents();
    let not_found = log.lines().find(|l| l.contains("code=not_found")).unwrap();
    assert!(not_found.starts_with(" WARN"), "{not_found}");
    assert!(not_found.contains("request_id=a-404"), "{not_found}");
    let internal = log
        .lines()
        .find(|l| l.contains("code=internal_error"))
        .unwrap();
    assert!(internal.starts_with("ERROR"), "{internal}");
    assert!(internal.contains("request_id=a-500"), "{internal}");
}

#[tokio::test]
async fn a_successful_response_body_is_untouched() {
    let response = get("/anime/1", Some("abc-123")).await;
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value = serde_json::from_str(&body_text(response).await).unwrap();
    assert_eq!(body["title"], "Cowboy Bebop");
    assert!(body.get("error").is_none());
}

#[tokio::test]
async fn a_route_that_does_not_exist_still_gets_an_id() {
    let response = get("/nope", Some("abc-123")).await;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(header(&response, "x-request-id"), "abc-123");
}
