# 07.4 — Design a notification system

Module 1's `02-realtime-communication` lesson laid SSE next to WebSocket
and named the deciding factor: SSE is **server-to-client only**. A
notification feed is the textbook case for that shape — the client never
needs to push anything back over the same connection it's receiving
notifications on, unlike the previous lesson's chat system, which
genuinely needed WebSocket's bidirectionality.

## Requirements

- `POST /notifications {user_id, message}` creates a notification for a
  user.
- A client currently connected via SSE sees it **immediately**.
- A client that connects *later* (or was never connected — polling
  instead) can still fetch it via `GET /notifications/{user_id}`.
- `POST /notifications/{id}/read` marks one as read; unread listing
  excludes it afterward.

## Why persistence AND a broadcast channel, not just one or the other

An in-memory-only design (just a `broadcast` channel, no database) would
lose every notification created while a user isn't connected — exactly
the failure mode a notification system exists to prevent. A
database-only design (no broadcast channel, clients just poll
`GET /notifications/{user_id}` on an interval) works, but "immediately"
becomes "up to one polling interval late," and every connected client
hits the database on every poll whether anything changed or not. This
lesson does both: Postgres is the source of truth (durable, survives a
process restart, is what `list_unread` reads from), and an in-memory
`broadcast::Sender` per user is a **live notification path layered on
top** — purely an optimization for currently-connected clients, never the
only way a notification reaches someone.

## The "poll or push" decision is the client's, not the server's

```rust
pub fn app(store: Arc<NotificationStore>) -> Router {
    Router::new()
        .route("/notifications/{user_id}", get(list_unread))          // poll
        .route("/notifications/{user_id}/stream", get(notification_stream)) // push
        ...
}
```

Both routes read from the exact same underlying data (`list_unread`
queries Postgres directly; the SSE stream is fed by `create`'s broadcast
call, which only ever runs *after* the row is already committed to
Postgres). A mobile client on an unreliable connection might poll every
30 seconds; a web client with an open tab might hold an SSE connection
open indefinitely. Neither client needs the server to know which
strategy it's using — the server just exposes both, and stays correct
either way because both paths ultimately agree with the same table.

## Your task

Open `src/lib.rs`. Implement `NotificationStore::create`, `list_unread`,
and `mark_read`.

## Checkpoint

```sh
DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
  cargo test -p p5-07-04-design-a-notification-system -- --ignored
```

Then `CHECKPOINT.md`, then `solution/SOLUTION.md`.
