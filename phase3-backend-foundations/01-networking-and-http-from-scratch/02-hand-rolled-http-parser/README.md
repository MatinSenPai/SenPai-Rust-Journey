# 3.1.2 — Hand-rolled HTTP parser

## At a glance

After this lesson you can:

- Explain the four exact wire-format rules an HTTP/1.1 request follows — and say precisely what goes wrong if you get one of them (the `\r\n` line separator) wrong.
- Write an HTTP parser and serializer entirely by hand: raw bytes in, a `Result<HttpRequest, HttpParseError>` out, with every kind of malformed input getting its own named variant instead of one generic failure.
- Say exactly why `axum`, two lessons from now, hands back an automatic 400 when `Json<T>` or `Path<T>` doesn't match — because today you walked that exact path by hand.

**Time:** ~70 minutes · **Prerequisites:**
[3.1.1 — TCP echo server](../01-tcp-echo-server/README.md)

---

## Why this matters

The last lesson gave you a `TcpStream` — a raw byte pipe, no structure at all. It also pointed out that this is exactly what Django's WSGI/ASGI server sees under the hood, before it hands your view a tidy `request` object with `.method` and `.path` and `.headers`. This lesson fills exactly that gap: from a raw `&[u8]` to a structured `HttpRequest`, and back.

The point isn't that HTTP is complicated — it isn't. The point is that HTTP is a **text, line-oriented protocol**, exactly as much as a CSV file or a config file is, just riding a socket instead of a disk. Whatever looks like "magic" in Django, DRF, gunicorn, or `axum` (two lessons from now) is exactly this: reading text, splitting it on a handful of fixed separators, and being honest about every place that text could be malformed. Today you do that once, by hand — so it stops being magic from here on.

---

## The concept

### The shape of an HTTP/1.1 request

A real HTTP/1.1 request, on the wire, looks exactly like this:

```text
GET /anime?status=watching HTTP/1.1\r\n
Host: localhost:7879\r\n
User-Agent: curl/8.4.0\r\n
Accept: */*\r\n
\r\n
```

This is what's called the **wire format**: the exact byte-for-byte shape actually sent over the network, distinct from whatever Rust type represents it once parsed. Four things to notice:

1. **Lines are separated by `\r\n`** (carriage return + line feed), not just `\n` — a holdover from older text protocols, and a real trap if you forget it: split by hand on bare `\n` and that `\r` stays baked into whatever comes right before it.
2. **The first line** (the "request line") has the shape `{METHOD} {PATH} {VERSION}` — three space-separated tokens.
3. **Each header line** has the shape `Name: value` — a colon, then the value. Header *names* are case-insensitive by spec (`Host` and `host` are the same header) — real clients and proxies rely on exactly that.
4. **A completely blank line** (`\r\n\r\n` at the end) marks the end of the headers. For a `GET`, nothing follows it — this lesson deliberately stops right there. A `POST`/`PUT` body, and the `Content-Length` header that says how many bytes to read for one, is [module 2 — `axum`](../../02-axum-and-rest-api-design/README.md)'s job.

### From bytes to text: why you validate first

Reading off a socket gives you `&[u8]`, not `&str`. The first thing any parser has to do is confirm those bytes are UTF-8 at all — a client (or an attacker) can send whatever garbage bytes it wants:

```rust
fn bytes_off_a_socket() -> Vec<u8> {
    vec![0x47, 0x45, 0x54, 0xff, 0xfe]
}

let good: &[u8] = b"GET / HTTP/1.1";
match std::str::from_utf8(good) {
    Ok(text) => println!("valid:   {text:?}"),
    Err(e) => println!("invalid: {e}"),
}
let bad = bytes_off_a_socket();
match std::str::from_utf8(&bad) {
    Ok(text) => println!("valid:   {text:?}"),
    Err(e) => println!("invalid: {e}"),
}
```

```text
valid:   "GET / HTTP/1.1"
invalid: invalid utf-8 sequence of 1 bytes from index 3
```

`std::str::from_utf8` makes exactly this promise: either a valid `&str`, or an `Err` saying where it broke — no panic, no silently dropping anything. This is the first time in this phase `Result` is doing its job against data that is genuinely *untrusted* (not a toy exercise input), which is exactly why nothing downstream of this function is allowed to assume the input is well-formed.

### Every kind of malformed input gets its own named error

