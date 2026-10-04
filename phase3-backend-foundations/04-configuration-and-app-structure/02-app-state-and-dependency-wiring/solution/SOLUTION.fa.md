# راه‌حل — ۳.۴.۲ وضعیتِ اپلیکیشن و سیم‌کشیِ وابستگی‌ها

کدِ کامل `solution/src/lib.rs` است؛ همه‌ی تست‌هایِ `solution/tests/` را پاس می‌کند، از جمله `build_test.rs` برایِ پله‌ی «بساز».

## تعمیر

- `04`: `#[derive(Clone)]` رویِ `AppState`.
- `05`: `trait Clock: Send + Sync`، تا هر پیاده‌ساز در `impl`ِ خودش وارسی شود.
- `06`: `#[derive(Clone, FromRef)]` رویِ `AppState` (و import کردنِ `axum::extract::FromRef`). هندلر همان‌طور که بود می‌ماند.
- `07`: یک newtype برایِ هر شمارنده (`struct Hits(Arc<AtomicU32>)` و `struct Misses(Arc<AtomicU32>)`) و هندلرِ `hits` یک `State<Hits>` می‌گیرد.

## ساعت‌ها

```rust
impl Clock for SystemClock {
    fn now(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}
```

`duration_since` فقط وقتی شکست می‌خورد که ساعتِ سیستم قبل از ۱۹۷۰ باشد، و مشخصات می‌گوید جوابِ آن `0` است، پس خطا به‌جایِ `unwrap` کنار گذاشته می‌شود.

```rust
impl FakeClock {
    pub fn at(secs: u64) -> Self { Self { secs: AtomicU64::new(secs) } }
    pub fn advance(&self, secs: u64) { self.secs.fetch_add(secs, Ordering::SeqCst); }
}
impl Clock for FakeClock {
    fn now(&self) -> u64 { self.secs.load(Ordering::SeqCst) }
}
```

atomic تغییرپذیریِ درونی است که قفل نمی‌خواهد، پس `advance` می‌تواند `&self` بگیرد. کلِ هدفِ طرح همین است: تست یک `Arc<FakeClock>` نگه می‌دارد، اپ یک کلون از آن دارد، و `advance` رویِ یکی در دیگری دیده می‌شود (`every_holder_of_the_same_fake_clock_sees_the_same_time`). `Cell` جواب نمی‌داد: `Sync` نیست، پس فیک شرطِ `Clock: Send + Sync` را برآورده نمی‌کرد.

## ذخیره‌گاه

```rust
pub fn add(&self, title: &str, at: u64) -> Watch {
    let mut entries = self.entries.lock().unwrap();
    let watch = Watch { id: entries.len() as u64 + 1, title: title.to_string(), watched_at: at };
    entries.push(watch.clone());
    watch
}
```

شناسه از طول گرفته می‌شود، زیرِ همان قفلی که `push` را می‌کند، پس دو `add` هم‌زمان نمی‌توانند یک شناسه بگیرند. این فقط به این دلیل ایمن است که ورودی‌ها هرگز حذف نمی‌شوند؛ با یک `DELETE` به شمارنده‌ی جدا نیاز داری، مثلِ ذخیره‌گاهِ ۳.۲.۳. `since` با `watched_at >= cutoff` فیلتر می‌کند (مرز شامل است، `since_keeps_entries_at_or_after_the_cutoff`) و ترتیبِ درج را نگه می‌دارد، که همان ترتیبِ زمان نیست (`since_keeps_the_order_entries_were_added_in`). `count` همان `lock().unwrap().len()` است.

## وضعیت و هندلرها

```rust
impl AppState {
    pub fn new(clock: Arc<dyn Clock>, config: Config) -> Self {
        Self { clock, store: Arc::new(WatchStore::default()), config }
    }
}
```

`new` ذخیره‌گاه را خودش می‌سازد: صدازننده ساعت و تنظیمات را انتخاب می‌کند، که چیزهایی‌اند که بینِ `main` و یک تست فرق می‌کنند.

```rust
let title = input.title.trim();
if title.is_empty() || title.chars().count() > config.max_title_len {
    return Err((StatusCode::UNPROCESSABLE_ENTITY, "invalid title"));
}
Ok((StatusCode::CREATED, Json(store.add(title, clock.now()))))
```

