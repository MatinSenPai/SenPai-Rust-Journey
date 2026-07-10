# Solution

```rust
pub async fn create(&self, input: CreateNotification) -> Result<Notification, NotificationError> {
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

    let _ = self.channel_for(&notification.user_id).send(notification.clone());

    Ok(notification)
}
```

`list_unread` and `mark_read` are single queries — see `src/lib.rs`, the
`todo!()` hints already spelled out the exact query text.

## Why `send`'s error is safely ignored

`broadcast::Sender::send` returns `Err(SendError(value))` in exactly one
situation: **zero receivers currently subscribed**. Since `create` doesn't
know or care whether the target user happens to have an SSE connection
open right now, this is an entirely expected, routine outcome, not a
failure — most notifications will probably be created while the
recipient isn't actively watching a live stream. The notification is
already safely committed to Postgres by the time `send` is even called
(the `INSERT ... RETURNING id` already succeeded), so a "nobody's
listening live" outcome costs nothing: `list_unread` will still return it
whenever the user next asks, whether that's 3 seconds or 3 days later.
Propagating this as a `NotificationError` would be actively wrong — it
would make `create` fail for the extremely common case of "the recipient
isn't currently connected," which isn't an error at all.

## Why Postgres first, broadcast second

If `create` broadcast to the channel *before* the `INSERT` committed, a
connected client could receive the live SSE push, then immediately
`GET /notifications/{user_id}` (say, to confirm or refresh) and get back
a list that doesn't yet include the notification they just saw pushed —
because the `INSERT` might not have finished, or could even still fail
afterward (a constraint violation, a connection drop) after the broadcast
already went out, leaving a client with a phantom notification that was
never actually persisted. Insert-then-broadcast guarantees every pushed
notification has already passed through the same durability guarantee
the polling endpoint relies on — a connected client's live view and a
freshly-polling client's view can never disagree about what's real.

## Where per-user isolation actually comes from

Exactly the same answer as the chat lesson's per-room isolation: nothing
inside `create` or `notification_stream` explicitly checks `user_id`
against anything at broadcast time. `channels: Mutex<HashMap<String,
broadcast::Sender<Notification>>>` gives each `user_id` key its own
entirely separate channel — `channel_for("alice")` and
`channel_for("carol")` are two unrelated `broadcast::Sender`s with no
shared subscribers, so a message sent on alice's channel structurally
cannot reach anything subscribed only to carol's.

## What happens to notifications created during a disconnected gap

They aren't lost — they're sitting in Postgres, `read = false`, exactly
as durable as any other row. But the *live push* for each of those 5
notifications is gone forever the moment it's sent: `broadcast::Sender::send`
only reaches receivers subscribed at that exact instant, and a
disconnected client has no subscribed `Receiver` to reach. This is the
real, honest gap this design has and doesn't try to hide: for
correctness, a well-behaved client **must** call `GET
/notifications/{user_id}` once immediately after establishing (or
re-establishing) an SSE connection, to catch up on anything created while
it wasn't listening — the SSE stream alone is only a "what's new from
this point forward" feed, never a complete history. The dual
poll-then-push pattern in this lesson's own client contract (fetch once,
then stream) is exactly how real systems close this gap.

## Adding a webhook-style push to a third party

Neither the SSE broadcast (only reaches an already-connected browser
client holding the response stream open) nor the polling endpoint (the
third party would have to actively ask, which is polling, not being
notified) fits "notify an external service the instant this happens."
The addition would be a third notification path inside `create`, after
the same `INSERT`: look up any webhook URL(s) registered for `user_id`
(a new table, or a config value for this lesson's scope), and fire an
HTTP `POST` to it with the notification payload — using `reqwest`, fired
via `tokio::spawn` so a slow or down webhook endpoint never blocks the
`create` call itself from returning to its original caller. This is
structurally the same "push, don't wait to be asked" idea as SSE, just
server-to-server instead of server-to-browser, and it introduces the
exact reliability problem Module 1's webhook discussion (and Module 4's
idempotency lesson) already named: an HTTP POST can fail or the receiver
can process it twice on retry, so a production version would need
retries with backoff and an idempotency key in the payload — the same
pattern this repo's `POST /jobs` idempotency gap already flagged as a
real, unaddressed TODO for TaskForge.
