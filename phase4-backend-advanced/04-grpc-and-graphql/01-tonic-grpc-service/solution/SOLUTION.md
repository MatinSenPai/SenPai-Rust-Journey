# Solution

```rust
async fn create_note(
    &self,
    request: Request<CreateNoteRequest>,
) -> Result<Response<Note>, Status> {
    let req = request.into_inner();
    let mut state = self.state.lock().unwrap();

    let id = state.next_id.to_string();
    state.next_id += 1;

    let note = Note {
        id: id.clone(),
        title: req.title,
        body: req.body,
    };
    state.notes.insert(id, note.clone());

    Ok(Response::new(note))
}

async fn get_note(&self, request: Request<GetNoteRequest>) -> Result<Response<Note>, Status> {
    let req = request.into_inner();
    let state = self.state.lock().unwrap();

    match state.notes.get(&req.id) {
        Some(note) => Ok(Response::new(note.clone())),
        None => Err(Status::not_found(format!("note {} not found", req.id))),
    }
}

async fn list_notes(
    &self,
    _request: Request<ListNotesRequest>,
) -> Result<Response<ListNotesResponse>, Status> {
    let state = self.state.lock().unwrap();
    let notes: Vec<Note> = state.notes.values().cloned().collect();
    Ok(Response::new(ListNotesResponse { notes }))
}
```

## The shape of a generated service

`tonic::include_proto!("notes")` pulls in code that `build.rs` generated from
`proto/notes.proto` at compile time (via `tonic_build::compile_protos`, using
`protoc-bin-vendored`'s bundled `protoc` binary since this sandbox has no
system `protoc` — the same trick you'd use on any CI runner or teammate's
machine that hasn't installed `protoc` separately). That generated code gives
you two things: plain data structs (`Note`, `CreateNoteRequest`, ...) that are
just prost `Message` impls — the gRPC equivalent of a `#[derive(Serialize,
Deserialize)]` struct — and a `NotesService` trait (`notes_service_server`
module) with one `async fn` per RPC in the `.proto` file. Implementing that
trait for `NotesServiceImpl` is the entire job; `tonic::async_trait` is there
only because native `async fn` in traits didn't support the `dyn`-safety and
`Send` bounds tonic needs when this crate was first written (the ecosystem has
since largely moved off the macro, but generated server code still uses it
for backwards compatibility).

## `Mutex<NotesState>`, same pattern as everywhere else

`NotesServiceImpl` holds a single `Mutex<NotesState>` guarding an in-memory
`HashMap` and an id counter — exactly the `Arc<Mutex<...>>`-free version of
the pattern from Phase 2's threads-and-mutex lesson (no `Arc` needed here
because tonic wraps the whole service in its own `Arc` internally when you
call `.add_service(...)`, so `&self` is already shared-and-cloneable from the
framework's point of view). Every method takes the lock, does its work, and
lets the guard drop at the end of the block — the same discipline as
`InMemoryQueue` in the previous module's toy queue lesson.

## Errors as values, gRPC-flavored

Compare `get_note`'s `Err(Status::not_found(...))` to Phase 3's axum
`ApiError` enum that implemented `IntoResponse` to turn a domain error into an
HTTP status + JSON body. `tonic::Status` is gRPC's equivalent: a wire-level
error type carrying a `Code` (an enum with variants like `NotFound`,
`InvalidArgument`, `Internal` — gRPC's analogue of HTTP status codes) plus a
human-readable message. Both are cases of the same idea you've now seen three
times (`Result<T, DomainError>` in Phase 1-2, `Result<T, ApiError>` in axum,
`Result<Response<T>, Status>` here): let the type system force every caller to
handle failure explicitly, and centralize the translation from "what went
wrong" to "what the client sees" at the boundary, rather than scattering
`panic!`s or silent defaults through business logic.

## Why no `id` collisions across concurrent `create_note` calls

`next_id` is only ever read and incremented while holding the `Mutex`, so two
concurrent `create_note` RPCs (tonic runs each request on its own tokio task)
can't observe the same value — the second call's lock acquisition blocks
until the first has already advanced the counter and released the lock. This
is the same reasoning as `each_created_note_gets_a_distinct_id` in the test
module, and the same reasoning that made `FOR UPDATE SKIP LOCKED` necessary
once you moved from one process's `Mutex` to `PostgresQueue` in the previous
lesson — a single-process in-memory service can lean on `std::sync::Mutex`
precisely because there's only one process's memory to protect.
