# 01.1 — TCP echo server

## Why start below the framework

Every Django view you've ever written assumed a lot of machinery already
existed: a WSGI/ASGI server accepted a raw TCP connection, read bytes off
the wire, parsed them into an HTTP request, matched a URL, and handed you a
tidy `request` object. Phase 3 is about rebuilding that intuition from the
socket up, so that when you write `axum::Router::new().route(...)` two
lessons from now, you know *exactly* what it's doing underneath instead of
treating it as magic. This first lesson has no framework and no
dependencies at all — just the standard library's networking primitives.

## What a socket actually is

A **TCP connection** is a reliable, ordered, two-way byte stream between two
processes (possibly on different machines). "Reliable and ordered" means:
if you write `b"hello"` then `b"world"` on one end, the other end is
guaranteed to read `hello` before `world`, with nothing dropped or
reordered (the OS and the TCP protocol handle retransmission for you). It
does **not** mean the bytes arrive as one neat chunk — a single `write` on
one side can show up as multiple `read`s on the other, or several small
writes can coalesce into one `read`. This matters in a moment.

A **`TcpListener`** binds to an address (`host:port`) and *listens* for
incoming connection attempts. Each accepted connection becomes a
**`TcpStream`** — an open, bidirectional byte pipe you read from and write
to, exactly like a file handle. This is the same relationship as Python's
`socket.socket()` + `.bind()` + `.listen()` + `.accept()` — Rust's
`std::net` module is deliberately similar, just typed and without a GIL.

```rust
use std::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:7878")?;
for stream in listener.incoming() {
    let stream = stream?;
    // handle this one connection
}
```

## Why line-based, and why a generic function

An **echo server** is the "hello world" of networking: whatever the client
sends, the server sends straight back. We'll do it line-by-line (split on
`\n`) rather than byte-for-byte, because it maps directly onto how you'll
read an HTTP request in the next lesson — a line at a time, until you hit
something meaningful (a blank line, a known header).

The core logic in `src/lib.rs`, `run_echo`, is written against **generic**
`R: BufRead` and `W: Write` parameters instead of concrete `TcpStream`
types:

```rust
pub fn run_echo<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<usize> {
    // ...
}
```

This is the same "keep your core logic testable, keep the I/O thin" pattern
from the anime-quote-cli side-quest, applied to networking: a real
`TcpStream` implements both `Read` and `Write`, but so does an in-memory
`std::io::Cursor<&[u8]>` / `Vec<u8>`. That means `run_echo`'s tests in
`tests/echo_test.rs` never open a real socket at all — they just hand it
byte buffers. Only `serve_once`/`main.rs` (the thin shell) ever touches an
actual `TcpListener`. You'll see this exact split again in every Phase 3
lesson: pure logic in testable functions, I/O pushed to the edges.

## Concurrency: a thread per connection

`TcpListener::incoming()` hands you one connection at a time on the
*accepting* thread. If you called `run_echo` directly in that loop, a slow
or silent client would block every other client from being served — the
whole server would stall on one connection. The classic fix (and still a
perfectly reasonable one for learning purposes, before Phase 3 module 2
introduces `tokio`'s async model) is **one OS thread per connection**:

```rust
for stream in listener.incoming() {
    let stream = stream?;
    std::thread::spawn(move || {
        // handle this connection on its own thread
    });
}
```

Threads aren't free (each one costs real OS resources — a stack, a
scheduler slot), which is exactly *why* async runtimes like `tokio` exist:
they let you juggle thousands of concurrent connections on a handful of OS
threads instead of one-per-connection. You'll feel that contrast directly
once `axum` enters the picture.

## Your task

Implement the two `todo!()`s in `src/lib.rs`:

1. `run_echo` — read lines from `reader` until EOF (`read_line` returning
   `Ok(0)`), writing each one straight back out to `writer`, and return the
   total number of bytes read.
2. `serve_once` — accept exactly one connection on a bound `TcpListener`,
   split it into a reader/writer pair with `TcpStream::try_clone`, and run
   `run_echo` over it. (A real server loops forever accepting connections;
   `serve_once` handles one, which is all the tests need and keeps the
   concurrency code in `main.rs`, which isn't test-covered by design — you
   can't easily unit-test "loops forever.")

`src/main.rs` is already written for you — it loops on `serve_once`-style
handling forever, spawning a thread per connection, and binds to
`127.0.0.1:7878`.

## Try it for real

```sh
cargo run -p p3-01-01-tcp-echo-server &
printf 'hello\nworld\n' | nc 127.0.0.1 7878
# or, without netcat:
python3 -c "
import socket
s = socket.create_connection(('127.0.0.1', 7878))
s.sendall(b'hello\n')
print(s.recv(1024))
"
```

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
