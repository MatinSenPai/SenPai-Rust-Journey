# پاسخ تشریحی

```rust
pub fn group_joined_rows(rows: Vec<JoinedRow>) -> Vec<PostWithComments> {
    let mut posts: Vec<PostWithComments> = Vec::new();
    let mut index_of: HashMap<i64, usize> = HashMap::new();
    for row in rows {
        let index = *index_of.entry(row.post_id).or_insert_with(|| {
            posts.push(PostWithComments {
                post: Post { id: row.post_id, title: row.post_title.clone() },
                comments: Vec::new(),
            });
            posts.len() - 1
        });
        if let Some(comment_id) = row.comment_id {
            posts[index].comments.push(Comment {
                id: comment_id, post_id: row.post_id,
                body: row.comment_body.unwrap_or_default(),
            });
        }
    }
    posts
}
```

`HashMap<i64, usize>` می‌داند هر post قبلاً در کدام index `posts` ساخته شده است، حتی اگر rowهایش contiguous نباشند. `entry(...).or_insert_with(...)` lookup و ساخت post تازه را یکجا انجام می‌دهد. comment_id برابر None، حالت post بدون comment در LEFT JOIN است و comment اضافه نمی‌کند.

```rust
pub async fn posts_with_comments_naive(&self) -> Result<Vec<PostWithComments>, sqlx::Error> {
    let posts = self.list_posts().await?;
    let mut result = Vec::with_capacity(posts.len());
    for post in posts {
        let comments = self.comments_for_post(post.id).await?;
        result.push(PostWithComments { post, comments });
    }
    Ok(result)
}
```

این N+1 آشکار است: `list_posts` query شماره ۱ و هر iteration یک query کامل دیگر. با ۲ post سه query و با ۱۰هزار، ۱۰٬۰۰۱ query می‌شود. batched method با یک LEFT JOIN فقط یک query می‌زند و grouping را در memory انجام می‌دهد؛ همان trade-off `prefetch_related`.

`AtomicU64` و `fetch_add(1, Ordering::SeqCst)` query count را assertionپذیر می‌کند؛ ordering ساده برای bookkeeping test کافی است. B-tree value مرتب و pointer row را نگه می‌دارد، پس lookup `post_id = 42` به‌جای scan کل table tree را می‌پیماید. indexها write و disk هزینه دارند. runtime API نیز workspace را بدون Postgres هنگام compile سالم نگه می‌دارد؛ macroهای `!` به schema زنده یا offline cache نیاز دارند. همه testهای DB باید tag serial مشترک داشته باشند، چون test untagged می‌تواند وسط `store.reset()` دیگری query بزند.
