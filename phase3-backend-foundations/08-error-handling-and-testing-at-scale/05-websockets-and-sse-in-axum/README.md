# 3.8.5 — WebSockets and SSE in `axum`

## At a glance

After this lesson you can:

- Say why a Server-Sent Events response is an ordinary HTTP response that never ends, and read its raw bytes: no `Content-Length`, `Transfer-Encoding: chunked`, one chunk per event.
- Write an SSE endpoint with `Sse`, `Event` and `KeepAlive`, and a WebSocket endpoint with `WebSocketUpgrade`, and say what the `101` handshake between a request and a socket does.
- Fan one message out to many connected clients with `tokio::sync::broadcast`, and handle the subscriber that falls behind.
- Choose between SSE and WebSockets for a given feature, and say what each one makes you build yourself.

**Time:** ~100 minutes · **Prerequisites:**
[3.1.3 — HTTP semantics you must know](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md),
[3.2.1 — Routing, handlers, extractors](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md),
[2.9.3 — Streams](../../../phase2-intermediate/09-async-in-practice/03-streams/README.md)

---

## Why this matters

Every endpoint so far has been a question followed by an answer: the client asks, the server replies, the exchange is over. A lot of real features do not fit that shape. A progress bar for a long import, a "3 new episodes" badge, a live chat, a dashboard that updates by itself: here it is the *server* that knows when something happened, and the client has no way to know when to ask.

In Django you would reach for polling (the page calls an endpoint every few seconds) or for Channels, which is a separate piece of infrastructure with its own worker processes. `axum` needs neither. A handler can hold a connection open and keep writing to it, because a handler is just an `async fn` and the runtime does not mind one that takes an hour. This lesson shows the two standard ways to do it. [3.1.3](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md) ended on a promise: chunked encoding lets a response start before its length is known, and the "streaming responses that never set a `Content-Length`" were deferred to here. SSE is exactly that. WebSockets go further and drop HTTP's request-and-response shape altogether.

---

## The concept

### Two ways to keep a connection pushing

**Server-Sent Events** (SSE) is the simple one. The client sends a normal `GET`. The server answers `200` with `Content-Type: text/event-stream` and then never finishes the body: every time something happens it writes one more small text block, and the browser's `EventSource` hands each block to your JavaScript. Data flows in one direction only, server to client.

**WebSockets** upgrade the connection. The client sends a `GET` that asks to switch protocols, the server answers `101`, and from then on the same TCP connection carries framed messages in *both* directions, with no HTTP in it at all.

```senpai-visual
{"kind":"network","labels":["SSE: client sends one GET","server answers 200 text/event-stream","server writes event, event, event...","WebSocket: client sends GET with Upgrade","server answers 101 Switching Protocols","frames flow both ways until close"]}
```

### SSE is a chunked response that does not end

`examples/01-sse-raw-response.rs` serves three events and then reads the response over a plain `TcpStream`, the type you met in [3.1.1](../../01-networking-and-http-from-scratch/01-tcp-echo-server/README.md). The handler is this:

```rust
async fn ticks() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = (1..=3).map(|n| {
        Ok(Event::default().event("tick").data(format!("tick {n}")))
    });
    Sse::new(tokio_stream::iter(events))
}
```

The example prints every CRLF as a visible `\r\n` and leaves out the `date:` header, which changes on every run:

```text
HTTP/1.1 200 OK\r\n
content-type: text/event-stream\r\n
cache-control: no-cache\r\n
connection: close\r\n
transfer-encoding: chunked\r\n
\r\n
1A\r\n
event: tick
data: tick 1

\r\n
1A\r\n
event: tick
data: tick 2

\r\n
1A\r\n
event: tick
data: tick 3

\r\n
0\r\n
\r\n
```

There is no `Content-Length`, because the server does not know how long this will go on. There is `Transfer-Encoding: chunked`, which is the mechanism [3.1.3](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md) built by hand: each chunk is its size in hexadecimal (`1A` is 26 bytes), `\r\n`, the data, `\r\n`. Each event is one chunk. A real SSE endpoint simply never sends the final `0` chunk, so the response stays open for as long as the connection lives. (This one has three events and ends because the stream ends, and the request asked for `Connection: close`.)

