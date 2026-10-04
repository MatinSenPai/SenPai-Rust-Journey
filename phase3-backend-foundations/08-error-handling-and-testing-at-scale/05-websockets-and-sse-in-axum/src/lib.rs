//! Exercises for 3.8.5 — WebSockets and SSE in `axum`.
//!
//! The app has three routes, all sharing one [`Hub`]:
//!
//! - `GET /events`  — Server-Sent Events: every message published to the hub
//! - `POST /publish` — put a message into the hub
//! - `GET /ws`      — a WebSocket that answers text commands
//!
//! The WebSocket side (`ws_handler`, `handle_socket`) is finished and given,
//! the same way `LogLayer` was in 3.2.4. You write the pieces around it.

#[allow(unused_imports)]
use std::convert::Infallible;
use std::time::Duration;

use axum::body::Bytes;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
#[allow(unused_imports)]
use axum::http::StatusCode;
#[allow(unused_imports)]
use axum::response::sse::{Event, KeepAlive};
#[allow(unused_imports)]
use axum::response::IntoResponse;
use axum::response::Response;
#[allow(unused_imports)]
use axum::response::Sse;
use axum::routing::{get, post};
use axum::Router;
use tokio::sync::broadcast;
use tokio::time::{interval_at, Instant};
// Imports you will probably want for the exercises below.
#[allow(unused_imports)]
use tokio_stream::wrappers::errors::BroadcastStreamRecvError;
#[allow(unused_imports)]
use tokio_stream::wrappers::BroadcastStream;
#[allow(unused_imports)]
use tokio_stream::StreamExt;

/// How often the server pings an idle WebSocket (see `handle_socket`).
pub const HEARTBEAT: Duration = Duration::from_secs(30);

// ------------------------------------------------------------ Implement: SSE

/// Renders one Server-Sent Event the way it goes over the wire.
///
/// The result is, in this order:
///
/// 1. `event: <name>\n` — only if `event` is `Some`;
/// 2. `id: <n>\n` — only if `id` is `Some`, with `n` in plain decimal;
/// 3. one `data: <line>\n` for every line of `data`, where lines are the
///    pieces of `data.split('\n')`. An empty `data` is one empty line, so it
///    gives `data: \n` (the word, a colon, one space, nothing);
/// 4. one final `\n`, the blank line that ends the event.
///
/// Examples: `sse_frame(None, None, "hi")` is `"data: hi\n\n"`.
/// `sse_frame(Some("chat"), Some(7), "a\nb")` is
/// `"event: chat\nid: 7\ndata: a\ndata: b\n\n"`.
pub fn sse_frame(event: Option<&str>, id: Option<u64>, data: &str) -> String {
    todo!("render the event in the SSE wire format described above")
}

// ----------------------------------------------------- Implement: WS commands

/// The text a WebSocket client gets back for a text message it sent.
///
/// - exactly `ping` gives `pong`;
/// - text that starts with `echo ` (the word and one space) gives everything
///   after that prefix, unchanged (`echo hi there` gives `hi there`);
/// - anything else, including the bare word `echo` and the empty string,
///   gives `unknown: ` followed by the original text.
pub fn ws_reply(text: &str) -> String {
    todo!("answer the three kinds of message described above")
}

// -------------------------------------------------------------- Build: the hub

/// A broadcast room. Every [`Hub::subscribe`] gets its own copy of every
/// message published after it subscribed. Cloning a `Hub` shares the room.
#[derive(Clone)]
pub struct Hub {
    tx: broadcast::Sender<String>,
}

impl Hub {
    /// A room that remembers at most `capacity` unread messages per
    /// subscriber; a subscriber that falls further behind loses the oldest.
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Hub { tx }
    }

    /// A new subscriber. It sees only messages published after this call.
    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.tx.subscribe()
    }

    /// Sends `msg` to every current subscriber and returns how many
    /// subscribers it was delivered to. With nobody listening it returns `0`
    /// and does not fail.
    pub fn publish(&self, msg: &str) -> usize {
        todo!("deliver the message to all subscribers and report how many there were")
    }
}

/// `GET /events`: a Server-Sent Events stream of everything published.
///
/// - The response is `200` with `Content-Type: text/event-stream`, built with
///   [`Sse`] (so it has no `Content-Length`).
/// - The handler subscribes to the hub **before it returns**, not later inside
///   the stream, so a message published right after the client sees the
///   response headers is never missed.
/// - Each published message `m` becomes one event named `chat` whose data is
///   `m`.
/// - A subscriber that fell behind and lost `n` messages gets one event named
///   `lagged` whose data is `n` in decimal, and the stream carries on.
/// - A keep-alive comment is sent every 15 seconds.
async fn events(State(hub): State<Hub>) -> Response {
    todo!("stream the hub's messages to the client as server-sent events")
}

/// `POST /publish`: the request body (UTF-8 text) is published to the hub.
///
/// Answers `202 Accepted` with a body holding, in decimal, how many
/// subscribers received it (`0` is fine).
async fn publish(State(hub): State<Hub>, body: String) -> (StatusCode, String) {
    todo!("publish the body and answer 202 with the delivery count")
}

// ------------------------------------------------------------ Given: the app

/// The whole app: `/events`, `/publish` and `/ws`.
pub fn app(hub: Hub) -> Router {
    Router::new()
        .route("/events", get(events))
        .route("/publish", post(publish))
        .route("/ws", get(ws_handler))
        .with_state(hub)
}

/// Upgrades the request to a WebSocket and hands the socket to `handle_socket`.
async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

/// Answers each text message with `ws_reply`, pings every [`HEARTBEAT`], and
/// stops when the client closes or the connection breaks.
async fn handle_socket(mut socket: WebSocket) {
    let mut beat = interval_at(Instant::now() + HEARTBEAT, HEARTBEAT);
    loop {
        tokio::select! {
            incoming = socket.recv() => match incoming {
                Some(Ok(Message::Text(text))) => {
                    let reply = ws_reply(text.as_str());
                    if socket.send(Message::text(reply)).await.is_err() {
                        break;
                    }
                }
                // A Close frame is not a reason to stop here: the library has
                // already queued the reply, and the next `recv` flushes it and
                // then returns `None`.
                Some(Err(_)) | None => break,
                Some(Ok(_)) => {}
            },
            _ = beat.tick() => {
                if socket.send(Message::Ping(Bytes::new())).await.is_err() {
                    break;
                }
            }
        }
    }
}