فاصله‌های دورِ عنوان اول پاک می‌شود و نسخه‌ی *پاک‌شده* ذخیره می‌شود (`the_title_is_trimmed_before_it_is_stored`). حد بر حسبِ نویسه شمرده می‌شود، نه بایت: `title.len()` عنوانِ `"é é é"` را با `max_title_len = 5` ردّ می‌کرد (`the_title_limit_counts_characters_not_bytes`). شکست `422` است، نه `400`: JSON سالم بود و محتوا یک قاعده را شکست (۳.۱.۳). هندلر هرگز به `AppState` دست نمی‌زند؛ سه `State` می‌گیرد، که هرکدام را `FromRef`ِ derive‌شده تحویل می‌دهد.

```rust
let cutoff = clock.now().saturating_sub(config.recent_window_secs);
Json(store.since(cutoff))
```

`saturating_sub` تنها تله است: ساعتِ فیکِ `10` با پنجره‌ی `3600` یک `u64` را زیرِ صفر می‌برد (در بیلدِ debug پنیک، در release یک نقطه‌ی برشِ عظیم که همه‌چیز را پنهان می‌کند). اشباع‌شدن به `0` یعنی «همه‌چیز حساب می‌شود» (`a_clock_smaller_than_the_window_does_not_underflow`). چون برش `now - window` است و فیلتر `>=`، ورودیِ دقیقاً یک پنجره‌یِ قدیمی هنوز حساب می‌شود (`an_entry_exactly_one_window_old_still_counts`).

```rust
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/watch", post(add_watch))
        .route("/watch/recent", get(recent_watches))
        .with_state(state)
}
```

متدی که کسی ثبتش نکرده، مثلِ `DELETE /watch`، خودش `405` جواب می‌دهد.

## بساز: وابستگیِ دوم

```rust
pub trait Notifier: Send + Sync {
    fn notify(&self, message: &str);
}

pub fn with_notifier(mut self, notifier: Arc<dyn Notifier>) -> Self {
    self.notifier = notifier;
    self
}
```

`AppState::new` فیلدِ تازه را با `NullNotifier` پر می‌کند، پس هر تستی که پیش از پله‌ی «بساز» نوشته شده کامپایل می‌ماند، و `with_notifier` آن را عوض می‌کند. فیلد یک `Arc<dyn ...>` دیگر است، پس derive بی‌هیچ زحمتی `State<Arc<dyn Notifier>>` را به هندلرها می‌دهد: هیچ سیم‌کشیِ دیگری عوض نمی‌شود. `add_watch` بعد از فراخوانیِ ذخیره‌گاه و فقط در مسیرِ موفق `notifier.notify(&format!("watched: {}", watch.title))` را صدا می‌زند، پس عنوانِ ردشده (که پیش از رسیدن به آن برمی‌گردد) کسی را خبر نمی‌کند. بدلِ تست یک `Mutex<Vec<String>>` پشتِ صفت است: در اصطلاحاتِ واژه‌نامه یک جاسوس (spy).

## چالش: ساعتِ جنریک

یک جوابِ یگانه نیست، ولی انتظارِ این یافته‌ها را داشته باش. `AppState<C>` هر جا به کار برود `C: Clock` می‌خواهد: `app<C>`، هر هندلر، و هر اکسترکتوری که وضعیت را نام ببرد. `#[derive(FromRef)]` رویش اصلاً کار نمی‌کند (ماکرو می‌گوید «`#[derive(FromRef)]` doesn't support generics»)، پس هر `impl<C: Clock> FromRef<AppState<C>> for ...` را خودت می‌نویسی. هندلرهایی که `State<Arc<C>>` می‌گیرند هم باید جنریک باشند. `#[derive(Clone)]` رویِ `AppState<C>` کرانِ `C: Clone` اضافه می‌کند که `Arc<C>` هرگز لازمش نداشت، پس `Clone` را یا کران می‌زنی یا دستی پیاده می‌کنی. سودش این است که `now()` یک فراخوانیِ مستقیم و inline‌شدنی می‌شود؛ برایِ ساعتی که در هر درخواست یک‌بار خوانده می‌شود این چیزِ قابلِ اندازه‌گیری‌ای نمی‌خرد، که دلیلِ نگه‌داشتنِ `Arc<dyn Clock>` در اینجاست.