So an SSE stream is not a new protocol. It is HTTP/1.1 doing something it always allowed. That is also why it passes through proxies, load balancers and `curl` without any special support.

### The wire format of an event

The body of the response is plain text in a fixed format. Look at one chunk from above, between the size line and the `\r\n`:

```text
event: tick
data: tick 1

```

An event is a group of `field: value` lines ended by a blank line. The fields are `data` (the payload; several `data:` lines are joined with `\n` by the browser), `event` (a name, so JavaScript can listen to `"tick"` separately from other events), `id` (see "Reconnecting" below) and `retry` (how long the browser should wait before reconnecting, in milliseconds). A line that starts with `:` is a comment, which the browser ignores. That one is how `axum` keeps a quiet connection alive.

You write the formatter yourself in the "Implement" exercise, because writing the format once by hand is the quickest way to stop seeing it as magic.

### SSE in `axum`

`examples/02-sse-ticker-server.rs` is an endless ticker on port 3240. The parts:

```rust
let stream = IntervalStream::new(interval(Duration::from_secs(1))).map(move |_| {
    id += 1;
    Ok(Event::default().event("tick").id(id.to_string()).data(format!("tick {id}")))
});
Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
```

`Sse::new` takes a `Stream` ([2.9.3](../../../phase2-intermediate/09-async-in-practice/03-streams/README.md)) whose items are `Result<Event, E>`. `Event` is a builder: `.event(...)`, `.id(...)`, `.data(...)`, and `.json_data(...)` if you want the payload to be JSON. The `Result` is there so the stream can fail, but in a ticker nothing can, so the error type is `Infallible`. In "Errors you will meet" you leave the `Result` out and read what `axum` says.

`.keep_alive(...)` makes `axum` insert a comment line (`:`) whenever the stream has been quiet for the interval. The default interval is 15 seconds. Without it, a stream that is silent for a minute looks like a dead connection to every proxy on the path, and some of them close it.

