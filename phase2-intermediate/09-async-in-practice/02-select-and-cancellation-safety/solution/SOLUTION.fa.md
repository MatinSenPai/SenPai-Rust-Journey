# راه‌حل

```rust
async fn fetch(delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("data after {delay_ms}ms")
}

pub async fn fetch_with_deadline(delay_ms: u64, budget_ms: u64) -> Option<String> {
    tokio::select! {
        data = fetch(delay_ms) => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    }
}

pub async fn run_until_cancelled(token: CancellationToken, tick_ms: u64) -> u32 {
    let mut ticks = 0;
    loop {
        tokio::select! {
            _ = token.cancelled() => return ticks,
            _ = sleep(Duration::from_millis(tick_ms)) => ticks += 1,
        }
    }
}
```

`fetch_with_deadline` دقیقاً همان الگویی است که «مفهوم» قدم‌به‌قدم ساخت — یک شاخه fetch را می‌گیرد، شاخه‌ی دیگر نقشِ مهلت را بازی می‌کند، و برنده تعیین می‌کند نتیجه `Some` است یا `None`.

`run_until_cancelled` نکته‌ی ظریف‌تری دارد: چرا `return ticks` به‌جایِ `break`؟ چون این `select!` داخلِ یک `loop` نشسته که خودش هیچ مقداری برنمی‌گرداند — یک `break` ساده فقط از حلقه بیرون می‌زد، نه از تابع. `return ticks` هم از حلقه بیرون می‌زند هم مقدار را از خودِ تابع برمی‌گرداند، در یک حرکت. نکته‌ی دومی که تستِ `ticks_stop_after_cancellation` رویش تکیه دارد: یک tick فقط وقتی شمرده می‌شود که شاخه‌ی `sleep` واقعاً برنده شود — اگر توکن دقیقاً وسطِ یک `sleep` لغو شود، آن `sleep` بدونِ اینکه `ticks` را بالا ببرد drop می‌شود، چون شاخه‌ی `cancelled` برنده شده، نه شاخه‌ی tick.

## «بساز»: تلاشِ دوباره تا لغو یا موفقیت

```rust
pub async fn fetch_with_retries(
    token: CancellationToken,
    delay_ms: u64,
    budget_ms: u64,
    max_attempts: u32,
) -> Option<String> {
    for _ in 0..max_attempts {
        tokio::select! {
            _ = token.cancelled() => return None,
            result = fetch_with_deadline(delay_ms, budget_ms) => {
                if let Some(data) = result {
                    return Some(data);
                }
            }
        }
    }
    None
}
```

اینجا خودِ `fetch_with_deadline` — که خودش از قبل یک `select!` است — به‌عنوانِ یک شاخه‌ی `select!`ِ بیرونی‌تر نشسته. این کاملاً قانونی است: هر Future را می‌شود شاخه گذاشت، مهم نیست خودش از چند لایه‌ی `select!` تشکیل شده باشد. اگر `token` وسطِ یک تلاش لغو شود، شاخه‌ی `token.cancelled()` برنده می‌شود و کلِ `fetch_with_deadline`ِ در حالِ اجرا — با هر دو شاخه‌ی داخلی‌اش — همان لحظه drop می‌شود؛ `fetch_with_retries` بی‌درنگ `None` برمی‌گرداند، حتی وسطِ یک تلاش.

## «چالش»: اولین تسکی که تمام شود

```rust
pub async fn fetch_first_of(delays: Vec<u64>, budget_ms: u64) -> Option<String> {
    let mut set = JoinSet::new();
    for delay_ms in delays {
        set.spawn(fetch(delay_ms));
    }

    tokio::select! {
        Some(Ok(data)) = set.join_next() => Some(data),
        _ = sleep(Duration::from_millis(budget_ms)) => None,
    }
}
```

هر تأخیر یک تسکِ کاملاً مستقل می‌شود — همه‌شان از همان اول، هم‌زمان، دارند می‌شمارند. شاخه‌ی اولِ `select!` یک الگوی ردشدنی است: `Some(Ok(data)) = set.join_next()`. `set.join_next()` خودش یک متدِ async است که هر بار *یک* تسکِ تمام‌شده را برمی‌گرداند — پس همین یک شاخه، هر بار که `select!` دوباره poll می‌کند، دنبالِ اولین تسکِ آماده می‌گردد؛ همان تسکی که از همه‌ی تأخیرها کوتاه‌تر بود، همیشه اول می‌رسد. اگر هیچ تسکی پیش از `budget_ms` تمام نشود، شاخه‌ی `sleep` می‌برد و `None` برمی‌گردد — بقیه‌ی تسک‌های هنوز-در-حالِ-اجرا همان‌جا، وسطِ کار، رها می‌شوند (دقیقاً همان الگویِ ایمنیِ لغوی که کلِ این درس رویش ساخته شده، این‌بار برایِ خودِ تسک‌ها، نه یک Futureِ تنها).
