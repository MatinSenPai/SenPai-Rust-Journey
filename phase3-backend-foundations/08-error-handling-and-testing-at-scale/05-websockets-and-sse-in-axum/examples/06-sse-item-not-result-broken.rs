//! DELIBERATELY BROKEN — expected: E0271
//!
//! `Sse` wants a stream of `Result<Event, E>`, not a stream of bare `Event`s.

use axum::response::sse::{Event, Sse};

fn main() {
    let events = vec![Event::default().data("tick 1")];
    let _sse = Sse::new(tokio_stream::iter(events));
}
