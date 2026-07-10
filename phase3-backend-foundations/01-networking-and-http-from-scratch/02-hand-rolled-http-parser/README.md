# 01.2 — Hand-rolled HTTP parser

## What `axum` is hiding from you

An HTTP request is just **text sent over a TCP connection** (module 1's raw
bytes), in a specific, line-oriented shape. When Django's dev server (or
gunicorn, or `axum`, two lessons from now) hands you a `request` object with
`.method`, `.path`, `.headers`, it has already done exactly the parsing this
lesson makes you do by hand. Once, deliberately, so it's never a black box
again.

## The shape of an HTTP/1.1 request

```
GET /anime?status=watching HTTP/1.1\r\n
Host: localhost:7878\r\n
User-Agent: curl/8.4.0\r\n
Accept: */*\r\n
\r\n
```

Four things to notice:

1. **Lines are separated by `\r\n`** (carriage return + line feed), not just
   `\n` — a holdover from HTTP's text-protocol ancestors, and a real trap if
   you forget it (`"...HTTP/1.1\r"` with a stray `\r` left on the end of a
   naively-`\n`-split line will silently break comparisons downstream).
2. **The first line** (the "request line") is
   `{METHOD} {PATH} {VERSION}` — three space-separated tokens.
3. **Each header line** is `Name: value` — a colon, then whitespace, then
   the value. Header *names* are case-insensitive by spec (`Host` and
   `host` are the same header) — real HTTP clients and proxies rely on this,
   so a parser that only recognizes lowercase `host` is subtly broken.
4. **A blank line** (`\r\n\r\n` at the end, i.e. an empty line by itself)
   marks the end of the headers. For a `GET` request there's no body after
   it — this lesson deliberately stops there. (`POST`/`PUT` bodies, and the
   `Content-Length` header that tells you how many bytes to read for one,
   are a real can of worms — out of scope here, `axum` handles that for you
   starting next module.)

## Bytes in, structured data out — and why that can fail

A socket read gives you `&[u8]` (raw bytes), not a `&str`. The first thing
any parser has to do is confirm those bytes are valid UTF-8
(`std::str::from_utf8`) before treating them as text at all — a client (or
an attacker) can send whatever garbage bytes it wants, and "assume it's
valid UTF-8" is exactly the kind of assumption that turns into a crash or a
security bug in production code. This is `Result`-shaped error handling
doing real work: `parse_request` returns
`Result<HttpRequest, HttpParseError>`, and every place the input could be
malformed — invalid UTF-8, a request line missing a piece, a header line
with no colon — is its own named `HttpParseError` variant instead of a
single generic "parse failed" catch-all. That's the same instinct DRF's
serializer `.errors` dict encourages (tell the caller *what* was wrong, not
just *that* something was), just enforced by the type system instead of by
convention.

## Building a response by hand

The other direction — Rust data back into HTTP bytes — is the mirror image:

```
HTTP/1.1 200 OK\r\n
Content-Length: 13\r\n
\r\n
Hello, world!
```

Status line, headers (crucially including `Content-Length` — without it,
clients waiting for the body have no way to know when it ends, since the
connection itself might stay open), a blank line, then the body.

## Your task

Implement the `todo!()`s in `src/lib.rs`:

- `Method::parse` — map a request-line token like `"GET"` to a `Method`
  variant, falling back to `Method::Other(String)` for anything unrecognized.
- `parse_request` — turn raw request bytes into a validated `HttpRequest`.
- `HttpRequest::header` — case-insensitive header lookup.
- `HttpResponse::to_bytes` — serialize a response back into wire format,
  including a correct `Content-Length`.

`src/main.rs` wires these into an actual tiny server on `127.0.0.1:7879`
that returns a greeting for `GET /` and a 404 for anything else — try it
with `curl -v http://127.0.0.1:7879/` and `curl -v http://127.0.0.1:7879/nope`
and read the raw response `curl -v` prints.

## Checkpoint

`CHECKPOINT.md`, then `solution/SOLUTION.md`.
