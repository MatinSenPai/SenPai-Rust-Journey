# Checkpoint

1. `migrating_creates_the_widgets_table` starts by `DROP TABLE IF EXISTS`
   and manually deleting the tracking row for version 1 before calling
   `run_migrations`. Why is that cleanup necessary for this specific test
   to be meaningful, given that `_sqlx_migrations` persists across test
   runs in the same database?
2. If you added a second file, `0002_add_widget_price.sql`, containing
   `ALTER TABLE p3_04_02_widgets ADD COLUMN price_cents BIGINT NOT NULL;`,
   and ran `run_migrations` against a database that already had migration
   `0001` applied, what would happen? What would happen if you instead
   *edited* the existing `0001_create_widgets.sql` file after it had
   already been applied to a database?
3. `run_migrations` embeds migration SQL into the compiled binary at
   compile time. What's the practical deployment benefit of that, compared
   to reading `.sql` files from disk at runtime?
4. Compare this lesson's `_sqlx_migrations`-based tracking to Django's
   `django_migrations` table — same underlying idea? What's genuinely
   different about the workflow (hint: think about `makemigrations`).
5. `insert_widget` and `count_widgets` use `sqlx::query_scalar` rather
   than `sqlx::query_as`. Given you've already implemented both, what's
   the actual difference — when would `query_scalar` not be the right
   choice?
6. This lesson connects to its own dedicated `p3_04_02_migrations`
   database instead of the shared `taskforge` database every other
   Postgres lesson in this repo reuses. Re-read the README's explanation
   of why. In your own words: what specifically about `_sqlx_migrations`
   makes it unsafe to share across two unrelated `sqlx::migrate!` call
   sites, in a way that simply prefixing table names (the fix every other
   lesson uses) does not solve?
7. Every test in this lesson carries `#[serial(p3_04_02_migrations_db)]`.
   Try removing it from just `inserted_widgets_are_counted` (leave it on
   the other two) and run `cargo test -p p3-04-02-migrations -- --ignored`
   several times in a row. What do you observe, and why does removing it
   from only *one* of the three tests still leave the suite unsafe?
