# Checkpoint

Answer these in your own words before moving on — there's no code for this
lesson, the point is whether you can explain each idea precisely, not
whether a test passes.

1. `phase3-backend-foundations/01-networking-and-http-from-scratch/02-hand-rolled-http-parser`
   makes you handle `\r\n` line endings, validate UTF-8, and look up headers
   case-insensitively, all by hand. For each of those three, explain
   specifically why HTTP/2's binary framing makes it a non-issue — not just
   "HTTP/2 is binary," but what property of the binary format removes that
   particular ambiguity.
2. HTTP/2 multiplexes many streams over one TCP connection and eliminates
   application-layer head-of-line blocking. Explain why that same design
   introduces a *worse* head-of-line blocking problem one layer down, at the
   TCP level, than HTTP/1.1's "six parallel connections" approach had — and
   how HTTP/3 fixes it.
3. `capstone-taskforge/taskforge-api/src/error.rs` maps `JobError::NotCancellable`
   to a 409, and `phase3-backend-foundations/07-error-handling-and-testing-at-scale/01-consistent-error-envelopes`
   maps a failed `validator::Validate` check to a 400. Explain why one is a
   409-shaped problem and the other isn't, even though both are "the request
   got rejected." Then explain why the second one used 400 instead of the
   arguably-more-precise 422, and why that's a defensible choice rather than
   a mistake.
4. `POST /jobs` in `taskforge-api` returns 201 Created, even though the job
   itself won't actually *run* until a worker process claims it later.
   Explain why 201 is still the right code here, and describe a
   (hypothetical) change to how `taskforge-api` enqueues jobs that would make
   202 Accepted the more honest choice instead.
5. A client library is deciding whether to automatically retry a failed
   request. Explain why "retry on any non-2xx" and "never retry automatically"
   are both wrong default policies, using the difference between a 403, a
   429, and a 503 as your three concrete examples of how the right behavior
   differs per code.
