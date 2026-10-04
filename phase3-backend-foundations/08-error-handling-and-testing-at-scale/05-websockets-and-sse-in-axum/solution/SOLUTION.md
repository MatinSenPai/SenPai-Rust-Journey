# Solution — 3.8.5 WebSockets and SSE in `axum`

## `sse_frame`

```rust
pub fn sse_frame(event: Option<&str>, id: Option<u64>, data: &str) -> String {
    let mut out = String::new();
    if let Some(name) = event {
        out.push_str(&format!("event: {name}\n"));
    }
    if let Some(n) = id {
        out.push_str(&format!("id: {n}\n"));
    }
    for line in data.split('\n') {
        out.push_str(&format!("data: {line}\n"));
    }
    out.push('\n');
    out
}
```

Each field is one line and the event ends with an extra blank line. The only subtle part is `data`: the format has no way to put a newline inside a `data:` value, so each line of the payload gets its own `data:` line, and the browser joins them back with `\n`. `"".split('\n')` yields one empty piece, which is why empty data gives `data: ` and `"a\n"` gives two lines, `a` and an empty one. `matches_what_axum_sends` compares the result with the bytes `axum`'s own `Event` produces for the same event.

## `ws_reply`

```rust
pub fn ws_reply(text: &str) -> String {
    if text == "ping" {
        "pong".to_string()
    } else if let Some(rest) = text.strip_prefix("echo ") {
        rest.to_string()
    } else {
        format!("unknown: {text}")
    }
}
```

`strip_prefix("echo ")` includes the space in the prefix, so the bare word `echo` falls through to `unknown:`, and `echo  two spaces` keeps its second space in the reply. Keeping this a pure `&str -> String` function is what makes the protocol testable without a socket: `ws_reply_test` needs no server, and `ws_wire_test` only has to prove the plumbing around it.

## `Hub::publish`

```rust
pub fn publish(&self, msg: &str) -> usize {
    self.tx.send(msg.to_string()).unwrap_or(0)
}
```

`broadcast::Sender::send` returns `Ok(receivers_reached)`, and `Err` only when there are no receivers at all. For a chat room nobody listening is a normal state, not an error, so `unwrap_or(0)` turns it into the count it is.

## `events`

```rust
let stream = BroadcastStream::new(hub.subscribe()).map(|item| {
    let event = match item {
        Ok(msg) => Event::default().event("chat").data(msg),
        Err(BroadcastStreamRecvError::Lagged(n)) => {
            Event::default().event("lagged").data(n.to_string())
        }
    };
    Ok::<Event, Infallible>(event)
});
Sse::new(stream)
    .keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
    .into_response()
```

`hub.subscribe()` runs in the handler body, before the response exists, so by the time a client has read the headers it is already subscribed. That is what `publish_reaches_every_event_stream` relies on: it connects two clients, publishes, and expects `"2"` back. Subscribing lazily instead (say, inside an `async` block that only runs when the body is first polled) would register the receiver later, and a message published in that gap would be lost. A lag becomes data in the stream (`Err(Lagged(n))` mapped to a `lagged` event) instead of an `unwrap`, so a slow client is informed and the stream carries on. The `Infallible` annotation tells the compiler what the stream's error type is, because nothing in the closure can fail.

## `publish`

```rust
(StatusCode::ACCEPTED, hub.publish(&body).to_string())
```

`202 Accepted` and not `200` because the message was handed to the room, not confirmed received by anyone. The body is the delivery count in decimal.

## On the challenge (optional)

Keep a `VecDeque<(u64, String)>` of the last 100 messages and an incrementing id inside the `Hub`, behind a `Mutex`. On connect, take the lock once and in the same critical section both subscribe to the channel and copy out every buffered message with an id greater than the client's `Last-Event-ID`. Chain the replay in front of the live stream. Doing the subscribe and the copy under one lock is what closes the seam: with them separate, a message published in between is either replayed twice or never seen.

## What the tests do and do not cover

They run a real server on an ephemeral port and read raw bytes: decoded chunks for SSE, hand-built frames for WebSockets. Fragmented or binary frames, ping and pong, the 30-second heartbeat and the 15-second keep-alive are not tested, because each needs either a long wait or a full client library. The `lagged` path is only checked for shape, because whether a lag happens depends on how fast the server task drains the room, which the test does not control.
