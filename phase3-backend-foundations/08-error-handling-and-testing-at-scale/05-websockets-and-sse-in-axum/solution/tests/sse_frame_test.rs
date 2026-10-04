mod common;
use common::*;
use p3_08_05_websockets_and_sse_in_axum_solution::{app, sse_frame, Hub};

#[test]
fn data_only() {
    assert_eq!(sse_frame(None, None, "hi"), "data: hi\n\n");
}

#[test]
fn event_id_and_multiline_data() {
    assert_eq!(
        sse_frame(Some("chat"), Some(7), "a\nb"),
        "event: chat\nid: 7\ndata: a\ndata: b\n\n"
    );
}

#[test]
fn empty_data_is_one_empty_line() {
    assert_eq!(sse_frame(Some("ping"), None, ""), "event: ping\ndata: \n\n");
}

#[test]
fn trailing_newline_gives_an_empty_last_data_line() {
    assert_eq!(sse_frame(None, None, "a\n"), "data: a\ndata: \n\n");
}

/// The hand-made frame is byte-for-byte what `axum`'s `Event` puts on the wire
/// (this one needs the Build exercise's `/events` route too).
#[tokio::test]
async fn matches_what_axum_sends() {
    let hub = Hub::new(8);
    let (addr, server) = spawn(app(hub.clone())).await;
    let (mut reader, _) = sse_connect(addr, "/events").await;
    hub.publish("one\ntwo");
    let chunk = next_chunk(&mut reader).await;
    assert_eq!(chunk, sse_frame(Some("chat"), None, "one\ntwo"));
    server.abort();
}
