# Checkpoint

1. `create_note`, `get_note`, and `list_notes` all take
   `Request<T>`/return `Result<Response<T>, Status>`, not just plain `T` /
   `Result<T, Status>`. What extra information does `tonic::Request<T>`
   carry beyond the message body itself (look at what methods it has beyond
   `.into_inner()`)? Can you think of a real use for that extra information
   in a service that needs authentication?
2. `get_note` returns `Status::not_found(...)` for a missing id. What gRPC
   status code (`tonic::Code`) would you reach for if `CreateNoteRequest`
   arrived with an empty `title`, and the service wanted to reject that
   before creating anything? Why is a specific, meaningful status code more
   useful to a caller than always returning `Status::internal(...)`?
3. This lesson's tests call `service.get_note(...)` directly — no server,
   no client, no network. What, concretely, does that test *not* prove
   about the service that a real end-to-end test (spinning up
   `tonic::transport::Server`, connecting a generated client, making a real
   call) would? Is that gap worth caring about for this lesson's purposes —
   why or why not?
4. `NotesServiceImpl` stores notes in a `Mutex<NotesState>` — the exact
   same shape as `InMemoryQueue` from the previous module's toy queue
   lesson. If this service needed to survive a process restart (notes
   preserved across deploys), what's the minimal change to make, and which
   earlier lesson's code would you borrow the pattern from?
5. Re-read the `build.rs` walkthrough in the README. In your own words,
   explain why a `build.rs` that fails is a bigger deal in this repository
   specifically than it would be in a typical standalone project — what
   does "one Cargo workspace" have to do with it?
