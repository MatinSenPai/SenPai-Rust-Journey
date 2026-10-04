# 04.4.1 — `tonic` gRPC service

Every backend lesson so far that spoke HTTP spoke JSON over REST (axum,
Phase 3). gRPC is a different contract: you define your service's messages
and methods in a `.proto` file (a schema, not code), a code generator turns
that schema into strongly-typed request/response structs and a
client/server trait pair for your language, and calls travel as binary
Protocol Buffers over HTTP/2 instead of JSON over HTTP/1.1.

## Why gRPC exists alongside REST, not instead of it

- **Service-to-service, not browser-to-server.** gRPC's typical home is
  internal calls between your *own* backend services (e.g.
  `taskforge-worker` calling `taskforge-api`), where both ends are your
  code, generated from the same `.proto`, and you control the deploy of
  both sides together. Browsers can't natively speak gRPC (no built-in
  HTTP/2-trailers + binary-framing client), which is why public,
  browser-facing APIs are still overwhelmingly REST or GraphQL (next
  lesson).
- **Schema-first, not schema-optional.** A REST endpoint's request/response
  shape lives wherever your team's documentation discipline puts it
  (OpenAPI, a wiki page, "read the handler code"). A `.proto` file *is* the
  contract — the generated Rust types and a generated client in, say, Go or
  Python are all mechanically derived from the same source of truth, so
  client and server can never silently drift on field names or types the
  way hand-maintained REST documentation can.
- **Performance.** Binary Protobuf encoding is smaller and faster to
  (de)serialize than JSON, and HTTP/2 multiplexes many calls over one
  connection — real wins at high internal call volume, though rarely the
  deciding factor for a public API with modest traffic.

## Reading `proto/notes.proto`

```proto
service NotesService {
  rpc CreateNote(CreateNoteRequest) returns (Note);
  rpc GetNote(GetNoteRequest) returns (Note);
  rpc ListNotes(ListNotesRequest) returns (ListNotesResponse);
}
```

Every `rpc` here is **unary**: one request message in, one response message
out — the gRPC equivalent of a normal REST call. (gRPC also supports
streaming RPCs — client-streaming, server-streaming, bidirectional — where
either side sends a sequence of messages over one call; genuinely useful for
things like a live log tail, but out of scope for this lesson, which stays
with unary calls to keep the focus on the request/response/error-handling
shape.)

## `build.rs`: compiling the `.proto` into Rust

`build.rs` runs `tonic_build::compile_protos("proto/notes.proto")` at
compile time, which shells out to `protoc` (the Protocol Buffers compiler)
and turns the schema into a Rust module (written to `OUT_DIR`, pulled into
`src/lib.rs` via `tonic::include_proto!("notes")`) containing:

- Plain structs for every `message` (`Note`, `CreateNoteRequest`, ...).
- A `notes_service_server` module with a `NotesService` **trait** (the
  contract you implement — this lesson's `NotesServiceImpl` below) and a
  `NotesServiceServer<T>` wrapper you'd hand to `tonic::transport::Server`
  in a real `main.rs`.
- A `notes_service_client` module with a generated, ready-to-use RPC client.

**A sandbox-specific wrinkle, worth understanding even outside this repo:**
`protoc` is a system binary `tonic-build`/`prost-build` normally expects to
already be installed (`apt install protobuf-compiler`, `brew install
protobuf`, etc.) — and it is genuinely **not installed** in this sandbox,
by design (this repo can't assume every learner's machine, or CI, has it
either). Shelling out to a missing binary from `build.rs` would fail
`cargo build` for this crate — and because this whole repo is one Cargo
workspace, that failure could threaten `cargo build --workspace` for
*everything*, an unacceptable blast radius for one lesson. The fix, visible
at the top of `build.rs`:

```rust
let protoc_path = protoc_bin_vendored::protoc_bin_path()?;
std::env::set_var("PROTOC", protoc_path);
tonic_build::compile_protos("proto/notes.proto")?;
```

[`protoc-bin-vendored`](https://docs.rs/protoc-bin-vendored) is a crate
that ships pre-built `protoc` binaries for every common platform and picks
the right one at build time — no system install required, just a normal
`cargo`-fetched build-dependency. Pointing the `PROTOC` env var at it before
calling `compile_protos` is the whole fix. This was verified end-to-end in
this exact sandbox (`cargo build` succeeds, `.proto` → Rust codegen runs for
real) before this lesson was written this way — it's the cleaner option
precisely because it exercises the real `.proto` → codegen pipeline, rather
than hand-writing what codegen would have produced.

## Errors are values: `tonic::Status`

```rust
async fn get_note(&self, request: Request<GetNoteRequest>) -> Result<Response<Note>, Status> {
```

Every RPC method returns `Result<Response<T>, Status>` — `Status` is gRPC's
error type, carrying a `Code` (`NotFound`, `InvalidArgument`,
`PermissionDenied`, ...) and a message, the direct analogue of an HTTP
status code + JSON error body in axum's `Result<T, ApiError>` pattern from
Phase 3. `get_note` returning `Err(Status::not_found(...))` for a missing
id — rather than, say, an `Ok(Response::new(Note::default()))` with empty
fields, or a panic — is the same "make illegal states unrepresentable, make
failure explicit" discipline you've used everywhere else in this
curriculum, just carried by a different envelope.

## Testing without a running server

Every test in this lesson calls the service's methods **directly** —
`service.create_note(Request::new(...)).await` — no `tonic::transport::Server`
is bound to a port, no client connects over the network. `NotesServiceImpl`
is a plain struct whose methods happen to take `tonic::Request<T>` and
return `Result<Response<T>, Status>`; nothing about that requires an actual
HTTP/2 connection to exercise. This mirrors exactly how
`capstone-taskforge/taskforge-api` tests its axum handlers in-process
against `InMemoryJobStore` rather than spinning up a real server and
`reqwest`-ing it — the network transport is a separate, generated,
already-correct concern; what you're testing is your own request-handling
logic.

## Your task

Open `src/lib.rs`. Implement the three `NotesService` trait methods on
`NotesServiceImpl`: `create_note`, `get_note`, `list_notes`. The generated
`notes` module (from `proto/notes.proto`) is given — you don't write or
edit generated code, only the trait impl that uses it.

## Next

`cargo test -p p4-04-01-tonic-grpc-service`, then the recall questions, then
`solution/SOLUTION.md`.
