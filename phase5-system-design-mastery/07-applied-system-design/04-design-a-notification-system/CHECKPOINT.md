# Checkpoint

1. `create` calls `self.channel_for(&notification.user_id).send(...)` and
   explicitly ignores the `Result` with `let _ = ...`. Under what
   condition does `broadcast::Sender::send` actually return an `Err`
   here, and why is silently ignoring it the correct behavior for this
   lesson's design, rather than propagating it as a `NotificationError`?
2. `create` inserts into Postgres *before* broadcasting to the channel,
   never the other way around. Walk through what could go wrong for a
   connected SSE client if that order were reversed — broadcast first,
   `INSERT` second.
3. `a_subscriber_for_a_different_user_does_not_see_it` proves per-user
   isolation. Trace exactly where that isolation comes from — is it a
   `WHERE user_id = ...` check anywhere in `create` or
   `notification_stream`, or a structural property of how `channels` is
   keyed?
4. This design has no way to tell a client "you've missed some
   notifications" the way `broadcast::Receiver::recv()`'s `Lagged(n)`
   error does in the chat lesson. Concretely: if a client's SSE connection
   drops for 10 minutes and 5 notifications were created for them during
   that gap, what happens when they reconnect and open a new SSE stream —
   do they see those 5, or are they gone? What has to happen on the
   client side to make this design actually correct end-to-end?
5. Compare this lesson's dual poll/push design to Module 1's discussion of
   webhooks as an "inverse" real-time pattern. If a *third-party* service
   (not a browser client) needed to be notified the instant a
   notification is created — not poll, not hold open an SSE connection —
   what would you add to `NotificationStore::create`, and how is that
   different from both the SSE broadcast and the polling endpoint this
   lesson already has?
