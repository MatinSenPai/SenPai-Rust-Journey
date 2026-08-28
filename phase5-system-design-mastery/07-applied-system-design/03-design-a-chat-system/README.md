# 07.3 — Design a chat system

Module 1's `02-realtime-communication` lesson laid out short polling, long
polling, SSE, and WebSocket as the real-time transport options and where
each wins. Chat is the textbook case for WebSocket specifically: it's
**bidirectional** — either side can send at any moment — which polling and
SSE (server-to-client only) simply can't do without bolting a second,
separate HTTP request path on for the client-to-server direction.

## Requirements

- Multiple named rooms; joining `/ws/{room}?username=alice` puts you in
  that room only.
- A message sent by any participant in a room is broadcast to every other
  participant currently in that room.
- Rooms are created on first use — no separate "create a room" endpoint.

## The core primitive: one `broadcast::Sender` per room

```rust
pub struct ChatServer {
    rooms: Mutex<HashMap<String, broadcast::Sender<ChatMessage>>>,
}
```

`tokio::sync::broadcast` is a multi-producer, multi-consumer channel where
**every** subscribed `Receiver` gets a clone of every value sent — exactly
the fan-out semantics a chat room needs. `ChatServer` is a lazy registry:
the first client to join `"general"` causes `room_sender("general")` to
create a fresh channel; every client after that gets a `.clone()` of the
same `Sender` (cheap — it's an `Arc` internally), so they're all talking
to the same underlying channel without ever needing to discover each
other directly.

## Reading `handle_socket` (given, fully implemented)

```rust
let (mut ws_sink, mut ws_stream) = socket.split();
let mut send_task = tokio::spawn(/* broadcast receiver -> ws_sink */);
let mut recv_task = tokio::spawn(/* ws_stream -> broadcast sender.send(...) */);
tokio::select! { ... }
```

Every connected client runs **two independent tasks** concurrently: one
reading from this client's WebSocket and re-broadcasting to the room
(`recv_task`), one reading from the room's broadcast channel and writing
out to this client's WebSocket (`send_task`). `tokio::select!` waits for
whichever finishes first (typically: the client disconnects, one side's
task errors) and aborts the other — without this, a disconnected client's
`send_task` would loop forever trying to write to a dead socket.

## Testing a WebSocket for real

Every other axum lesson in this repo tests with `tower::ServiceExt::oneshot`
— a fake in-process request, no real network. That doesn't work for a
WebSocket upgrade (`oneshot` doesn't speak the upgrade handshake). This
lesson's `tests/websocket_test.rs` instead binds the real app to a real,
OS-assigned ephemeral port (`TcpListener::bind("127.0.0.1:0")`) and
connects genuine WebSocket clients via `tokio-tungstenite` — an actual,
if entirely local, network round trip. No `#[ignore]` needed: unlike the
Postgres/Redis lessons, this test needs no separately-running
infrastructure, since the test itself boots the only server it talks to.

## Your task

Open `src/lib.rs`. Implement `ChatServer::room_sender`.

## Next

`cargo test -p p5-07-03-design-a-chat-system` (runs everything — the
in-memory unit tests *and* the real WebSocket integration tests, no live
infrastructure required). Then `solution/SOLUTION.md` — but only after a real attempt.
