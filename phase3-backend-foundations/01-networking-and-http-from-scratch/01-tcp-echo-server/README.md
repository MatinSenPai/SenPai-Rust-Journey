# 3.1.1 — TCP echo server

## At a glance

After this lesson you can:

- Explain why a TCP connection only guarantees that bytes arrive in order and
  intact, never that write boundaries survive — and watch that with your own
  eyes on a real socket: two separate writes arriving in a single read.
- Write an accept loop with `TcpListener` and `TcpStream`, and say in one
  sentence why `run_echo`'s core is generic over `R: BufRead, W: Write`
  instead of a concrete `TcpStream`.
- Read a server that hangs after one line, find and fix the missing EOF check
  that caused it, and explain why a real server spawns a fresh thread per
  connection instead of serving clients one at a time.

**Time:** ~55 minutes · **Prerequisites:**
[2.3.2 — Generic functions and structs, bounds, `where`](../../../phase2-intermediate/03-traits-and-generics/02-generic-functions-and-structs/README.md) ·
[2.8.1 — Threads, `Mutex`, `Arc`](../../../phase2-intermediate/08-concurrency/01-threads-mutex-arc/README.md)

---

## Why this matters

Every Django view you've written so far had the luxury of assuming a lot of
heavy machinery already existed: a WSGI/ASGI server accepted a raw TCP
connection, read bytes off the wire, parsed them into an HTTP request,
matched a URL against it, and handed you a tidy `request` object. Starting
right here, Phase 3 rebuilds exactly that machinery from the lowest level
that exists — the socket itself — so that a few lessons from now, when
you're working with a real framework, it reads as "the same thing you just
built, automated" instead of as a black box.

This first lesson deliberately has no framework and no dependency at all —
just the standard library's own networking primitives. An **echo server** —
send back whatever the client sends, unchanged — is the smallest network
service you can write, which is exactly why it's the right place to meet a
hard TCP fact head-on that no framework ever hides from you, only smooths
over: bytes don't arrive the shape you sent them in, they arrive as one
continuous stream. Everything else you build in Phase 3 — hand-parsing an
HTTP request next lesson, and `axum` itself in module 2 — sits on top of
this exact fact.

---

## The concept

### A TCP connection is a byte stream, not a queue of messages

A **TCP connection** is a reliable, ordered, bidirectional byte stream
between two processes (possibly on different machines). "Reliable and
ordered" means: if one side writes `b"hello\n"` then `b"world\n"` in two
separate `write` calls, the other side is guaranteed to read `hello` before
`world` — nothing lost, nothing reordered (the OS and TCP itself handle
retransmitting lost packets for you).

What that guarantee does *not* cover is the **boundary** between writes. A
single `read` can hand you several writes at once — or, the other way
around, one large `write` can take several separate `read`s to fully arrive.
The code below proves it on a real loopback socket: one thread accepts and,
after a short pause, does exactly one `read`; the main thread makes two
separate `write_all` calls.

```rust
let listener = TcpListener::bind("127.0.0.1:0")?;
let addr = listener.local_addr()?;

let reader = thread::spawn(move || {
    let (mut stream, _) = listener.accept().unwrap();
    thread::sleep(Duration::from_millis(200)); // let both writes land first
    let mut buf = [0u8; 64];
    let n = stream.read(&mut buf).unwrap();
    println!("one read() call got {n} bytes: {:?}", String::from_utf8_lossy(&buf[..n]));
});
```

```rust
let mut client = TcpStream::connect(addr)?;
client.write_all(b"hello\n")?; // first write() call
client.write_all(b"world\n")?; // second, separate write() call

reader.join().unwrap();
```

```text
one read() call got 12 bytes: "hello\nworld\n"
```

A single `read()` picked up both `write_all` calls at once. The full,
runnable version of this is `examples/01-two-writes-one-read.rs`. This is
what "byte stream" means in practice: TCP is not a message queue where each
`write` becomes its own packet on the other end — it's one continuous pipe
of bytes, and deciding where one "message" ends is entirely up to your own
code (here: by looking for `\n`).

### `TcpListener` listens; `accept()` hands back a `TcpStream`

A **`TcpListener`** binds to an address (`host:port`) and listens for
incoming connection attempts. Each accepted connection becomes a
**`TcpStream`** — an open, bidirectional byte pipe you read from and write
to exactly like a file handle.

```rust
use std::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:7878")?;
for stream in listener.incoming() {
    let stream = stream?;
    // handle this one connection
}
```

If you've worked with Python, this is exactly the relationship between
`socket.socket()` + `.bind()` + `.listen()` + `.accept()` — `std::net` is
deliberately similar, just strictly typed and with no GIL anywhere in sight.
Where the analogy bends: `TcpListener::bind` does both `bind()` and
`listen()` in one call, and the `Result` it hands back is the exact same
`Result<T, E>` from Phase 1 — a taken port or a permissions error is a
value, not an exception.

