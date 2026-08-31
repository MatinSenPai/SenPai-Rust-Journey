# راه‌حل

```rust
#[async_trait]
impl Greeter for Formal {
    async fn greet(&self, name: &str) -> String {
        format!("Good day, {name}.")
    }
}

pub async fn run_cpu_work_off_the_runtime(n: u64) -> u64 {
    tokio::task::spawn_blocking(move || sum_range(n))
        .await
        .unwrap()
}
```

`Formal::greet` دقیقاً همان مشخصات است، یک صدازدنِ `format!`. `run_cpu_work_off_the_runtime` نقطه‌ی اصلیِ این نیمه‌یِ درس است: `sum_range(n)` یک کلوژرِ کاملاً همگام است — هیچ‌چیزِ داخلش `async` نیست، هیچ‌جایش هیچ‌وقت yield نمی‌کند — پس سپردنِ مستقیمِ آن به `spawn_blocking` همان چیزی است که آن را از رویِ ریسمان‌هایِ کارگرِ همیاری‌طورِ `tokio` بیرون می‌برد و رویِ استخرِ مسدودکننده‌یِ جداگانه می‌گذارد. `spawn_blocking` همان نوعِ `JoinHandle`ای را برمی‌گرداند که `tokio::spawn` برمی‌گرداند، پس `.await`اش به یک `Result<u64, JoinError>` می‌رسد؛ این‌جا `.unwrap()` امن است چون `sum_range` نمی‌تواند پنیک کند. `cpu_work_returns_the_correct_sum` مقدار را چک می‌کند؛ `cpu_work_runs_without_blocking_a_concurrent_sleep` خودِ *نکته* را چک می‌کند — یک `sleep`ِ همزمانِ ۱۰میلی‌ثانیه‌ای طبقِ برنامه تمام می‌شود حتی وقتی یک جمعِ ۵۰میلیونی در حالِ اجراست، چیزی که اگر `sum_range(n)` مستقیم داخلِ همان `async fn` صدا زده می‌شد، هرگز درست نبود.

## بساز: یک `Greeter`ِ دوم، و `#[async_trait]` برایِ `dyn`

```rust
#[async_trait]
pub trait Greeter {
    async fn greet(&self, name: &str) -> String;
}

pub struct Casual;

#[async_trait]
impl Greeter for Casual {
    async fn greet(&self, name: &str) -> String {
        format!("Hey {name}!")
    }
}
```

`Casual` دوقلوی `Formal` است، همان شکل، رشته‌ی متفاوت. کارِ اصلی همان `#[async_trait]` است — رویِ خودِ صفت و رویِ **هر دو** `impl` — یک ماکرو، سه جا — و دقیقاً همین است که `Greeter` را از «`async fn` خام، فقط dispatchِ ایستا» به «قابلِ جعبه‌شدن به‌عنوانِ `dyn Greeter`» تبدیل می‌کند، همان معامله‌ای که بخشِ «مفهوم»ِ این درس نشانت داد: یک تخصیصِ heap به‌ازایِ هر صدازدن، پرداخته‌شده برایِ اینکه `Box<dyn Greeter>` اصلاً کامپایل شود.

```rust
pub async fn greet_all(greeters: &[Box<dyn Greeter>], name: &str) -> Vec<String> {
    let mut results = Vec::with_capacity(greeters.len());
    for greeter in greeters {
        results.push(greeter.greet(name).await);
    }
    results
}
```

`greet_all` همان چیزی است که آن بازسازی برایش بود: یک اسلایس که هم `Formal` هم `Casual` را پشتِ همان `Box<dyn Greeter>` نگه می‌دارد، یکی‌یکی `.await` می‌شود، به همان ترتیب — `greet_all_collects_every_greeter_in_order` هر دو greeter را تویِ همان یک `Vec` قاطی می‌کند و چک می‌کند نتیجه‌ها دقیقاً به همان ترتیب برمی‌گردند.

## چالش: هر جمع را رویِ `spawn_blocking`ِ خودش پخش کن

```rust
pub async fn sum_many_off_the_runtime(ns: Vec<u64>) -> Vec<u64> {
    let handles: Vec<_> = ns
        .into_iter()
        .map(|n| tokio::task::spawn_blocking(move || sum_range(n)))
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}
```

دقیقاً همان شکلِ «اول همه را spawn کن، بعد await کن» که [۲.۹.۱](../01-spawn-joinset-structured-concurrency/README.fa.md) برایِ `tokio::spawn` یادت داد — گذرِ `.map(...).collect()` هر `spawn_blocking` را پیش از هر `.await`ای شلیک می‌کند، پس تا وقتی حلقه‌ی دوم شروع به جمع‌کردن کند، همه‌شان از قبل روی استخرِ مسدودکننده هم‌زمان در حالِ اجرا هستند. `sum_many_matches_individual_sums` مقدارها را چک می‌کند؛ `sum_many_runs_concurrently_not_sequentially` خودِ *نکته* را چک می‌کند — پنج جمعِ ۳۰میلیونی خیلی زودتر از آنچه پنج صدازدنِ پشتِ‌سرِهمِ `sum_range` طول می‌کشید تمام می‌شوند، چون استخرِ مسدودکننده واقعاً آن‌ها را کنارِ هم اجرا می‌کند.