`parse_request` returns a `Result<HttpRequest, HttpParseError>`, and instead of one generic `ParseFailed`, `HttpParseError` gives every kind of malformed input its own variant:

```rust
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HttpParseError {
    #[error("request bytes are not valid UTF-8")]
    InvalidUtf8,
    #[error("request is empty")]
    EmptyRequest,
    #[error("malformed request line: {0:?}")]
    MalformedRequestLine(String),
    #[error("malformed header line: {0:?}")]
    MalformedHeaderLine(String),
}
```

This is exactly the instinct DRF's serializer `.errors` dict follows too: tell the caller *what exactly* was wrong, not just *that* something was — except here it isn't a coding convention doing the telling, it's the type system. Code catching this error can `match` on the variant and know exactly which stage failed.

```senpai-visual
{"kind":"result","labels":["parse_request(bytes)","Err(InvalidUtf8)","Err(MalformedRequestLine)","Err(MalformedHeaderLine)","Ok(HttpRequest)"]}
```

### Header names aren't case-sensitive

Per the HTTP spec, header *names* don't distinguish case at all. `str::eq_ignore_ascii_case` does exactly that comparison, without allocating a lowercased copy of either side:

```rust
let sent_by_client = "Host";
for candidate in ["host", "HOST", "Host", "Content-Type"] {
    println!(
        "{candidate:?} matches {sent_by_client:?}: {}",
        candidate.eq_ignore_ascii_case(sent_by_client)
    );
}
```

```text
"host" matches "Host": true
"HOST" matches "Host": true
"Host" matches "Host": true
"Content-Type" matches "Host": false
```

This rule covers *names* only, nothing else. `Method`, for instance, is case-sensitive by spec — `"get"` is not `Method::Get`; you'll see that right now in `src/lib.rs`, and work with it directly in "Challenge".

### Building a response: the exact mirror of the same path

The reverse direction — Rust data into HTTP bytes — follows the exact same rules, in reverse:

```rust
fn response_bytes(status: u16, reason: &str, headers: &[(&str, &str)], body: &str) -> Vec<u8> {
    let mut out = format!("HTTP/1.1 {status} {reason}\r\n");
    for (name, value) in headers {
        out.push_str(&format!("{name}: {value}\r\n"));
    }
    out.push_str(&format!("Content-Length: {}\r\n", body.len()));
    out.push_str("\r\n");
    out.push_str(body);
    out.into_bytes()
}
```

```text
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 14

Hello, world!
```

Status line, headers (which must include `Content-Length` — without it, a client waiting for the body has no way to know when it ends, since the connection itself might stay open), a blank line, then the body. `Content-Length` must be the body's **byte** length, not `.chars().count()` — a multi-byte UTF-8 character is more than one byte, and a client reads exactly that many bytes off the wire.

```senpai-visual
{"kind":"network","labels":["TCP bytes in","parse_request","HttpRequest","HttpResponse::to_bytes","TCP bytes out"]}
```

---

## Hands on

```sh
cargo run -p p3-01-02-hand-rolled-http-parser --example 01-bytes-must-be-validated
cargo run -p p3-01-02-hand-rolled-http-parser --example 02-splitting-the-wire-format
cargo run -p p3-01-02-hand-rolled-http-parser --example 03-case-insensitive-header-names
cargo run -p p3-01-02-hand-rolled-http-parser --example 04-building-a-response-by-hand
```

Then the two broken ones:

```sh
cargo run -p p3-01-02-hand-rolled-http-parser --example 05-newline-only-split-broken --features broken
cargo build -p p3-01-02-hand-rolled-http-parser --example 06-header-returns-string-not-str-broken --features broken
```

Then try these:

1. In `02-splitting-the-wire-format`, change the request line to `"GET / HTTP/1.1"` (no query) — how does the `target` output change?
2. In `03-case-insensitive-header-names`, add another candidate like `"HoSt"` — guess before you run it.
3. In `04-building-a-response-by-hand`, change the body to a string with a Persian letter in it (e.g. `"سلام"`). Is `Content-Length` still equal to the **character** count?

---

## Errors you will meet

### A run-time panic — a stray `\r` left over from the wrong separator