```senpai-visual
{"kind":"network","labels":["TcpListener::bind claims a port","client connects over TCP","accept() returns a TcpStream","read + write like a file handle"]}
```

### Reading a line at a time needs `BufRead`, not just `Read`

This lesson's echo server works a line at a time (split on `\n`), not byte
by byte — exactly what you'll need next lesson to read an HTTP request: a
line at a time, until you hit something meaningful.

`std::io::Read` only gives you raw bytes; it has no notion of a "line" at
all. **`std::io::BufRead`** adds a buffered layer on top and, with it, the
`read_line` method — exactly what's needed. `TcpStream` doesn't implement
`BufRead` on its own (only `Read` and `Write`), so it has to be wrapped in a
`BufReader` first:

```rust
use std::io::{BufRead, BufReader};

let reader = BufReader::new(stream); // read_line is available now
```

### A testable core: generic over `R: BufRead, W: Write`

The core logic in `src/lib.rs`, `run_echo`, is written against **generic**
`R: BufRead` and `W: Write` parameters instead of the concrete `TcpStream`
type:

```rust
pub fn run_echo<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<usize> {
    // ...
}
```

This is the same "keep the core logic testable, keep I/O thin" pattern from
Phase 2's anime-quote-cli side quest, applied to networking: a real
`TcpStream` implements both `Read` and `Write`, but so does an in-memory
`std::io::Cursor<&[u8]>` / `Vec<u8>`. That's what lets `run_echo`'s tests in
`tests/echo_test.rs` never open a real socket at all — they just hand it a
byte buffer and get another one back. Only `serve_once` (and, after it,
`main.rs`) ever touches a real `TcpListener`. You'll see this exact split
again in every Phase 3 lesson: pure logic in testable functions, I/O pushed
to the edges.

### Concurrency: one thread per connection

`TcpListener::incoming()` hands you connections one at a time, on the
thread that's doing the accepting. Call `run_echo` directly in that loop and
one slow or silent client blocks every other client from being served — the
whole server stalls on a single connection. The classic fix (and, before
module 2 introduces `tokio`'s async model, still a perfectly reasonable one
for learning) is **one OS thread per connection**:

```rust
for stream in listener.incoming() {
    let stream = stream?;
    std::thread::spawn(move || {
        // handle this connection on its own thread
    });
}
```

Threads aren't free — each one claims a real stack and a scheduler slot from
the OS — which is exactly why async runtimes like `tokio` exist: they let
you juggle thousands of concurrent connections on a small pool of real OS
threads instead of one per connection. You've already seen `async`/`await`
and `tokio`'s basics in 2.8.5/2.8.6; this lesson deliberately sets them aside
and stays with plain threads, because module 2 — once `axum` is introduced —
is exactly where you'll watch that thread get replaced by a real async
runtime and feel the difference directly.

```senpai-visual
{"kind":"concurrency","labels":["main thread: listener.incoming()","thread::spawn per connection","client A's thread runs run_echo","client B's thread runs run_echo","a slow client only blocks its own thread"]}
```

---

## Hands on

```sh
cargo run -p p3-01-01-tcp-echo-server --example 01-two-writes-one-read
```

```text
one read() call got 12 bytes: "hello\nworld\n"
```

Now run the two broken ones — deliberately broken, and their results are the
whole subject of the next section:

```sh
cargo run -p p3-01-01-tcp-echo-server --example 02-double-mut-borrow-broken --features broken
cargo run -p p3-01-01-tcp-echo-server --example 03-missing-eof-check-broken
```

The first one doesn't compile at all. The second one compiles and runs,
prints one line, and then never returns — interrupt it with Ctrl+C.

---

## Errors you will meet

### `E0499` — borrowing the same `stream` as mutable twice

The natural first attempt at building a reader and a writer out of the same
connection is to build both from the same variable:

```text
error[E0499]: cannot borrow `stream` as mutable more than once at a time
  --> phase3-backend-foundations\01-networking-and-http-from-scratch\01-tcp-echo-server\examples\02-double-mut-borrow-broken.rs:14:18
   |
13 |     let reader = BufReader::new(&mut stream);
   |                                 ----------- first mutable borrow occurs here
14 |     let writer = &mut stream;
   |                  ^^^^^^^^^^^ second mutable borrow occurs here
15 |     run_echo(reader, writer)
   |              ------ first borrow later used here

For more information about this error, try `rustc --explain E0499`.
```

