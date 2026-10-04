use p3_08_05_websockets_and_sse_in_axum_solution::ws_reply;

#[test]
fn ping_gets_pong() {
    assert_eq!(ws_reply("ping"), "pong");
}

#[test]
fn echo_returns_the_rest_unchanged() {
    assert_eq!(ws_reply("echo hi there"), "hi there");
    assert_eq!(ws_reply("echo  two spaces"), " two spaces");
}

#[test]
fn everything_else_is_unknown() {
    assert_eq!(ws_reply("echo"), "unknown: echo");
    assert_eq!(ws_reply(""), "unknown: ");
    assert_eq!(ws_reply("PING"), "unknown: PING");
    assert_eq!(ws_reply("ping "), "unknown: ping ");
}