```text
thread 'main' (9568) panicked at phase3-backend-foundations\01-networking-and-http-from-scratch\02-hand-rolled-http-parser\examples\05-newline-only-split-broken.rs:20:5:
assertion `left == right` failed: the version should not carry a stray \r
  left: "HTTP/1.1\r"
 right: "HTTP/1.1"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(That number in parentheses is the thread's own id and changes every run; the rest of the message is always the same.)

**What's actually broken:** `examples/05-newline-only-split-broken.rs` deliberately splits on `\n` alone, not `\r\n`. Split the request line that way and the `\r` that was really part of the `\r\n` separator stays baked into the last token on that line — here, `version`. The code compiles and runs fine; it only panics exactly where `assert_eq!` compares the actual value against `"HTTP/1.1"`.

**The fix:** change the split from `raw.split('\n')` to `raw.split("\r\n")`.

**Why this is the fix:** now the whole `\r\n` is consumed as the separator and never survives inside any token. This is exactly what your own `parse_request` has to do too — and exactly why its specification says "on `\r\n`, not bare `\n`."

### `E0308` — `header` returns a `&String`, not a `&str`

```text
error[E0308]: mismatched types
  --> phase3-backend-foundations\01-networking-and-http-from-scratch\02-hand-rolled-http-parser\examples\06-header-returns-string-not-str-broken.rs:14:9
   |