**What the compiler is actually objecting to:** `BufReader::new(&mut stream)`
takes the first mutable borrow of `stream` and keeps it alive until
`run_echo` is called (since `reader` is used there). The next line tries to
take a *second* mutable borrow of that same `stream` — exactly the aliasing
rule from Phase 1: any number of shared borrows, or exactly one mutable
borrow, never both.

**The fix:** instead of borrowing the same `stream` twice, call
`TcpStream::try_clone` to get a fully independent second handle to the same
underlying OS socket:

```rust
let writer = stream.try_clone()?;
let reader = BufReader::new(stream);
run_echo(reader, writer)
```

**Why this is the fix:** `try_clone` doesn't duplicate the socket — it
duplicates the *handle* (under the hood, a `dup()` of the file descriptor).
`stream` and the cloned handle are now two fully separate, owned values, so
each can move into `run_echo` independently with no borrow conflict at all.
Reading from one and writing to the other sees exactly the same TCP
conversation a single `stream` would have.

### A runtime hang — the missing `Ok(0)` check

This isn't even a compiler error: the program compiles and runs cleanly.

```text
$ cargo run -p p3-01-01-tcp-echo-server --example 03-missing-eof-check-broken
hello
```

And that's it. Nothing else ever prints, and the program never returns — you
have to interrupt it with Ctrl+C.

**What's actually happening:** `read_line` returns `Ok(0)` once there's
nothing left to read (EOF) — not an error, an `Ok` of length zero. The
broken code never checks for that: it clears the line, calls `read_line`
again, and — because every `read_line` after EOF immediately returns that
same `Ok(0)` again, without ever blocking — the loop never `break`s.
Writing an empty string doesn't error either (it just writes zero bytes), so
the loop spins silently, forever, with nothing in it that ever waits for
anything.

**The fix:** check the value `read_line` returns, and break out once it's
zero:

```rust
let bytes_read = reader.read_line(&mut line)?;
if bytes_read == 0 {
    break; // EOF — nothing left to read
}
```

**Why this is the fix:** `read_line`'s `Ok(0)` is the *only* signal meaning
"the other side closed its write half" — an empty line (`"\n"`) is one byte,
not zero. Without this check, the code assumes every `Ok` means "a new line
arrived," when EOF means exactly the opposite: nothing new is ever coming.

---

## Exercises

### Warm up

<details>
<summary>If one side writes <code>b"hello\n"</code> then <code>b"world\n"</code> in two separate <code>write</code> calls, is the other side guaranteed to see them as two separate <code>read</code>s?</summary>

Think it through before checking the answer.

</details>

<details>
<summary>Answer</summary>

No. TCP only guarantees order and complete delivery, not that write
boundaries survive. A single `read` can pick up both writes at once — exactly
what `examples/01-two-writes-one-read.rs` showed you.

</details>

<details>
<summary>Why is <code>run_echo</code> generic over <code>R: BufRead, W: Write</code> instead of concrete <code>TcpStream</code> parameters?</summary>

Think it through before checking the answer.

</details>

<details>
<summary>Answer</summary>

So the core logic stays testable. A real `TcpStream` implements both `Read`
and `Write`, but so does an in-memory `Cursor<&[u8]>`/`Vec<u8>` — so tests
never need to open a real socket at all.

</details>

<details>
<summary>Does this compile?</summary>

```rust
let reader = BufReader::new(&mut stream);
let writer = &mut stream;
run_echo(reader, writer)
```

</details>

<details>
<summary>Answer</summary>

No — `E0499`. `BufReader::new(&mut stream)` takes the first mutable borrow
and keeps it alive; the next line tries to take a second one from the same
`stream`. The fix is `stream.try_clone()` instead of borrowing twice.

</details>

<details>
<summary>A client sends <code>"hi"</code> with no trailing <code>\n</code> and immediately closes its connection. Does the next <code>read_line</code> return <code>Ok(0)</code>, or <code>Ok(2)</code> with <code>line == "hi"</code>?</summary>

Think it through before checking the answer.

</details>

<details>
<summary>Answer</summary>

`Ok(2)` with `line == "hi"`. As long as at least one fresh byte arrives,
`read_line` hands it back as `Ok(n)` — even if the line ended without a
`\n`. `Ok(0)` is reserved for the call *after* that, once there's truly
nothing left.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/02-double-mut-borrow-broken.rs` by taking a
   `stream.try_clone()` instead of borrowing the same `stream` twice.
2. Fix `examples/03-missing-eof-check-broken.rs` by adding the
   `bytes_read == 0` check and a `break`, so it returns cleanly once the
   input is exhausted instead of spinning forever.

### Implement

Two functions in `src/lib.rs`, each fully specified by its own doc comment:

```sh
cargo test -p p3-01-01-tcp-echo-server
```

- `run_echo` — read lines from `reader` one at a time until EOF, writing
  each one straight back out to `writer` as it arrives, and return the total
  number of bytes read.
- `serve_once` — accept exactly one connection from `listener`, get an
  independent reader and writer handle to it, and run `run_echo` over the
  pair.

Once the tests are green, try it once more against a real socket —
`cargo run -p p3-01-01-tcp-echo-server` brings the server up on
`127.0.0.1:7878`; in another terminal:

```sh
python3 - <<'PY'
import socket
s = socket.create_connection(("127.0.0.1", 7878))
s.sendall(b"hello\n")
s.sendall(b"world\n")
s.shutdown(socket.SHUT_WR)
data = b""
while chunk := s.recv(4096):
    data += chunk
