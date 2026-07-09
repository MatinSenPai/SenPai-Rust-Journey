//! Two `JobStore` implementations (see `taskforge-core`'s `JobStore`
//! trait): [`PostgresJobStore`], the real one, and [`InMemoryJobStore`], a
//! test double with identical semantics used throughout this workspace's
//! test suites so nothing needs a live database to verify its logic. See
//! `../docs/adr/0001-architecture-overview.md`.

mod in_memory;
mod postgres;
mod row;

pub use in_memory::InMemoryJobStore;
pub use postgres::PostgresJobStore;
