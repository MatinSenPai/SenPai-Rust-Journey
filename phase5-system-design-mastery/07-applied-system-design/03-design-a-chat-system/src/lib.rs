//! Design a chat system — the applied payoff of Module 1's
//! `02-realtime-communication` lesson: WebSocket is the right transport
//! here specifically because chat is genuinely bidirectional (anyone can
//! send at any time), unlike a notification feed where the server is
//! usually the only side pushing. See `README.md` for the requirements
//! walkthrough before touching any `todo!()` here.

use std::collections::HashMap;
use std::sync::Mutex;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub username: String,
    pub body: String,
}

/// One `broadcast::Sender` per room — every participant in a room holds a
/// `Receiver` subscribed to that room's `Sender`; a message sent by any one
/// participant fans out to every other subscriber automatically. Rooms are
/// created lazily on first use and never explicitly torn down (a genuine,
/// acceptable simplification for this lesson — see `README.md`'s
/// checkpoint about what a production version would need to add).
#[derive(Default)]
pub struct ChatServer {
    rooms: Mutex<HashMap<String, broadcast::Sender<ChatMessage>>>,
}

impl ChatServer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the `broadcast::Sender` for `room`, creating a fresh one
    /// (capacity 100 — how many unread messages a slow subscriber can fall
    /// behind by before it starts missing messages) if this is the first
    /// time anyone has joined `room`.
    pub fn room_sender(&self, room: &str) -> broadcast::Sender<ChatMessage> {
        todo!(
            "lock self.rooms; if room already has an entry, return a .clone() of its Sender \
             (broadcast::Sender is cheaply cloneable, same underlying channel); otherwise \
             create one with broadcast::channel(100), .insert(room.to_string(), sender.clone()) \
             it into the map, and return the sender: {{}}"
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct JoinQuery {
    pub username: String,
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(room): Path<String>,
    Query(query): Query<JoinQuery>,
    State(server): State<Arc<ChatServer>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, room, query.username, server))
}

/// Given, fully implemented: splits the socket into a sink/stream pair,
/// subscribes to the room's broadcast channel, and spawns one task
/// forwarding broadcast messages out to this client while the current
/// task reads incoming messages from this client. This wiring is the
/// "given" part — `ChatServer::room_sender` above is the exercise.
async fn handle_socket(socket: WebSocket, room: String, username: String, server: Arc<ChatServer>) {
    let sender = server.room_sender(&room);
    let mut receiver = sender.subscribe();
    let (mut ws_sink, mut ws_stream) = socket.split();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = receiver.recv().await {
            let text = serde_json::to_string(&msg).unwrap();
            if ws_sink.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn({
        let sender = sender.clone();
        async move {
            while let Some(Ok(Message::Text(text))) = ws_stream.next().await {
                let chat_message = ChatMessage {
                    username: username.clone(),
                    body: text.to_string(),
                };
                // A send error just means there are currently no
                // subscribers (everyone else already left the room) —
                // not a reason to stop reading from this client.
                let _ = sender.send(chat_message);
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

pub fn app(server: Arc<ChatServer>) -> Router {
    Router::new()
        .route("/ws/{room}", get(ws_handler))
        .with_state(server)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_room_always_returns_the_same_sender() {
        let server = ChatServer::new();
        let a = server.room_sender("general");
        let b = server.room_sender("general");

        // Sending on one and receiving on a subscriber of the other proves
        // they're the same underlying channel, not just two independently
        // constructed channels.
        let mut rx = b.subscribe();
        a.send(ChatMessage {
            username: "alice".to_string(),
            body: "hi".to_string(),
        })
        .unwrap();

        let received = rx.try_recv().unwrap();
        assert_eq!(received.body, "hi");
    }

    #[test]
    fn different_rooms_get_different_senders() {
        let server = ChatServer::new();
        let general = server.room_sender("general");
        let random = server.room_sender("random");

        // Every `broadcast::Sender::send` errors if it currently has zero
        // subscribers — subscribe to both rooms first so the send below
        // has somewhere to go.
        let mut general_rx = general.subscribe();
        let mut random_rx = random.subscribe();
        general
            .send(ChatMessage {
                username: "alice".to_string(),
                body: "only for #general".to_string(),
            })
            .unwrap();

        assert_eq!(general_rx.try_recv().unwrap().body, "only for #general");

        // #random's subscriber should see nothing — the two rooms' channels
        // are entirely independent.
        assert!(random_rx.try_recv().is_err());
    }
}