print(data)
PY
```

```text
b'hello\nworld\n'
```

### Build

Write your own variant of `run_echo` — one that uppercases each line before
echoing it back, say, or prefixes each line with a running counter
(`1: hello`, `2: world`, ...). Test it exactly the way the existing tests
do — an in-memory `Cursor`/`Vec<u8>`, no real socket at all. Confirm your
version still recognizes EOF correctly and still hands back a final line
even without a trailing `\n`.

### Challenge (optional)

**Part one.** Extend your fixed `examples/03-missing-eof-check-broken.rs`
with a real test: a real client that closes its connection mid-line (no
trailing `\n`). Confirm `serve_once` echoes back exactly that partial piece
and returns cleanly instead of hanging — the same behavior the
`echoes_a_final_line_with_no_trailing_newline` test already proves against a
`Cursor`, this time against a real `TcpStream`.

**Part two.** Write a function that returns just the count of distinct lines
echoed, instead of the echoed text itself. This is the last time you'll see
"read a line at a time until you hit something meaningful" this simply —
next lesson applies the exact same idea to parsing a real HTTP request.

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| TCP connection | reliable, ordered, bidirectional byte stream; doesn't preserve write boundaries | anywhere you have to decide, on raw bytes, where your own protocol's "message" ends |
| `TcpListener` | binds to an address and listens for incoming connections | every TCP server's accept loop |
| `TcpStream` | one accepted connection; `Read` + `Write`, exactly like a file handle | reading/writing bytes to one specific client |
| `BufRead` | the `std::io` trait adding `read_line`/`.lines()` on top of `Read` | line-oriented protocols — this lesson's echo, next lesson's HTTP requests |
| thread-per-connection | each accepted connection handled on its own thread, so one slow client can't block the rest | this lesson's `main.rs`; replaced by `tokio` tasks once module 2 begins |

### What you now know

- Why TCP only guarantees order and complete delivery, not write
  boundaries — and you've seen it on a real socket.
- The `TcpListener` / `accept()` / `TcpStream` relationship, and exactly how
  it maps onto Python's `socket()`+`bind()`+`listen()`+`accept()`.
- Why `BufRead` is needed for line-at-a-time reading, and why the server's
  core logic is generic over it — so it's testable with no real socket at
  all.
- Why `try_clone` is necessary (`E0499`), and why forgetting `read_line`'s
  `Ok(0)` check is a runtime hang, not a compiler error.
- Why a real server spawns a thread per connection, and exactly what a
  thread costs to spawn.

### What comes back later

- **Parsing a real protocol over this same line-at-a-time stream (an HTTP
  request)** — [3.1.2 — Hand-rolled HTTP parser](../02-hand-rolled-http-parser/README.md)
- **This same thread-per-connection loop, replaced by `tokio` tasks under
  `axum`** — [Module 2 — `axum` & REST API design](../../02-axum-and-rest-api-design/README.md)

### Can you explain?

- What does "TCP is a byte stream, not a queue of messages" mean, with a
  concrete example?
- What's the relationship between `TcpListener`, `accept()`, and
  `TcpStream`, and what's the Python equivalent?
- Why is `run_echo` generic over `R: BufRead, W: Write` instead of
  `TcpStream`?
- Why does `serve_once` need `try_clone`, and what error do you get if you
  hand it `&mut stream` twice instead?
- What exactly happens when `read_line`'s `Ok(0)` check is missing, and why
  is that a hang rather than a compile error?

---

## Going further

- [The Rust Book — Final Project: Building a Multithreaded Web Server](https://doc.rust-lang.org/book/ch21-00-final-project-a-web-server.html) — walks this exact path, from a raw `TcpListener` to a thread pool.
- [`std::net` docs](https://doc.rust-lang.org/std/net/index.html) — the whole module: `TcpListener`, `TcpStream`, `UdpSocket`, and the rest.
- [`std::io::BufRead` docs](https://doc.rust-lang.org/std/io/trait.BufRead.html) — `read_line`, `.lines()`, and the rest of the buffered-reading API.
