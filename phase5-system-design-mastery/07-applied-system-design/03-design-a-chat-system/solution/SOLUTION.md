# Solution

```rust
pub fn room_sender(&self, room: &str) -> broadcast::Sender<ChatMessage> {
    let mut rooms = self.rooms.lock().unwrap();
    if let Some(sender) = rooms.get(room) {
        return sender.clone();
    }
    let (sender, _receiver) = broadcast::channel(100);
    rooms.insert(room.to_string(), sender.clone());
    sender
}
```

`broadcast::channel(100)` returns a `(Sender, Receiver)` pair, but nobody
needs the freshly-created `Receiver` here — each real client subscribes
its *own* receiver later via `sender.subscribe()` inside `handle_socket`.
The initial receiver is bound to `_receiver` and dropped immediately;
this is fine and deliberate (the underlying channel stays alive as long
as the `Sender` — or any subscriber — holds a reference to it, not
because of this particular unused receiver).

## Should the sender receive their own message?

Whether to echo the sender's own message back is a genuine, defensible
product decision either way — some chat UIs render your own outgoing
message optimistically the instant you hit send (never waiting for a
round trip) and would find a server echo redundant or even confusing if
it arrives out of order relative to other UI state; others render nothing
until the server confirms, in which case the echo *is* the confirmation.
This lesson broadcasts unconditionally because it's the simpler default —
one code path, no special-casing. To exclude the sender, `handle_socket`
would need to know, at broadcast time, "which `Receiver` belongs to the
connection that just sent this" — which the current design doesn't track,
since every subscriber to a room's channel looks identical from the
`Sender`'s point of view. The straightforward fix: attach a unique
connection id to each subscriber (e.g., a `HashMap<ConnectionId,
Receiver>` instead of raw `subscribe()`, or simpler, have each
`send_task` compare the `ChatMessage`'s `username` against its own and
skip re-displaying — a client-side filter rather than a server-side one,
which is honestly the more common real-world choice, since the server
broadcasting to everyone uniformly is simpler and the client already
knows which messages are its own).

## Where room isolation actually comes from

Not from any check inside `handle_socket` — there's no `if room == other_room`
anywhere. It's structural: each room gets its own, entirely separate
`broadcast::channel`, and a `Sender`'s `.send(...)` only ever reaches
`Receiver`s that called `.subscribe()` on *that specific* `Sender`
instance. Two different rooms' `Sender`s are two unrelated channels that
share nothing — the isolation is a consequence of `HashMap<String,
Sender<...>>` giving each room key its own independent channel, not
anything actively enforced at message-send time.

## What happens to a slow subscriber

`Receiver::recv()` returns `Err(RecvError::Lagged(n))` — not silent data
loss disguised as success, and not a panic either. `Lagged(n)` tells the
caller exactly how many messages it missed, so `handle_socket`'s
`while let Ok(msg) = receiver.recv().await` loop as written would
actually **exit** the moment a client falls behind by more than 100
messages (since `Err(Lagged(_))` doesn't match the `Ok(msg)` pattern,
breaking the `while let`) — a real, honest limitation worth naming: a
sufficiently slow client currently gets silently disconnected from the
broadcast loop rather than being told "you missed some messages, catch
up." A production version would likely match on `Err(RecvError::Lagged(n))`
explicitly and either notify the client or keep looping instead of
falling out of the `while let`.

## The never-cleaned-up rooms cost

Every room ever joined, even once, stays in `self.rooms` for the lifetime
of the process — a `HashMap` entry plus one `broadcast::Sender`'s
allocation (small, but not free) per room, forever. For a long-running
chat server with many short-lived or one-off room names (think: a support
ticket system spinning up a new room per ticket), this is a genuine, slow
memory leak. The fix would need `ChatServer` to track, per room, how many
active subscribers currently exist (`Sender::receiver_count()` already
exposes this) and periodically (or on each disconnect) remove rooms whose
count has dropped to zero — a small background sweep task or a check
inside `handle_socket`'s cleanup path when a connection ends.

## Why `127.0.0.1:0`, not a fixed port

Binding a fixed port like `3000` would mean every test in this file (and
any test in a completely different file running concurrently in the same
`cargo test` invocation) that tried to bind that same port would fail
with "address already in use" — `cargo test`'s default concurrent
execution runs multiple `#[tokio::test]` functions at once, each calling
`spawn_server()` independently. Port `0` is the standard OS convention for
"assign me any currently-free ephemeral port" — the OS guarantees no two
concurrent binds ever collide, which is exactly what lets this lesson's
three tests (and any number of future ones) each boot their own isolated
server without needing `#[serial]` or any other cross-test coordination,
the same "make isolation free instead of coordinating around a shared
resource" idea the distributed-rate-limiter lesson's unique Redis keys
used, just applied to TCP ports instead of Redis keys.
