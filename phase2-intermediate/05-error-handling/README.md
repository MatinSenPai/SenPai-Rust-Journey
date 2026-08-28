# 05 — Error handling

Phase 1 gave you `Result` and `?`. This module is about the error *type*
itself: designing one that carries real information, chains to the error
that caused it, and tells a caller what actually went wrong instead of just
that something did.

1. [Custom error types and `std::error::Error`](01-custom-error-types/README.md)
2. [Source chains and `Box<dyn Error>`](02-error-source-chains/README.md)
3. [`thiserror` versus `anyhow`, and the library/binary boundary](03-thiserror-and-anyhow/README.md)
4. [Designing an error taxonomy for a service](04-error-taxonomy-for-a-service/README.md)
