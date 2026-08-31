-- Prefixed `p3_04_03_` for the same reason as module 2's `p3_04_02_widgets`:
-- this database is shared with other lessons in this repo.
CREATE TABLE p3_04_03_anime (
    id BIGSERIAL PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    rating SMALLINT
);
