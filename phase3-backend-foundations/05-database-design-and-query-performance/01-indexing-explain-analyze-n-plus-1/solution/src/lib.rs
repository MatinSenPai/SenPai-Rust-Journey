//! A tiny blog (`posts` + `comments`) used to feel two things in your own
//! hands: what an index changes about `EXPLAIN ANALYZE` output (see
//! `README.md` for worked examples), and what the N+1 query problem looks
//! like once it's not just a Django ORM warning anymore. See `README.md`
//! for the theory before you touch any `todo!()` here.
//!
//! Split the same way `phase4-backend-advanced/03-background-jobs-and-
//! message-queues/01-postgres-skip-locked-toy-queue` splits `InMemoryQueue`
//! from `PostgresQueue`: the pure, no-I/O logic (`group_joined_rows`) is a
//! plain function you can unit test with zero infrastructure, and it backs
//! every non-`#[ignore]`d test in this file. Everything that actually talks
//! to Postgres (`BlogStore`'s methods) is `#[ignore]`d and needs a live
//! database — see `README.md` for how to start one and run those tests.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use sqlx::PgPool;

/// A blog post. Deliberately minimal — this lesson is about query shape,
/// not schema design.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub id: i64,
    pub title: String,
}

/// A comment belonging to exactly one post.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub body: String,
}

/// A post with all of its comments attached — the shape you actually want
/// to hand back to an API client (compare to a DRF serializer nesting a
/// `CommentSerializer(many=True)` inside a `PostSerializer`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PostWithComments {
    pub post: Post,
    pub comments: Vec<Comment>,
}

/// One row of the flattened result of `posts LEFT JOIN comments`. Because
/// it's a `LEFT JOIN` (not a plain `JOIN`), a post with zero comments still
/// produces exactly one row here, with `comment_id`/`comment_body` both
/// `None` — a plain `JOIN` would make that post vanish from the result set
/// entirely, which is *not* what "give me every post, with comments if it
/// has any" means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JoinedRow {
    pub post_id: i64,
    pub post_title: String,
    pub comment_id: Option<i64>,
    pub comment_body: Option<String>,
}

/// Groups flattened join rows back into one `PostWithComments` per post,
/// preserving the order posts first appear in `rows`. Pure, synchronous, no
/// I/O — this is the part of "collapse the N+1 into one query" that's
/// actually worth unit testing without a database at all, and it's what
/// `BlogStore::posts_with_comments_batched` calls after it fetches the raw
/// rows.
pub fn group_joined_rows(rows: Vec<JoinedRow>) -> Vec<PostWithComments> {
    let mut posts: Vec<PostWithComments> = Vec::new();
    let mut index_of: HashMap<i64, usize> = HashMap::new();

    for row in rows {
        let index = *index_of.entry(row.post_id).or_insert_with(|| {
            posts.push(PostWithComments {
                post: Post {
                    id: row.post_id,
                    title: row.post_title.clone(),
                },
                comments: Vec::new(),
            });
            posts.len() - 1
        });

        if let Some(comment_id) = row.comment_id {
            posts[index].comments.push(Comment {
                id: comment_id,
                post_id: row.post_id,
                body: row.comment_body.unwrap_or_default(),
            });
        }
    }

    posts
}

/// A Postgres-backed store over `posts` and `comments`, instrumented with a
/// query counter so tests can *prove* how many round trips each access
/// pattern below takes — the same number you'd see by eyeballing Django's
/// debug toolbar SQL panel after an unguarded `for post in Post.objects.all()`.
pub struct BlogStore {
    pool: PgPool,
    pub query_count: AtomicU64,
}

impl BlogStore {
    /// Connects and creates `posts`/`comments` if they don't already exist.
    /// A real system would use `sqlx::migrate!` (see
    /// `04-postgres-and-sqlx/02-migrations`) — this toy inlines `CREATE
    /// TABLE IF NOT EXISTS` to keep the lesson to a single file.
    pub async fn connect(database_url: &str) -> Result<Self, sqlx::Error> {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS posts ( \
                id BIGSERIAL PRIMARY KEY, \
                title TEXT NOT NULL \
             )",
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS comments ( \
                id BIGSERIAL PRIMARY KEY, \
                post_id BIGINT NOT NULL REFERENCES posts(id), \
                body TEXT NOT NULL \
             )",
        )
        .execute(&pool)
        .await?;