13 |       fn get(&self, name: &str) -> Option<&str> {
   |                                    ------------ expected `Option<&str>` because of return type
14 | /         self.entries
15 | |             .iter()
16 | |             .find(|(k, _)| k.eq_ignore_ascii_case(name))
17 | |             .map(|(_, v)| v)
   | |____________________________^ expected `Option<&str>`, found `Option<&String>`
   |
   = note: expected enum `Option<&str>`
              found enum `Option<&String>`
help: try converting the passed type into a `&str`
   |
17 |             .map(|(_, v)| v).map(|x| x.as_str())
   |                             ++++++++++++++++++++

For more information about this error, try `rustc --explain E0308`.
```

**What the compiler is objecting to:** `self.entries` is a `Vec<(String, String)>`, so after `.find()` and `.map(|(_, v)| v)`, what comes out is `Option<&String>` — a reference to the `String` sitting inside the tuple, not a `&str`. The signature says `Option<&str>`; those are two different types, and Rust doesn't automatically coerce `&String` to `&str` inside an `Option` (unlike an argument position, where that coercion is free).

**The fix:** the compiler's own suggestion — one more `.map(|x| x.as_str())` — works, but the cleaner version writes `v.as_str()` directly inside the first closure:

```rust
.map(|(_, v)| v.as_str())
```

**Why this is the fix:** `.as_str()` views that same `&String` borrow as a `&str`, with no clone. This is exactly what Phase 2's deref coercion covered: the automatic conversion happens at the call site, not once a type has already come back out of an `Option`.

---

## Exercises

### Warm up

<details>
<summary>Is <code>Host</code> the same header as <code>host</code>, per the HTTP spec?</summary>

Yes. Header *names* are case-insensitive by spec — these two are exactly the same header.

</details>

<details>
<summary>If a client sends bytes that aren't valid UTF-8, what does <code>parse_request</code> do?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

Neither panics nor silently drops them — it returns `Err(HttpParseError::InvalidUtf8)`. `std::str::from_utf8` is the very first check the function makes.

</details>

<details>
<summary>What separates the lines of an HTTP request — bare <code>\n</code>, or <code>\r\n</code>?</summary>

Think it through before you look at the answer.

</details>

<details>
<summary>Answer</summary>

`\r\n`. Split on bare `\n` alone and the `\r` stays baked into the last token of the previous line — exactly what `examples/05-newline-only-split-broken.rs` shows you.

</details>

<details>
<summary>What does this print?</summary>

```rust
let m = Method::parse("get");
println!("{m:?}");
```

</details>

<details>
<summary>Answer</summary>

```text
Other("get")
```

HTTP methods, unlike header names, are case-sensitive by spec. `"get"` matches no named arm, so it falls to `Other`.

</details>

### Repair

Fix both broken examples:

1. Fix `examples/05-newline-only-split-broken.rs` so it doesn't panic — change the separator from `'\n'` to `"\r\n"`.
2. Fix `examples/06-header-returns-string-not-str-broken.rs` so it compiles — without changing `get`'s return type.

### Implement

Four functions in `src/lib.rs`:

```sh
cargo test -p p3-01-02-hand-rolled-http-parser
```

Each one is fully specified in its own doc comment — you should never need to open the test file to know what to build:

- `Method::parse` — map a request-line token like `"GET"` to the matching variant.
- `parse_request` — turn raw bytes into a valid `HttpRequest`, or the exact error naming whatever was malformed.
- `HttpRequest::header` — case-insensitive header lookup.
- `HttpResponse::to_bytes` — serialize back into wire format, with a correct `Content-Length`.

### Build

Add a `pub fn query_params(&self) -> Vec<(String, String)>` method on `HttpRequest` that turns the query string into key/value pairs — the same thing `request.GET` does for you automatically in Django.

Specification: if `self.query` is `None`, return an empty `Vec`. Otherwise split the query on `&` into individual pairs; split each pair on its *first* `=` into a key and a value — a pair with no `=` gets an empty-string value. Keep order and duplicates exactly as they arrived: `?tag=a&tag=b` should give two separate entries, which is exactly why the return type is a `Vec`, not a `HashMap`.

### Challenge (optional)

Make the request line parse more strictly correct instead of accidentally too strict: right now, extra consecutive spaces — say `"GET  / HTTP/1.1"`, with two spaces between `GET` and `/` — break parsing. Change your own `parse_request` in `src/lib.rs` so this parses cleanly too, without breaking a genuinely single space between real tokens. (Fully rebuilding HTTP's status codes and its stricter spec-level behaviors is [3.1.3](../03-http-semantics-you-must-know/README.md)'s job — this is just one small corner of parser strictness, on its own.)

---

## Wrapping up

| Term | What it means | Where you'll use it |
|---|---|---|
| Wire format | the exact byte-for-byte shape actually sent over the network | reading/writing any text protocol, not just HTTP |
| `std::str::from_utf8` | fallible validation of `&[u8]` into `&str` | the first step for any untrusted bytes |
| `HttpParseError` | one named variant per kind of malformed input, not a generic error | matching an error to exactly what caused it |
| `eq_ignore_ascii_case` | case-insensitive string comparison with no extra allocation | header-name lookup |
| `Content-Length` | the body's **byte** length, not its character count | building any hand-rolled HTTP response |

### What you now know

- The four wire-format rules of an HTTP/1.1 request — the `\r\n` line separator, the request line's shape, header names being case-insensitive, and the blank line that ends them.
- Why `parse_request`'s first job is confirming UTF-8, and what could go wrong without that check.
- Why `HttpParseError` has one variant per kind of malformed input instead of a single generic one.
- Header names are case-insensitive by spec; HTTP methods are not — and you no longer mix the two up.
- How to build an HTTP response by hand, `Content-Length` computed from byte length included.

### What comes back later

- **The full status-code catalog, content negotiation, keep-alive, and chunked encoding** — [3.1.3 — HTTP semantics you must know](../03-http-semantics-you-must-know/README.md)
- **`POST`/`PUT` bodies, read according to `Content-Length`** — [Module 2 — `axum` & REST API design](../../02-axum-and-rest-api-design/README.md)
- **This exact query string, decoded automatically — `axum`'s `Query<T>` extractor** — [3.2.1 — Routing, handlers, extractors](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.md)

### Can you explain?

- Why must `parse_request` validate the bytes as UTF-8 *before* anything else runs?
- What separates HTTP's lines, and what exactly happens if you split on the wrong thing?
- Why are header names case-insensitive but HTTP methods aren't?
- Why does `HttpParseError` have four separate variants instead of one `ParseFailed`?
- Should `Content-Length` be a byte length or a character count? Why does that distinction actually matter?

---

## Going further

- [MDN — HTTP messages](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages) — the same wire shape, illustrated.
- [RFC 9112 — HTTP/1.1](https://www.rfc-editor.org/rfc/rfc9112.html) — the official spec; §3 (request line) and §5 (field syntax) are exactly what you implemented today.
- [`std::str::from_utf8` documentation](https://doc.rust-lang.org/std/str/fn.from_utf8.html) — the full signature and the shape of `Utf8Error`.
- [`str::eq_ignore_ascii_case` documentation](https://doc.rust-lang.org/std/primitive.str.html#method.eq_ignore_ascii_case) — why this beats `.to_lowercase() ==` (no fresh allocation).
