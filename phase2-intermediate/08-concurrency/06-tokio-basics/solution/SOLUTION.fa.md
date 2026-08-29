# راه‌حل

```rust
pub async fn fetch_simulated(id: u32, delay_ms: u64) -> String {
    sleep(Duration::from_millis(delay_ms)).await;
    format!("item-{id}")
}

pub async fn fetch_all_concurrently(ids: Vec<u32>, delay_ms: u64) -> Vec<String> {
    let handles: Vec<_> = ids
        .into_iter()
        .map(|id| tokio::spawn(fetch_simulated(id, delay_ms)))
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}
```

تکه‌ی ساختاری‌ای که همه‌چیز بهش بستگی دارد همین است: **اول همه‌ی تسک‌ها را، در یک گذرِ واحد، spawn کن** (همان `.map(...).collect()`)، و *بعدش*، در یک حلقه‌ی مجزا، تک‌تکِ دستگیره‌ها را `.await` کن. اگر به‌جایش می‌نوشتی `tokio::spawn(fetch_simulated(id, delay_ms)).await` تویِ همان یک حلقه، هر spawn همان لحظه await می‌شد و پیش از شروعِ بعدی کامل تمام می‌شد — یعنی از نظرِ عملکردی دوباره کاملاً ترتیبی، فقط با سربارِ بی‌فایده‌ی spawn کردن. Spawn کردنِ همه‌شان از قبل دقیقاً همان چیزی است که به هر `sleep` اجازه می‌دهد هم‌زمان با بقیه بشمارد — تا وقتی اولین `.await` در حلقه‌ی دوم شروع به صبر کردن می‌کند، هر پنج تسک از قبل، کاملاً مستقل از هم، دارند رویِ تایمرِ خودشان می‌شمارند.

`handle.await.unwrap()`: `.await` رویِ دستگیره‌ای که `tokio::spawn` برمی‌گرداند، به یک `Result` می‌رسد که فقط وقتی `Err` است که تسکِ spawn‌شده پنیک کرده باشد (یا لغو شده باشد) — همان شکل و همان دلیل که `thread::spawn` در [۲.۸.۱](../../01-threads-mutex-arc/README.fa.md) با `.join()` به‌ات می‌داد، فقط این‌بار async است، نه مسدودکننده (blocking).

## سؤالِ گرم‌کردنِ اول، دوباره

اگر بازنویسیِ ترتیبی را امتحان کنی — `for id in ids { results.push(fetch_simulated(id, delay_ms).await); }`، بدونِ هیچ `tokio::spawn`ی — همچنان کامپایل می‌شود و همچنان سرتاسرش `async`/`.await` دارد، ولی `concurrent_fetches_are_actually_concurrent` رویِ این نسخه fail می‌کند: به‌جایِ زیرِ ۱۵۰ میلی‌ثانیه، حدودِ ۲۰۰ میلی‌ثانیه طول می‌کشد. این تیزترین نکته‌ی همین درس است: **نوشتنِ `async fn` و استفاده از `.await`، به‌تنهایی، هیچ‌چیزی را همروند نمی‌کند.** همروندی مشخصاً از `tokio::spawn` می‌آید (یا ترکیب‌گرهایی مثلِ `tokio::join!` که پایین‌تر می‌بینی) — چیزی که چند `Future` را دستِ محیط اجرا می‌دهد تا رویِشان مستقل پیش برود. `.await` به‌تنهایی فقط یعنی «همین‌جا صبر کن تا این یکی تمام شود» — همان‌طور که همین بالا دیدی، این کاملاً می‌تواند کدی کاملاً ترتیبی را هم توصیف کند.

## «بساز»: تأخیرهایِ جداگانه

```rust
pub async fn fetch_with_custom_delays(requests: Vec<(u32, u64)>) -> Vec<String> {
    let handles: Vec<_> = requests
        .into_iter()
        .map(|(id, delay_ms)| tokio::spawn(fetch_simulated(id, delay_ms)))
        .collect();

    let mut results = Vec::with_capacity(handles.len());
    for handle in handles {
        results.push(handle.await.unwrap());
    }
    results
}
```

دقیقاً همان الگو، فقط `.map` این‌بار یک تاپل را دستخوش می‌کند تا هر درخواست تأخیرِ خودش را با خودش ببرد. تستِ `custom_delays_run_concurrently_not_sequentially` همین را با عدد ثابت می‌کند: سه درخواستِ ۸۰، ۸۰ و ۲۴۰ میلی‌ثانیه‌ای، اگر ترتیبی بودند، رویِ هم ۴۰۰ میلی‌ثانیه می‌شدند؛ هم‌زمان، فقط به‌اندازه‌ی کندترین‌شان — حدودِ ۲۴۰ — طول می‌کشند.

## «چالش»: همان کار، بدونِ هیچ spawnای

```rust
pub async fn fetch_two_with_join(id_a: u32, id_b: u32, delay_ms: u64) -> (String, String) {
    tokio::join!(
        fetch_simulated(id_a, delay_ms),
        fetch_simulated(id_b, delay_ms)
    )
}
```

زمان‌بندیِ نتیجه دقیقاً همان است — تستِ `join_fetches_both_concurrently` همین را ثابت می‌کند. ولی مکانیزمِ زیرینش کاملاً فرق دارد. `tokio::join!` هیچ تسکِ تازه‌ای spawn نمی‌کند؛ هر دو `Future` را دستِ محیط اجرا نمی‌دهد — هر دو همچنان تویِ همان یک تسکِ فراخواننده می‌مانند، و `join!` بینِ نقاطِ `.await`شان دستی تناوب می‌کند: یکی را تا رسیدن به اولین نقطه‌ی معطلی پیش می‌برد، بعد سراغِ دیگری می‌رود، و همین‌طور تا هر دو تمام شوند. دقیقاً همین است که توضیح می‌دهد چرا این نسخه هیچ‌وقت به `Send + 'static` نیاز ندارد ([۲.۸.۴](../../04-send-and-sync/README.fa.md)): آن قاعده فقط برایِ چیزی لازم است که ممکن است محیط اجرا بینِ ریسمان‌هایِ کارگر جابه‌جایش کند — و چیزی که هیچ‌وقت به محیط اجرا سپرده نمی‌شود، هیچ‌وقت جابه‌جا هم نمی‌شود.
