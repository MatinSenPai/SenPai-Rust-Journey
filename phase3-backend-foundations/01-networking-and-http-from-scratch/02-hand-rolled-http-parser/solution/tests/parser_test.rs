use p3_01_02_hand_rolled_http_parser_solution::{
    parse_request, HttpParseError, HttpResponse, Method,
};

fn raw(lines: &[&str]) -> Vec<u8> {
    format!("{}\r\n\r\n", lines.join("\r\n")).into_bytes()
}

#[test]
fn parses_a_simple_get_request() {
    let request = parse_request(&raw(&["GET / HTTP/1.1", "Host: localhost:7879"])).unwrap();

    assert_eq!(request.method, Method::Get);
    assert_eq!(request.path, "/");
    assert_eq!(request.query, None);
    assert_eq!(request.version, "HTTP/1.1");
    assert_eq!(request.header("Host"), Some("localhost:7879"));
}

#[test]
fn splits_path_and_query_string() {
    let request = parse_request(&raw(&["GET /anime?status=watching HTTP/1.1", "Host: x"])).unwrap();

    assert_eq!(request.path, "/anime");
    assert_eq!(request.query.as_deref(), Some("status=watching"));
}

#[test]
fn header_lookup_is_case_insensitive() {
    let request = parse_request(&raw(&["GET / HTTP/1.1", "Content-Type: text/plain"])).unwrap();

    assert_eq!(request.header("content-type"), Some("text/plain"));
    assert_eq!(request.header("CONTENT-TYPE"), Some("text/plain"));
    assert_eq!(request.header("Content-Type"), Some("text/plain"));
}

#[test]
fn header_lookup_returns_none_for_missing_header() {
    let request = parse_request(&raw(&["GET / HTTP/1.1", "Host: x"])).unwrap();
    assert_eq!(request.header("X-Not-Present"), None);
}

#[test]
fn parses_a_request_with_multiple_headers() {
    let request = parse_request(&raw(&[
        "GET /anime HTTP/1.1",
        "Host: localhost",
        "User-Agent: curl/8.4.0",
        "Accept: */*",
    ]))
    .unwrap();

    assert_eq!(request.headers.len(), 3);
    assert_eq!(request.header("User-Agent"), Some("curl/8.4.0"));
}

#[test]
fn rejects_invalid_utf8() {
    let invalid = vec![0xff, 0xfe, 0xfd];
    assert_eq!(parse_request(&invalid), Err(HttpParseError::InvalidUtf8));
}

#[test]
fn rejects_empty_request() {
    assert_eq!(parse_request(b""), Err(HttpParseError::EmptyRequest));
}

#[test]
fn rejects_malformed_request_line() {
    let result = parse_request(&raw(&["GET /"]));
    assert!(matches!(
        result,
        Err(HttpParseError::MalformedRequestLine(_))
    ));
}

#[test]
fn rejects_malformed_header_line() {
    let result = parse_request(&raw(&["GET / HTTP/1.1", "not-a-valid-header-line"]));
    assert!(matches!(
        result,
        Err(HttpParseError::MalformedHeaderLine(_))
    ));
}

#[test]
fn serializes_a_response_with_correct_content_length() {
    let response = HttpResponse::ok("hi");
    let bytes = response.to_bytes();
    let text = String::from_utf8(bytes).unwrap();

    assert!(text.starts_with("HTTP/1.1 200 OK\r\n"));
    assert!(text.contains("Content-Length: 2\r\n"));
    assert!(text.ends_with("\r\n\r\nhi"));
}

#[test]
fn not_found_response_uses_404() {
    let response = HttpResponse::not_found();
    let text = String::from_utf8(response.to_bytes()).unwrap();
    assert!(text.starts_with("HTTP/1.1 404 Not Found\r\n"));
}
