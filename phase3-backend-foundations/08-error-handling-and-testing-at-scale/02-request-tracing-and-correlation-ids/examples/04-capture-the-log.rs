//! A subscriber that writes into memory, so the program (or a test) can read its
//! own log back. `LogBuffer` is given in `src/lib.rs`.
//!
//! Run: `cargo run -p p3-08-02-request-tracing-and-correlation-ids --example 04-capture-the-log`

use p3_08_02_request_tracing_and_correlation_ids::LogBuffer;

fn main() {
    let buffer = LogBuffer::new();
    let guard = tracing::subscriber::set_default(buffer.subscriber());

    let id = "abc-123";
    let span = tracing::info_span!("request", request_id = %id);
    span.in_scope(|| tracing::info!(status = 200, "finished"));
    drop(guard);

    println!("{:?}", buffer.contents());
    println!(
        "contains the id: {}",
        buffer.contents().contains("request_id=abc-123")
    );
}
