//! Design a notification system — combines module 2's persistence lessons
//! (a notification must survive even if nobody's connected when it's
//! created) with module 1's real-time-communication lesson's SSE option
//! (server-to-client only, exactly what a notification feed needs — unlike
//! the chat lesson, a client never needs to push *to* this service over
//! the same connection it's receiving on). See `README.md` for the full
//! requirements walkthrough before touching any `todo!()` here.

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt as _;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Notification {
    pub id: i64,
    pub user_id: String,
    pub message: String,
    pub read: bool,
}

#[derive(Debug, Deserialize)]
pub struct CreateNotification {
    pub user_id: String,
    pub message: String,
}

#[derive(Debug)]
pub enum NotificationError {
    NotFound,
    Storage(String),
}

impl IntoResponse for NotificationError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            NotificationError::NotFound => {
                (StatusCode::NOT_FOUND, "notification not found".to_string())
            }
            NotificationError::Storage(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

/// Postgres for durability (a notification created while you're offline
/// must still be there when you next poll or reconnect) plus one
/// in-memory `broadcast` channel per user for pushing live updates to
/// whoever's currently connected via SSE — the same "one channel per
/// key, created lazily" shape as the chat lesson's `ChatServer`, just
/// keyed by user instead of by room.
pub struct NotificationStore {
    pool: PgPool,
    channels: Mutex<HashMap<String, broadcast::Sender<Notification>>>,
}

impl NotificationStore {
    pub fn new(pool: PgPool) -> Self {
        NotificationStore {
            pool,
            channels: Mutex::new(HashMap::new()),
        }
    }

    fn channel_for(&self, user_id: &str) -> broadcast::Sender<Notification> {
        let mut channels = self.channels.lock().unwrap();
        if let Some(sender) = channels.get(user_id) {
            return sender.clone();
        }
        let (sender, _receiver) = broadcast::channel(32);
        channels.insert(user_id.to_string(), sender.clone());
        sender
    }

    /// Live-subscribes to `user_id`'s notification stream — used by the
    /// SSE handler. Given, fully implemented: the exercise is `create`,
    /// `list_unread`, and `mark_read` below.
    pub fn subscribe(&self, user_id: &str) -> broadcast::Receiver<Notification> {
        self.channel_for(user_id).subscribe()
    }

    /// Persists a notification, then broadcasts it to any currently
    /// connected SSE subscribers for `user_id`. A `broadcast::send` error
    /// (no active subscribers right now) is expected and fine — the
    /// notification is already durably stored either way, so whoever
    /// connects later will see it via `list_unread`.
    pub async fn create(
        &self,
        input: CreateNotification,
    ) -> Result<Notification, NotificationError> {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO p5_07_04_notifications (user_id, message, read) \
             VALUES ($1, $2, false) RETURNING id",
        )
        .bind(&input.user_id)
        .bind(&input.message)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| NotificationError::Storage(e.to_string()))?;

        let notification = Notification {
            id,
            user_id: input.user_id,
            message: input.message,
            read: false,
        };

        let _ = self
            .channel_for(&notification.user_id)
            .send(notification.clone());

        Ok(notification)
    }

    pub async fn list_unread(&self, user_id: &str) -> Result<Vec<Notification>, NotificationError> {
        sqlx::query_as(
            "SELECT id, user_id, message, read FROM p5_07_04_notifications \
             WHERE user_id = $1 AND read = false ORDER BY id",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| NotificationError::Storage(e.to_string()))
    }

    pub async fn mark_read(&self, id: i64) -> Result<(), NotificationError> {
        let result = sqlx::query("UPDATE p5_07_04_notifications SET read = true WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| NotificationError::Storage(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(NotificationError::NotFound);
        }
        Ok(())
    }
}

pub async fn create_notification(
    State(store): State<Arc<NotificationStore>>,
    Json(input): Json<CreateNotification>,
) -> Result<(StatusCode, Json<Notification>), NotificationError> {
    let notification = store.create(input).await?;
    Ok((StatusCode::CREATED, Json(notification)))
}

pub async fn list_unread(
    State(store): State<Arc<NotificationStore>>,
    Path(user_id): Path<String>,
) -> Result<Json<Vec<Notification>>, NotificationError> {
    store.list_unread(&user_id).await.map(Json)
}

pub async fn mark_read(
    State(store): State<Arc<NotificationStore>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, NotificationError> {
    store.mark_read(id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Given, fully implemented: the "poll or push" fork is entirely a client
/// decision, not a server one — `list_unread` above and this SSE endpoint
/// both just reflect the exact same underlying data.
pub async fn notification_stream(
    State(store): State<Arc<NotificationStore>>,
    Path(user_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let receiver = store.subscribe(&user_id);
    let stream = BroadcastStream::new(receiver).filter_map(|result| {
        result
            .ok()
            .and_then(|n| serde_json::to_string(&n).ok())
            .map(|json| Ok(Event::default().data(json)))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub fn app(store: Arc<NotificationStore>) -> Router {
    Router::new()
        .route("/notifications", post(create_notification))
        .route("/notifications/{user_id}", get(list_unread))
        .route("/notifications/{user_id}/stream", get(notification_stream))
        .route("/notifications/{id}/read", post(mark_read))
        .with_state(store)
}
