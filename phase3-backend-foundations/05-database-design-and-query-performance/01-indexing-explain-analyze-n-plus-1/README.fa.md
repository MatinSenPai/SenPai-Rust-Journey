# ۰۵.۱ — index، `EXPLAIN ANALYZE` و مسئله‌ی N+1

دو ایده با یک ریشه: **database حداقل کاری را می‌کند که گفته‌ای، نه حداقل کاری که منظورت بوده است.** بدون index، «این row را پیدا کن» یعنی همه rowها را نگاه کن. بدون query batch، «postها و commentهایشان را بده» یعنی برای هر post دوباره بپرس. روی ۱۲ row محلی دیده نمی‌شود؛ روی دو میلیون row incident ساعت دو شب است.

## بخش اول: index واقعاً چیست؟

`SELECT * FROM posts WHERE title = 'Frieren'` بدون index روی `title` همه rowها را می‌خواند و filter می‌کند: `Seq Scan` با هزینه‌ی `O(n)`. 

```sql
CREATE INDEX idx_posts_title ON posts(title);
```

یک B-tree جدا می‌سازد که valueهای title را مرتب و همراه pointer row نگه می‌دارد. lookup در tree حدود `O(log n)` است؛ برای دو میلیون row نزدیک ۲۱ comparison در برابر تا دو میلیون scan. اما index رایگان نیست: هر `INSERT`/`UPDATE`/`DELETE` روی column indexed باید tree را هم به‌روز کند و disk می‌گیرد. همه columnها را «برای احتیاط» indexکردن anti-pattern است؛ columnهای واقعی `WHERE`، `JOIN` و `ORDER BY` را index کن.

## بخش دوم: خواندن `EXPLAIN ANALYZE`

`EXPLAIN ANALYZE` query را **واقعاً اجرا** و plan و timing واقعی را نشان می‌دهد؛ `EXPLAIN` تنها plan تخمینی را می‌دهد. روی comments بدون index، دنبال `Seq Scan` و `Rows Removed by Filter: 999952` باش: تقریباً میلیون row برای ۴۸ جواب خوانده شده. پس از index، `Index Scan using idx_comments_post_id` و `Index Cond: (post_id = 42)` می‌بینی.

`cost=X..Y` تخمین planner با واحد غیرمیلی‌ثانیه‌ای است. `actual time=X..Y` و `rows=N` داده‌ی واقعی‌اند. اختلاف شدیدشان اغلب نشان آمار stale است و `ANALYZE <table>;` آن را تازه می‌کند.

## بخش سوم: N+1

```python
posts = Post.objects.all()       # ۱ query
for post in posts:
    print(post.comments.all())   # N query دیگر
```

`prefetch_related` این را batch می‌کند. در sqlx magic ORM وجود ندارد، پس شکل مشکل واضح است: `list_posts()` یک query و `comments_for_post(post.id)` برای هر post یک round trip. با ۱۰هزار post یعنی ۱۰٬۰۰۱ query.

راه درست یک `LEFT JOIN` است که همه `(post, comment)`ها را در یک query می‌آورد؛ سپس `group_joined_rows` rowهای flattened را در Rust به `PostWithComments` بازمی‌گرداند. `LEFT JOIN` لازم است تا post بدون comment با `comment_id`/`comment_body` برابر None حذف نشود.

## چرا API runtime-checked اینجاست؟

`query!`/`query_as!` برای type-check SQL هنگام compile به database یا cache `.sqlx` نیاز دارند. در lessonی که باید بدون infrastructure در کل workspace compile شود، `query`/`query_as` runtime-checked بهتر است؛ فقط test database-touching با `#[ignore]` به Postgres نیاز دارد.

```senpai-visual
{"kind":"database","labels":["posts","N query comment","LEFT JOIN یک‌بار","row تخت","group_joined_rows"]}
```

دفتر تلفن مرتب در برابر رسیدهای به‌هم‌ریخته تشبیه index است؛ اما optimizer ممکن است scan را برای table کوچک انتخاب کند و این همیشه bug نیست. N+1 نیز اشتباه syntax نیست؛ شکل access pattern است.

## تمرین تو

`group_joined_rows` خالص و `posts_with_comments_naive` عمداً N+1 را بنویس. سپس batched implementation آماده را با query count مقایسه کن.