Start it and read it with `curl -N` (`-N` turns off curl's output buffering, `--max-time` stops it after 2.5 seconds, which is why curl exits with code 28):

```sh
curl -sNi --max-time 2.5 http://127.0.0.1:3240/ticker
```

```text
HTTP/1.1 200 OK
content-type: text/event-stream
cache-control: no-cache
transfer-encoding: chunked

event: tick
id: 1
data: tick 1

event: tick
id: 2
data: tick 2

event: tick
id: 3
data: tick 3

```

(The `date:` header line is left out, because it changes on every run, and `\r` is stripped.) The first tick arrives at once, because a `tokio` interval's first tick is immediate.

### Reconnecting: `Last-Event-ID`

Networks drop. When an `EventSource` connection breaks, the browser reconnects by itself, and it sends the `id` of the last event it received in a `Last-Event-ID` request header. The ticker above reads that header and carries on from the next number:

```sh
curl -sN --max-time 2.5 -H "Last-Event-ID: 41" http://127.0.0.1:3240/ticker
```

```text
event: tick
id: 42
data: tick 42

event: tick
id: 43
data: tick 43

event: tick
id: 44
data: tick 44

```

Reconnection with resume is built into SSE. With WebSockets it is not: when the socket dies, nothing reconnects, and nothing remembers what the client had seen. You write both. That is the largest practical difference between the two, and the table below puts it in context.

### WebSockets: the handshake is an HTTP request

A WebSocket starts as an ordinary `GET` with three extra headers: `Upgrade: websocket`, `Connection: Upgrade`, and a `Sec-WebSocket-Key` that is a random 16-byte value in base64. The server's answer is the `1xx` class you met in [3.1.3](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.md): `101 Switching Protocols`. `examples/03-ws-handshake-by-hand.rs` does it with a `TcpStream` against an `axum` echo route:

```text
< HTTP/1.1 101 Switching Protocols
< connection: upgrade
< upgrade: websocket
< sec-websocket-accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
< 
> [81, 85, 01, 02, 03, 04, 69, 67, 6f, 68, 6e]
< [81, 05, 68, 65, 6c, 6c, 6f]  ("hello")
```

(The `date:` header is left out, as before. The `<` lines are what the server sent, the `>` line is the one frame the client sent.)

`Sec-WebSocket-Accept` is the server's proof that it understood the request: it is a hash of the client's key plus a fixed string from RFC 6455. The key in this example is the RFC's own sample, and `s3pPLMBiTxaQ9kYGzzhZRbK+xOo=` is the answer the RFC lists for it, which is how the tests in this lesson check a handshake without any hashing code.

After the `101` there is no HTTP. The two lines starting with `>` and `<` are frames. Read the first byte of each: `81` is "final fragment, opcode 1 = text". The second byte is the length, and its top bit says "masked". The client's frame is `85`: masked, length 5, and then a 4-byte mask and the five payload bytes XOR-ed with it (`69 67 6f 68 6e` is `hello` after masking). The server's frame is `05`: not masked, length 5, then `68 65 6c 6c 6f`, which is `hello`. The RFC says a client must mask and a server must not.

### WebSockets in `axum`

The `ws` feature is off by default. This lesson's `Cargo.toml` turns it on:

```toml
axum = { workspace = true, features = ["ws"] }
```

The handler asks for a `WebSocketUpgrade` extractor, and `on_upgrade` hands the finished socket to an `async` function. This is the given code in `src/lib.rs`, trimmed:

```rust
async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            let reply = ws_reply(text.as_str());
            if socket.send(Message::text(reply)).await.is_err() { break; }
        }
    }
}
```

The handler returns the `101` response immediately. The function you pass to `on_upgrade` runs *afterwards*, as its own task, once the connection has switched. `socket.recv().await` gives `Option<Result<Message, Error>>`: `None` when the connection is closed, `Some(Err(_))` when it broke, `Some(Ok(message))` otherwise.

The `Message` type has five variants, checked against `axum` 0.8.9's source:

| Variant | Payload | Meaning |
|---|---|---|
| `Text` | `Utf8Bytes` | a UTF-8 text message |
| `Binary` | `Bytes` | raw bytes |
| `Ping`, `Pong` | `Bytes` | liveness checks; `axum` answers a `Ping` with a `Pong` for you |
| `Close` | `Option<CloseFrame>` | the peer is closing; `axum` answers with its own close |

`Text` holds a `Utf8Bytes`, not a `String`. This is new in `axum` 0.8, and it is the first thing that bites if you learn from an older tutorial. A `Utf8Bytes` is a reference-counted buffer, cheap to clone, that always holds valid UTF-8: `.as_str()` borrows it as `&str`, and `.into()` builds one from a `String` or `&str`. `Message::text("hi")` is the convenience constructor. You will see the error in "Errors you will meet".

The real `handle_socket` in `src/lib.rs` is longer than the snippet. It also pings every 30 seconds, using `tokio::select!` ([2.9.2](../../../phase2-intermediate/09-async-in-practice/02-select-and-cancellation-safety/README.md)) to race `socket.recv()` against a timer. That is the heartbeat, and it is the second thing a WebSocket makes you build. A connection that has been silent might be healthy or might have been dropped by a NAT box ten minutes ago, and neither side finds out until it writes. The periodic ping makes the failure visible, and the browser answers it with a `Pong` automatically.

### Which one to use

| | SSE | WebSocket |
|---|---|---|
| Direction | server to client | both ways |
| Transport | plain HTTP response, chunked | HTTP `101`, then its own frames |
| Payload | UTF-8 text | text or binary |
| Reconnect | built into the browser, with `Last-Event-ID` | you build it |
| Heartbeat | `KeepAlive` comment lines | you build it (ping and pong) |
| Through proxies | normally fine; it is HTTP | needs the proxy to allow `Upgrade` |
| Browser API | `EventSource` | `WebSocket` |

The rule is short. If the data only ever flows *from* the server (notifications, progress, a live feed, a dashboard), use SSE, because the parts that are annoying about WebSockets are already done. If the client sends a steady stream of its own messages (a chat box, a multiplayer game, collaborative editing) or you need binary data, use a WebSocket. Anything the client sends occasionally in an SSE app can be a normal `POST`, which is what `POST /publish` is in this lesson.

### Fan-out: one message, every client

A single connection is not very interesting. The common job is: something happens in one handler, and *every* connected client should hear about it. That needs a channel with many receivers, each getting its own copy. [2.8.3](../../../phase2-intermediate/08-concurrency/03-channels-message-passing/README.md) used `std::sync::mpsc`, where each message goes to exactly one receiver. `tokio::sync::broadcast` is the other shape: many senders, many receivers, every receiver sees every message.

`examples/04-broadcast-fanout.rs` serves `/events` from one `broadcast::Sender`, connects two raw SSE clients, and sends one message:

```rust
let stream = BroadcastStream::new(tx.subscribe())
    .filter_map(|item| item.ok())
    .map(|msg| Ok(Event::default().data(msg)));
Sse::new(stream)
```

```text
subscribers: 2
delivered to 2
client A got "data: konnichiwa\n\n"
client B got "data: konnichiwa\n\n"
```

`tx.subscribe()` creates a new receiver that sees only what is sent *after* it exists. `BroadcastStream` from `tokio-stream` (the `sync` feature) turns that receiver into a `Stream`, so it plugs into `Sse::new`. `tx.send(msg)` returns the number of receivers it reached, and returns an `Err` when there are none, which is not a failure of the system, just nobody listening.

Two details matter in practice. First, `subscribe()` is called in the handler body, *before* the response is returned, not lazily inside the stream. Once the client has seen the response headers, it is already subscribed, so a message sent right then cannot be missed. Second, the channel has a fixed capacity. A client that reads too slowly does not slow down the senders (that would let one bad phone stall everyone): instead, once it is more than `capacity` messages behind, it loses the oldest and its next item is `Err(Lagged(n))`, where `n` is how many it missed. The `.filter_map(|item| item.ok())` above silently drops that. In "Errors you will meet" you see what `unwrap()` does with it, and in "Build" you report it to the client as an event.

```senpai-visual
{"kind":"queue","labels":["POST /publish calls hub.publish","broadcast channel holds recent messages","client A stream reads its own copy","client B stream reads its own copy","slow client falls behind: Lagged(n)","oldest messages are dropped, not the senders"]}
```

### What is tested here, and what is not

The tests in this lesson start a real server on `127.0.0.1:0` (the OS picks a free port) and talk to it over a raw `TcpStream`, so they check the bytes on the wire. The SSE tests decode the chunked encoding with a small helper in `tests/common/mod.rs`. For WebSockets there is no client library in this workspace, so that same file contains a hand-written client: the handshake with the RFC's sample key, one masked frame out, one frame in. It covers text frames under 126 bytes, close frames, and the `Sec-WebSocket-Accept` value. It does *not* cover fragmented messages, binary frames, ping and pong, or the 30-second heartbeat, and nothing here runs a browser. For a real client library, `tokio-tungstenite` is the usual choice. It is not one of this course's dependencies, so you will not need it for the exercises, but you will want it in a real project. Every server a test starts is stopped at the end of the test.

---

## Hands on

```sh
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 01-sse-raw-response
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 03-ws-handshake-by-hand
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 04-broadcast-fanout
```

Example `02` is a server. Start it in one terminal and use the two `curl` commands from "The concept" in another, then stop it with Ctrl+C:

```sh
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 02-sse-ticker-server
```

Then the three broken ones. `06` and `07` fail to compile, `08` panics:

```sh
cargo build -p p3-08-05-websockets-and-sse-in-axum --example 06-sse-item-not-result-broken --features broken
cargo build -p p3-08-05-websockets-and-sse-in-axum --example 07-ws-text-string-broken --features broken
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 08-lagged-unwrap-panic-broken --features broken
```

Then try these:

1. In `01-sse-raw-response`, add `.data("a\nb")` instead of the single-line data. How many `data:` lines does the event have on the wire, and does the chunk size change?
2. In `02-sse-ticker-server`, change the keep-alive interval to 1 second and the ticker interval to 3 seconds, restart, and watch `curl -N`. What shows up between the ticks?
3. In `04-broadcast-fanout`, connect a third client *after* the `send`. Does it receive `konnichiwa`?

---

## Errors you will meet

Every transcript below is the real output for the example named, with this lesson's own `todo!()` warnings left out.

### `E0271` — an SSE stream whose items are not `Result`

```text
error[E0271]: type mismatch resolving `<Iter<IntoIter<Event>> as Stream>::Item == Result<_, _>`
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\05-websockets-and-sse-in-axum\examples\06-sse-item-not-result-broken.rs:9:25
   |
 9 |     let _sse = Sse::new(tokio_stream::iter(events));
   |                -------- ^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<_, _>`, found `Event`
   |                |
   |                required by a bound introduced by this call
   |
   = note: expected enum `Result<_, _>`
            found struct `Event`
   = note: required for `tokio_stream::Iter<std::vec::IntoIter<Event>>` to implement `futures_core::stream::TryStream`
note: required by a bound in `Sse::<S>::new`
  --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\response\sse.rs:64:12
   |
62 |     pub fn new(stream: S) -> Self
   |            --- required by a bound in this associated function
63 |     where
64 |         S: TryStream<Ok = Event> + Send + 'static,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Sse::<S>::new`

error[E0271]: type mismatch resolving `<Iter<IntoIter<Event>> as Stream>::Item == Result<_, _>`
 --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\05-websockets-and-sse-in-axum\examples\06-sse-item-not-result-broken.rs:9:16
  |
9 |     let _sse = Sse::new(tokio_stream::iter(events));
  |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<_, _>`, found `Event`
  |
  = note: expected enum `Result<_, _>`
           found struct `Event`
  = note: required for `tokio_stream::Iter<std::vec::IntoIter<Event>>` to implement `futures_core::stream::TryStream`

For more information about this error, try `rustc --explain E0271`.
error: could not compile `p3-08-05-websockets-and-sse-in-axum` (example "06-sse-item-not-result-broken") due to 2 previous errors
```

**What the compiler is objecting to:** `Sse::new` requires `S: TryStream<Ok = Event>`. A `TryStream` is a `Stream` whose items are `Result`s, and the `note:` says it plainly: expected `Result<_, _>`, found `Event`. The second error is the same mistake reported again at the whole call.

**The fix:** wrap each item in `Ok`, and say what the error type is, `Infallible` when nothing can fail:

```rust
let events = events.into_iter().map(Ok::<Event, Infallible>);
let _sse = Sse::new(tokio_stream::iter(events));
```

**Why this is the fix:** the stream's error type is the error the *source* of the events can hit. For a database cursor or a file read it is a real error, and `axum` ends the stream when one appears. For an in-memory ticker nothing can go wrong, and `Infallible` is the type for "this cannot happen" (the same one a `Router` uses, from [3.2.4](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.md)).

### `E0308` — `Message::Text` is not a `String` any more

```text
error[E0308]: mismatched types
   --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\05-websockets-and-sse-in-axum\examples\07-ws-text-string-broken.rs:9:31
    |
  9 |     socket.send(Message::Text(hello)).await.unwrap();
    |                 ------------- ^^^^^ expected `Utf8Bytes`, found `String`
    |                 |
    |                 arguments to this enum variant are incorrect
    |
note: tuple variant defined here
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\extract\ws.rs:774:5
    |
774 |     Text(Utf8Bytes),
    |     ^^^^
help: call `Into::into` on this expression to convert `String` into `Utf8Bytes`
    |
  9 |     socket.send(Message::Text(hello.into())).await.unwrap();
    |                                    +++++++

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p3-08-05-websockets-and-sse-in-axum` (example "07-ws-text-string-broken") due to 1 previous error
```

**What the compiler is objecting to:** the variant takes a `Utf8Bytes` (the `note:` points at its definition in `axum` 0.8.9's `ws.rs`), and you handed it a `String`. Older tutorials, written against `axum` 0.6 and 0.7, have `Message::Text(String)`.

**The fix:** the compiler's own suggestion, `hello.into()`, or the constructor `Message::text(hello)`.

**Why this is the fix:** `Utf8Bytes` can be built from a `String` or a `&str`, and the conversion cannot fail (a `String` is already valid UTF-8), so `.into()` is enough. The type records that a text message is valid UTF-8, so nobody downstream has to check again.

### A run-time panic: `unwrap` on a lagging subscriber

```text
thread 'main' (41756) panicked at phase3-backend-foundations\08-error-handling-and-testing-at-scale\05-websockets-and-sse-in-axum\examples\08-lagged-unwrap-panic-broken.rs:19:33:
called `Result::unwrap()` on an `Err` value: Lagged(3)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(The number in parentheses is the thread's id and changes every run.)

**What's actually broken:** the example creates a channel with room for 2 messages, sends 5 before the receiver reads any, then `unwrap`s every item. The first item the receiver gets is not a message but `Err(Lagged(3))`: "you missed 3". The type system showed this possibility, since `BroadcastStream` yields `Result`s, and `unwrap` turned it into a crash.

**The fix:** match on the item and treat the lag as data:

```rust
match item {
    Ok(n) => println!("got {n}"),
    Err(BroadcastStreamRecvError::Lagged(missed)) => println!("lagged: missed {missed}"),
}
```

With that, the example prints `lagged: missed 3`, `got 4` and `got 5`.

**Why this is the fix:** in a server this `unwrap` would panic one connection's task every time a phone on a bad network fell behind. A lag is not a bug; it is the channel's way of keeping slow readers from stalling fast senders, and it needs a decision: skip silently, tell the client (the Build exercise's `lagged` event), or disconnect it.

---

## Exercises

### Warm up

<details>
<summary>An SSE response has no <code>Content-Length</code>. How does the client know where one event ends and the next begins, and how does it know the response itself is not over?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

A blank line ends each event, so the client reads until it sees `\n\n`. The response is never over because the server never sends the chunked encoding's final zero-size chunk. It keeps the connection open and writes another chunk whenever something happens.

</details>

<details>
<summary>The browser's connection to your SSE endpoint drops for ten seconds. What happens on the client, and what do you have to write on the server for it to resume correctly?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`EventSource` reconnects by itself and sends the last event `id` it saw in a `Last-Event-ID` header. For the resume to be correct, the server must send an `id` on its events and, on a reconnect, read that header and replay what was missed. Nothing reconnects a WebSocket for you.

</details>

<details>
<summary>A <code>broadcast</code> channel has capacity 3, and a subscriber is 10 messages behind. What does its next <code>recv</code> give, and does the sender slow down?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

An `Err(Lagged(n))` where `n` is the number of messages it lost, and then the oldest messages that are still in the buffer. The sender does not slow down: it never waits for receivers.

</details>

<details>
<summary>You are building a live scoreboard where the browser only watches, and a "reset" button that the admin presses occasionally. SSE or WebSocket, and how does the reset reach the server?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

SSE: the data only flows server to client, and the browser reconnects and resumes for free. The reset button is a normal `POST` request, as `POST /publish` is in this lesson.

</details>

### Repair

Fix all three broken examples:

1. `examples/06-sse-item-not-result-broken.rs` compiles.
2. `examples/07-ws-text-string-broken.rs` compiles.
3. `examples/08-lagged-unwrap-panic-broken.rs` prints `lagged: missed 3`, `got 4`, `got 5` and exits.

### Implement

Two pure functions in `src/lib.rs`:

- `sse_frame`: renders one SSE event as the exact text that goes over the wire.
- `ws_reply`: the answer to one text message on the `/ws` route.

```sh
cargo test -p p3-08-05-websockets-and-sse-in-axum --test sse_frame_test
cargo test -p p3-08-05-websockets-and-sse-in-axum --test ws_reply_test
cargo test -p p3-08-05-websockets-and-sse-in-axum --test ws_wire_test
```

The doc comment above each function is its whole specification, including the exact strings. You never need to read the tests. The first test file has one test, `matches_what_axum_sends`, that also needs the Build exercise's `/events` route, so it fails until Build is done. `ws_wire_test` starts a server and talks to it with the hand-written WebSocket client; it passes once `ws_reply` is implemented, because the given `handle_socket` calls it.

### Build

`Hub::publish`, the `events` handler, and the `publish` handler in the same file: the broadcast room behind `GET /events` and `POST /publish`.

```sh
cargo test -p p3-08-05-websockets-and-sse-in-axum --test hub_test
```

The doc comments are the specification. The two points the tests check beyond "it works" are the ones from "Fan-out": the handler subscribes before it returns, and a lagging subscriber is told with a `lagged` event instead of being cut off. The tests connect two real SSE clients, publish through `POST /publish`, and check the bytes each one receives.

### Challenge (optional)

Make `/events` resumable. Give every published message a number, keep the last 100 messages in a ring buffer inside the `Hub`, and send each event with `id: <number>`. When a client connects with a `Last-Event-ID` header, first replay everything newer than that id from the buffer, then switch to live messages. Nothing tests this one. Check it by hand with `curl -N -H "Last-Event-ID: 3"`: the hard part is not losing or repeating a message at the seam between replay and live, and that is the part to think about before you write code.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Server-Sent Events (SSE) | a never-ending chunked HTTP response of `text/event-stream` events | live feeds, progress, notifications |
| `Sse` / `Event` / `KeepAlive` | `axum`'s response type, event builder and idle-comment sender | every SSE endpoint |
| `Last-Event-ID` | the header a reconnecting browser sends with the last `id` it saw | resuming an SSE stream |
| WebSocket | a connection upgraded from HTTP with `101`, then framed messages both ways | chat, games, collaboration |
| `WebSocketUpgrade` / `WebSocket` | the extractor that does the handshake and the socket you read and write | every WebSocket endpoint |
| `Message` | text (`Utf8Bytes`), binary, ping, pong or close | reading and writing frames |
| heartbeat | a periodic ping so a dead connection is noticed | long-lived WebSockets |
| `broadcast` channel | many senders, many receivers, each gets every message | fan-out to connected clients |
| `Lagged(n)` | the subscriber missed `n` messages because it fell behind | slow-client handling |

### What you now know

- An SSE response is HTTP/1.1 doing what it always could: a chunked body with no `Content-Length` that the server never finishes.
- An event is `field: value` lines ended by a blank line, and `axum`'s `Sse` takes a `Stream` of `Result<Event, E>`.
- A WebSocket is an HTTP request answered with `101`, after which the connection speaks frames: clients mask, servers do not.
- `Message::Text` holds a `Utf8Bytes` in `axum` 0.8, not a `String`.
- SSE reconnects and resumes by itself and goes through proxies as plain HTTP. A WebSocket needs its own reconnect and its own heartbeat, and in return it is bidirectional and can carry binary data.
- `tokio::sync::broadcast` gives every subscriber its own copy of each message, never blocks the sender, and reports a slow subscriber with `Lagged(n)`.

### What comes back later

- **Authenticating requests with middleware, which a WebSocket upgrade request passes through like any other** — [3.7.3 — JWTs and `tower` middleware](../../07-auth-and-security/03-jwt-and-tower-middleware/README.md)
- **What happens to open connections when the server stops** — [3.4.3 — Graceful shutdown, health and readiness](../../04-configuration-and-app-structure/03-graceful-shutdown-health-readiness/README.md)

### Can you explain?

- Why can an SSE endpoint omit `Content-Length`, and what in the raw response tells the client how long each piece is?
- What does a client send to resume an SSE stream after a reconnect, and what does the server have to do with it?
- What are the three headers that start a WebSocket, what does the server answer, and why does that answer count as a `1xx` response?
- Why does a WebSocket need a heartbeat, and who sends it?
- Why does `subscribe()` have to run before the SSE handler returns, and what does `Lagged(n)` tell you?
- For a notification badge and for a multiplayer game, which of SSE and WebSocket would you pick, and why?

---

## Going further

- [`axum::response::sse`](https://docs.rs/axum/0.8.9/axum/response/sse/index.html): `Sse`, `Event`, `KeepAlive`, with examples.
- [`axum::extract::ws`](https://docs.rs/axum/0.8.9/axum/extract/ws/index.html): `WebSocketUpgrade`, `WebSocket` and `Message`, including splitting a socket into a reader and a writer.
- [WHATWG: Server-sent events](https://html.spec.whatwg.org/multipage/server-sent-events.html): the event stream format, reconnection time and `Last-Event-ID`.
- [RFC 6455 — The WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455.html): the handshake (§1.3, source of the sample key used here), framing and masking (§5), and ping and pong (§5.5).
- [`tokio::sync::broadcast`](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html): capacity, `Lagged` and `RecvError`.
