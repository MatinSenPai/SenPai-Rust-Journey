//! A real end-to-end test: binds the actual axum app to a real (ephemeral)
//! TCP port, connects real WebSocket clients to it via `tokio-tungstenite`,
//! and proves messages actually fan out over the network — not just
//! in-process broadcast channel semantics (see `src/lib.rs`'s own
//! `#[cfg(test)]` unit tests for those). No `#[ignore]` needed here: unlike
//! the Postgres/Redis lessons, this test needs no external infrastructure
//! at all — it stands up its own server on an OS-assigned port.

use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use p5_07_03_design_a_chat_system::{app, ChatMessage, ChatServer};
use tokio::net::TcpListener;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;

/// Boots the app on an OS-assigned free port and returns its `ws://` base
/// URL, e.g. `ws://127.0.0.1:54321`. The server runs on a spawned task for
/// the lifetime of the test process — fine for a test binary, since the
/// whole process exits when the test finishes.
async fn spawn_server() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let server = Arc::new(ChatServer::new());

    tokio::spawn(async move {
        axum::serve(listener, app(server)).await.unwrap();
    });

    format!("ws://{addr}")
}

async fn recv_chat_message(
    stream: &mut (impl StreamExt<Item = Result<TungsteniteMessage, tokio_tungstenite::tungstenite::Error>>
              + Unpin),
) -> ChatMessage {
    let msg = tokio::time::timeout(Duration::from_secs(2), stream.next())
        .await
        .expect("timed out waiting for a message")
        .expect("stream ended")
        .unwrap();
    match msg {
        TungsteniteMessage::Text(text) => serde_json::from_str(&text).unwrap(),
        other => panic!("expected a text message, got {other:?}"),
    }
}

#[tokio::test]
async fn a_message_from_one_client_reaches_another_in_the_same_room() {
    let base = spawn_server().await;

    let (mut alice, _) = connect_async(format!("{base}/ws/general?username=alice"))
        .await
        .unwrap();
    let (mut bob, _) = connect_async(format!("{base}/ws/general?username=bob"))
        .await
        .unwrap();

    // Give the server a moment to register both subscribers before anyone
    // sends — otherwise alice's message could broadcast before bob's
    // subscription exists yet, and bob would never see it (not a bug in
    // the broadcast channel, just an ordering issue in this test).
    tokio::time::sleep(Duration::from_millis(100)).await;

    alice
        .send(TungsteniteMessage::text("hello from alice"))
        .await
        .unwrap();

    let received = recv_chat_message(&mut bob).await;
    assert_eq!(received.username, "alice");
    assert_eq!(received.body, "hello from alice");
}

#[tokio::test]
async fn messages_do_not_leak_across_rooms() {
    let base = spawn_server().await;

    let (mut alice, _) = connect_async(format!("{base}/ws/general?username=alice"))
        .await
        .unwrap();
    let (mut carol, _) = connect_async(format!("{base}/ws/random?username=carol"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    alice
        .send(TungsteniteMessage::text("only for #general"))
        .await
        .unwrap();

    // carol is in #random — she should time out waiting, never receive
    // alice's #general message.
    let result = tokio::time::timeout(Duration::from_millis(500), carol.next()).await;
    assert!(
        result.is_err(),
        "carol should not receive a message meant for a different room"
    );
}

#[tokio::test]
async fn a_sender_also_receives_their_own_message_back() {
    // Broadcasting to everyone subscribed, including the sender's own
    // subscription, is a deliberate design choice (not every chat system
    // makes this choice — some explicitly exclude the sender). Documented
    // here as this lesson's actual behavior, discussed further in
    // `CHECKPOINT.md`.
    let base = spawn_server().await;

    let (mut alice, _) = connect_async(format!("{base}/ws/general?username=alice"))
        .await
        .unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    alice.send(TungsteniteMessage::text("echo?")).await.unwrap();

    let received = recv_chat_message(&mut alice).await;
    assert_eq!(received.body, "echo?");
}
