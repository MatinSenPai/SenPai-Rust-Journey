-- Job state is stored per ADR-0003: a `status` tag column plus the
-- nullable columns each variant needs (attempt/next_attempt_at/error).
-- Nothing in this schema *enforces* that only the right columns are set
-- for a given status — that guarantee lives in Rust, at the JobStore
-- boundary (see taskforge-storage/src/postgres.rs). A stretch extension
-- (noted in ADR-0003) is adding CHECK constraints for defense in depth.
CREATE TABLE jobs (
    id UUID PRIMARY KEY,
    job_type TEXT NOT NULL,
    payload JSONB NOT NULL,
    status TEXT NOT NULL,
    attempt INTEGER,
    next_attempt_at TIMESTAMPTZ,
    error TEXT,
    attempts INTEGER NOT NULL DEFAULT 0,
    max_attempts INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

-- Supports both `claim_next`'s WHERE clause and `list`'s "newest first"
-- ordering without a sequential scan once the table has real volume — see
-- phase3-backend-foundations/05-database-design-and-query-performance.
CREATE INDEX idx_jobs_claimable ON jobs (status, next_attempt_at);
CREATE INDEX idx_jobs_type_created ON jobs (job_type, created_at DESC);