        Ok(Self {
            pool,
            query_count: AtomicU64::new(0),
        })
    }

    /// Test helper: wipes both tables so each test starts from a known,
    /// empty state — the same reason `postgres-skip-locked-toy-queue`'s
    /// `#[ignore]`d tests each `DELETE FROM toy_jobs` first.
    pub async fn reset(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM comments")
            .execute(&self.pool)
            .await?;
        sqlx::query("DELETE FROM posts").execute(&self.pool).await?;
        self.reset_query_count();
        Ok(())
    }

    pub fn reset_query_count(&self) {
        self.query_count.store(0, Ordering::SeqCst);
    }

    pub fn query_count(&self) -> u64 {
        self.query_count.load(Ordering::SeqCst)
    }

    pub async fn create_post(&self, title: &str) -> Result<i64, sqlx::Error> {
        self.query_count.fetch_add(1, Ordering::SeqCst);
        let (id,): (i64,) = sqlx::query_as("INSERT INTO posts (title) VALUES ($1) RETURNING id")
            .bind(title)
            .fetch_one(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn create_comment(&self, post_id: i64, body: &str) -> Result<i64, sqlx::Error> {
        self.query_count.fetch_add(1, Ordering::SeqCst);
        let (id,): (i64,) =
            sqlx::query_as("INSERT INTO comments (post_id, body) VALUES ($1, $2) RETURNING id")
                .bind(post_id)
                .bind(body)
                .fetch_one(&self.pool)
                .await?;
        Ok(id)
    }

    /// One query: every post, ordered by id.
    pub async fn list_posts(&self) -> Result<Vec<Post>, sqlx::Error> {
        self.query_count.fetch_add(1, Ordering::SeqCst);
        let rows: Vec<(i64, String)> = sqlx::query_as("SELECT id, title FROM posts ORDER BY id")
            .fetch_all(&self.pool)
            .await?;
        Ok(rows
            .into_iter()
            .map(|(id, title)| Post { id, title })
            .collect())
    }

    /// One query: every comment for a single post, ordered by id.
    pub async fn comments_for_post(&self, post_id: i64) -> Result<Vec<Comment>, sqlx::Error> {
        self.query_count.fetch_add(1, Ordering::SeqCst);
        let rows: Vec<(i64, i64, String)> =
            sqlx::query_as("SELECT id, post_id, body FROM comments WHERE post_id = $1 ORDER BY id")
                .bind(post_id)
                .fetch_all(&self.pool)
                .await?;
        Ok(rows
            .into_iter()
            .map(|(id, post_id, body)| Comment { id, post_id, body })
            .collect())
    }

    /// **The N+1 pattern, deliberately.** One query for every post, then
    /// one *more* query per post for its comments — exactly the shape of
    /// `for post in Post.objects.all(): post.comments.all()` without
    /// `prefetch_related`. Fine at N=1, quietly ruinous at N=10,000.
    pub async fn posts_with_comments_naive(&self) -> Result<Vec<PostWithComments>, sqlx::Error> {
        let posts = self.list_posts().await?;
        let mut result = Vec::with_capacity(posts.len());
        for post in posts {
            let comments = self.comments_for_post(post.id).await?;
            result.push(PostWithComments { post, comments });
        }
        Ok(result)
    }

    /// The fix: one query total. A `LEFT JOIN` fetches every post and all
    /// of its comments in a single round trip; `group_joined_rows` (which
    /// you implement above) turns the flattened rows back into
    /// `PostWithComments`. This is the direct translation of Django's
    /// `Post.objects.prefetch_related("comments")` — same idea, same
    /// query-count win, just written out as raw SQL instead of ORM sugar.
    pub async fn posts_with_comments_batched(&self) -> Result<Vec<PostWithComments>, sqlx::Error> {
        self.query_count.fetch_add(1, Ordering::SeqCst);
        let rows: Vec<(i64, String, Option<i64>, Option<String>)> = sqlx::query_as(
            "SELECT posts.id, posts.title, comments.id, comments.body \
             FROM posts \
             LEFT JOIN comments ON comments.post_id = posts.id \
             ORDER BY posts.id, comments.id",
        )
        .fetch_all(&self.pool)
        .await?;

        let joined = rows
            .into_iter()
            .map(
                |(post_id, post_title, comment_id, comment_body)| JoinedRow {
                    post_id,
                    post_title,
                    comment_id,
                    comment_body,
                },
            )
            .collect();

        Ok(group_joined_rows(joined))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    fn row(post_id: i64, title: &str, comment_id: Option<i64>, body: Option<&str>) -> JoinedRow {
        JoinedRow {
            post_id,
            post_title: title.to_string(),
            comment_id,
            comment_body: body.map(str::to_string),
        }
    }

    #[test]
    fn group_joined_rows_groups_comments_under_their_post_preserving_order() {
        let rows = vec![
            row(1, "First Post", Some(10), Some("nice!")),
            row(1, "First Post", Some(11), Some("agreed")),
            row(2, "Second Post", Some(12), Some("meh")),
        ];

        let grouped = group_joined_rows(rows);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].post.id, 1);
        assert_eq!(grouped[0].comments.len(), 2);
        assert_eq!(grouped[0].comments[0].body, "nice!");
        assert_eq!(grouped[0].comments[1].body, "agreed");
        assert_eq!(grouped[1].post.id, 2);
        assert_eq!(grouped[1].comments.len(), 1);
    }

    /// The whole point of `LEFT JOIN` over a plain `JOIN`: a post with zero
    /// comments still shows up, with an empty `comments` Vec — not missing
    /// entirely from the result.
    #[test]
    fn group_joined_rows_gives_a_commentless_post_an_empty_vec() {
        let rows = vec![row(1, "Lonely Post", None, None)];

        let grouped = group_joined_rows(rows);

        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].post.title, "Lonely Post");
        assert!(grouped[0].comments.is_empty());
    }

    #[test]
    fn group_joined_rows_handles_an_empty_input() {
        assert_eq!(group_joined_rows(Vec::new()), Vec::new());
    }

    /// Needs a real local Postgres. Start one first (see README.md), then:
    ///   DATABASE_URL=postgres://taskforge:taskforge@localhost:5432/taskforge \
    ///     cargo test -p p3-05-01-indexing-explain-analyze-n-plus-1 -- --ignored --test-threads=1
    #[tokio::test]
    #[ignore]
    #[serial(p3_05_01_indexing_db)]
    async fn naive_makes_one_query_per_post_plus_one() {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        let store = BlogStore::connect(&database_url).await.unwrap();
        store.reset().await.unwrap();

        let post_a = store.create_post("First Post").await.unwrap();
        let post_b = store.create_post("Second Post").await.unwrap();
        store.create_comment(post_a, "nice!").await.unwrap();
        store.create_comment(post_a, "agreed").await.unwrap();
        store.create_comment(post_b, "meh").await.unwrap();

        store.reset_query_count();
        let result = store.posts_with_comments_naive().await.unwrap();

        assert_eq!(result.len(), 2);
        // 1 query for `list_posts`, plus 1 more per post (2 posts) = 3.
        assert_eq!(store.query_count(), 3);
    }

    #[tokio::test]
    #[ignore]
    #[serial(p3_05_01_indexing_db)]
    async fn batched_makes_exactly_one_query_regardless_of_post_count() {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        let store = BlogStore::connect(&database_url).await.unwrap();
        store.reset().await.unwrap();

        for i in 0..5 {
            let post_id = store.create_post(&format!("Post {i}")).await.unwrap();
            store.create_comment(post_id, "a comment").await.unwrap();
        }

        store.reset_query_count();
        let result = store.posts_with_comments_batched().await.unwrap();

        assert_eq!(result.len(), 5);
        assert_eq!(store.query_count(), 1);
    }

    #[tokio::test]
    #[ignore]
    #[serial(p3_05_01_indexing_db)]
    async fn naive_and_batched_agree_on_the_result() {
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run this ignored integration test");
        let store = BlogStore::connect(&database_url).await.unwrap();
        store.reset().await.unwrap();

        let post_a = store.create_post("First Post").await.unwrap();
        store.create_comment(post_a, "nice!").await.unwrap();
        store.create_post("Commentless Post").await.unwrap();

        let naive = store.posts_with_comments_naive().await.unwrap();
        let batched = store.posts_with_comments_batched().await.unwrap();

        assert_eq!(naive, batched);
    }
}
