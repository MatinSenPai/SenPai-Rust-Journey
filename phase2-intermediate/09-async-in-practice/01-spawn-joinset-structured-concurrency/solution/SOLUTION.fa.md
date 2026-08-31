# راه‌حل

```rust
pub async fn fetch_simulated(id: u32, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id}")
}

pub async fn fetch_all_via_joinset(requests: Vec<(u32, u64)>) -> Vec<String> {
    let mut set = JoinSet::new();
    for (id, delay_ms) in requests {
        set.spawn(fetch_simulated(id, delay_ms));
    }

    let mut results = Vec::new();
    while let Some(result) = set.join_next().await {
        results.push(result.unwrap());
    }
    results
}
```

حلقه‌ی spawn و حلقه‌ی جمع‌کردن، دو گذرِ کاملاً جدا هستند — همان حرکتِ ساختاری‌ای که `fetch_all_concurrently`ِ [۲.۸.۶](../../../08-concurrency/06-tokio-basics/README.fa.md) هم داشت: هر درخواست اول به `JoinSet` سپرده می‌شود، پس هر `sleep` تقریباً همان یک لحظه شروع به شمارش می‌کند، پیش از آنکه هیچ جمع‌کردنی شروع شود. فرقش با ۲.۸.۶ کاملاً تویِ حلقه‌ی دوم است — `while let Some(result) = set.join_next().await` هرکدام از تسک‌ها را که واقعاً تمام شده باشد پس می‌کشد، نه آن‌یکی که نوبتش تویِ یک فهرست است — و دقیقاً برایِ همین `joinset_returns_results_in_completion_order` برایِ تأخیرهایِ `[80, 10, 50]`، خروجیِ `["item-2", "item-3", "item-1"]` را می‌بیند — به ترتیبِ تأخیر، نه به ترتیبی که `requests` آن‌ها را فهرست کرده بود.

`result.unwrap()`: `.join_next()` یک `Option<Result<T, JoinError>>` پس می‌دهد؛ خودِ `Option` را همان `while let` تمام می‌کند، و این `.unwrap()` فرض می‌کند تسک پنیک نکرده — فرضی معقول برایِ همین تابع، چون هیچ‌جایِ `fetch_simulated` پنیک نمی‌کند.

## بساز: شمارشِ موفقیت‌ها و پنیک‌ها

```rust
pub async fn count_task_outcomes(ids: Vec<u32>, panics_at: Vec<u32>) -> (usize, usize) {
    let mut set = JoinSet::new();
    for id in ids {
        let should_panic = panics_at.contains(&id);
        set.spawn(async move {
            if should_panic {
                panic!("task {id} was told to panic");
            }
        });
    }

    let mut successes = 0;
    let mut panics = 0;
    while let Some(result) = set.join_next().await {
        match result {
            Ok(()) => successes += 1,
            Err(_) => panics += 1,
        }
    }
    (successes, panics)
}
```

`should_panic` پیش از خودِ بلوکِ `async move` محاسبه می‌شود، از رویِ همان `Vec<u32>`ِ عادی — `panics_at.contains(&id)` به `&panics_at` نیاز دارد، و خودِ تسک فقط همان یک `bool`ی را که گرفته لازم دارد، نه کلِ فهرست را. شمارش اصلاً کاری ندارد `join_next` تسک‌ها را به چه ترتیبی پس می‌دهد، فقط چندتا از هر نوع می‌رسند مهم است — پس هم `counts_successes_and_panics_separately` و هم `counts_all_panics`، بدونِ توجه به زمان‌بندی، سبز می‌مانند.

## چالش: اولین موفقیت برنده است، بقیه لغو می‌شوند

```rust
pub async fn first_ok_of(ids: Vec<u32>, delay_ms: u64, fail_ids: Vec<u32>) -> Option<String> {
    let mut set = JoinSet::new();
    for id in ids {
        let fails = fail_ids.contains(&id);
        set.spawn(async move {
            if fails {
                panic!("task {id} was told to fail");
            }
            fetch_simulated(id, delay_ms).await
        });
    }

    while let Some(result) = set.join_next().await {
        if let Ok(value) = result {
            set.abort_all();
            return Some(value);
        }
    }
    None
}
```

هر `Err`ی که از `join_next` بیاید، همان حلقه‌ی `while let` بی‌سروصدا ردش می‌کند — همین که یک `Ok(value)` ظاهر شد، `set.abort_all()` هرچه هنوز در حالِ اجراست را لغو می‌کند (همان هم‌روندیِ ساخت‌یافته، این‌بار عمداً به دستِ خودمان راه افتاده، نه با بسته‌شدنِ یک دامنه) و تابع بلافاصله، بدونِ صبر برایِ بقیه، برمی‌گردد. `first_ok_of_skips_the_failing_ids` idهایِ `[1, 2, 3]` را می‌دهد با `[1, 3]` شکست‌خورده، پس فقط تسکِ ۲ می‌تواند یک `Ok` بدهد — تست لازم نیست ترتیبِ تمام‌شدن را بداند تا مطمئن باشد جواب باید `"item-2"` باشد. `first_ok_of_returns_none_when_everything_panics` سرِ دیگرِ ماجراست: همه‌یِ idها شکست می‌خورند، `join_next` فقط `Err` پس می‌دهد، و حلقه بدونِ هیچ چیزی برایِ برگرداندن تمام می‌شود.
