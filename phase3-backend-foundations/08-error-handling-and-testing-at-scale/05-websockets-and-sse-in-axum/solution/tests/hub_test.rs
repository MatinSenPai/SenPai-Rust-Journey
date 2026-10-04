mod common;
use common::*;
use p3_08_05_websockets_and_sse_in_axum_solution::{app, Hub};

#[test]
fn publish_counts_subscribers() {
    let hub = Hub::new(4);
    assert_eq!(hub.publish("nobody"), 0);
    let _a = hub.subscribe();
    let _b = hub.subscribe();
    assert_eq!(hub.publish("two"), 2);
}

#[tokio::test]
async fn events_is_an_endless_chunked_response() {
    let (addr, server) = spawn(app(Hub::new(8))).await;
    let (_reader, head) = sse_connect(addr, "/events").await;
    let lower = head.to_ascii_lowercase();
    assert!(head.starts_with("HTTP/1.1 200"), "{head}");
    assert!(lower.contains("content-type: text/event-stream"), "{head}");
    assert!(lower.contains("transfer-encoding: chunked"), "{head}");
    assert!(!lower.contains("content-length"), "{head}");
    server.abort();
}

#[tokio::test]
async fn publish_reaches_every_event_stream() {
    let (addr, server) = spawn(app(Hub::new(8))).await;
    let (mut a, _) = sse_connect(addr, "/events").await;
    let (mut b, _) = sse_connect(addr, "/events").await;

    let (status, body) = post(addr, "/publish", "konnichiwa").await;
    assert!(status.starts_with("HTTP/1.1 202"), "{status}");
    assert_eq!(body, "2");

    assert_eq!(
        next_chunk(&mut a).await,
        "event: chat\ndata: konnichiwa\n\n"
    );
    assert_eq!(
        next_chunk(&mut b).await,
        "event: chat\ndata: konnichiwa\n\n"
    );
    server.abort();
}

#[tokio::test]
async fn publish_with_no_listener_is_still_accepted() {
    let (addr, server) = spawn(app(Hub::new(8))).await;
    let (status, body) = post(addr, "/publish", "alone").await;
    assert!(status.starts_with("HTTP/1.1 202"), "{status}");
    assert_eq!(body, "0");
    server.abort();
}

/// A room of two and five quick messages. Whether the server task keeps up
/// is up to the scheduler, so this test does not fix how many `lagged` events
/// appear; it checks that every event on the wire is a well-formed `chat` or
/// `lagged` event and that the stream stays alive after a loss.
#[tokio::test]
async fn every_event_is_chat_or_lagged() {
    let hub = Hub::new(2);
    let (addr, server) = spawn(app(hub.clone())).await;
    let (mut reader, _) = sse_connect(addr, "/events").await;
    for i in 0..5 {
        hub.publish(&format!("m{i}"));
    }
    hub.publish("last");
    loop {
        let chunk = next_chunk(&mut reader).await;
        assert!(
            chunk.starts_with("event: chat\ndata: ") || chunk.starts_with("event: lagged\ndata: "),
            "{chunk:?}"
        );
        if chunk == "event: chat\ndata: last\n\n" {
            break;
        }
    }
    server.abort();
}
