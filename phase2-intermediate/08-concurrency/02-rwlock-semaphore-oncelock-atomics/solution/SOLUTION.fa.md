# راه‌حل — ۲.۸.۲ `RwLock`، `Semaphore`، `OnceLock`/`LazyLock`، اتمیک‌ها

```rust
impl LazyGreeting {
    pub fn new(name: &str) -> Self {
        LazyGreeting {
            name: name.to_string(),
            cell: OnceLock::new(),
        }
    }

    pub fn greeting(&self) -> &str {
        self.cell.get_or_init(|| format!("hello, {}", self.name))
    }
}

impl HitCounter {
    pub fn new() -> Self {
        HitCounter {
            count: AtomicUsize::new(0),
        }
    }

    pub fn hit(&self) -> usize {
        self.count.fetch_add(1, Ordering::SeqCst) + 1
    }

    pub fn count(&self) -> usize {
        self.count.load(Ordering::SeqCst)
    }
}

impl ResourcePool {
    pub fn new(capacity: usize) -> Self {
        ResourcePool {
            available: AtomicUsize::new(capacity),
        }
    }

    pub fn try_claim(&self) -> bool {
        let mut current = self.available.load(Ordering::SeqCst);
        loop {
            if current == 0 {
                return false;
            }
            match self.available.compare_exchange(
                current,
                current - 1,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(_) => return true,
                Err(actual) => current = actual,
            }
        }
    }

    pub fn release(&self) {
        self.available.fetch_add(1, Ordering::SeqCst);
    }

    pub fn available(&self) -> usize {
        self.available.load(Ordering::SeqCst)
    }
}
```

هیچ‌کدامِ این‌ها چیزی فراتر از آنچه در «مفهوم» دیدی نمی‌خواست — همان سه ابزار، این‌بار رویِ سه نوعِ کوچک و مستقل.

## `LazyGreeting` — یک فیلدِ `OnceLock`، نه یک `static`

هر مثالِ `OnceLock`/`LazyLock` که تا این‌جا دیدی رویِ یک `static` بود، چون یک مقدارِ سراسری فقط یک‌بار در کلِ برنامه ساخته می‌شود، همین. `LazyGreeting` شکلِ رایجِ دیگرش را نشان می‌دهد: یک `OnceLock` که *داخلِ* یک ساختار زندگی می‌کند، پس هر نمونه‌ی `LazyGreeting` مقدارِ تنبلِ خودش را دارد، کاملاً مستقل از هر نمونه‌ی دیگر. تابعِ `greeting()` فقط یک خط است — `self.cell.get_or_init(|| format!("hello, {}", self.name))` — چون کلِ الگو همین است: به `get_or_init` یک کلوژر بده که مقدار را می‌سازد؛ یا آن کلوژر را اجرا می‌کند (اولین صدازدن)، یا نادیده‌اش می‌گیرد و همان چیزی را که از قبل هست پس می‌دهد (هر صدازدنِ بعدی).

تستِ `greeting_is_computed_once_and_then_cached` این را بدونِ نیاز به هیچ شمارنده‌ای اثبات می‌کند: اشاره‌گرِ خامِ پشتِ `&str`ِ برگشتی را بینِ دو صدازدن مقایسه می‌کند. اگر `get_or_init` چیزی را دوباره ساخته بود، `format!`ِ دوم یک `String`ِ *تازه* در یک آدرسِ *متفاوت* تخصیص می‌داد — و تست شکست می‌خورد. برگرداندنِ همان آدرسِ دقیق فقط وقتی ممکن است که صدازدنِ دوم اصلاً چیزی نساخته باشد.

## `HitCounter` — یک `AtomicUsize`، هیچ `Mutex`ی هیچ‌جا

تابعِ `hit()` همان `fetch_add(1, Ordering::SeqCst) + 1` است: خودِ `fetch_add` مقدارِ *پیش از* جمع را برمی‌گرداند، پس `+ 1` همان چیزی است که آن را به «مجموعی که شاملِ همین hit هم می‌شود» تبدیل می‌کند، دقیقاً همان چیزی که مشخصات خواسته بود. `count()` هم فقط یک `load` ساده است. هیچ گاردی نیست، هیچ صبرکردنی نیست، هیچ امکانی نیست که دو ریسمان به‌روزرسانیِ همدیگر را خراب کنند — `fetch_add` یک عملیاتِ سخت‌افزاریِ اتمیکِ تک است، نه یک خواندن که با یک نوشتنِ جدا دنبال شود.

تستِ `hit_counter_survives_many_threads_hitting_it_at_once` هشت ریسمان می‌سازد که هرکدام ۵۰۰ بار `hit()` را صدا می‌زنند، و مطمئن می‌شود شمارشِ نهایی دقیقاً ۴۰۰۰ است. این تستی نیست که «معمولاً» سبز شود — هرکدام از آن ۴۰۰۰ افزایش، به هر ترتیبی، تضمین‌شده جا می‌افتد، بدونِ اینکه هیچ‌کدام گم شود — همان خاصیتی که [مثالِ شمارنده‌ی بدونِ‌قفل](../README.fa.md) نشانت داد.

## `ResourcePool` — همان حلقه‌ی compare-and-swap که `try_claim` در درس داشت

تابعِ `try_claim` اول مقدارِ فعلیِ `available` را می‌خواند، و اگر از قبل صفر نبود، تلاش می‌کند با `compare_exchange` آن را با یک واحد کمتر عوض کند. `Ok(_)` یعنی عوض‌کردن واقعاً اتفاق افتاد — هیچ‌کس دیگری بینِ آن `load` و آن `compare_exchange` مقدار را عوض نکرد، پس این ریسمان واقعاً یک منبع را گرفت. `Err(actual)` یعنی claim یا release یک ریسمانِ دیگر زودتر رسید؛ `current` با همان مقدارِ واقعی به‌روز می‌شود و حلقه دوباره از همان‌جا تلاش می‌کند — نه اینکه تسلیم شود، نه اینکه شمارنده را خراب کند. این دقیقاً همان شکلی است که [مثالِ پرچمِ یک‌باره](../README.fa.md) با `AtomicBool` داشت، فقط تعمیم‌یافته از «هنوز `false` بود؟» به «بیش از صفر بود؟».

تستِ `pool_never_lets_more_than_capacity_claims_stay_out_at_once` بیست ریسمان می‌سازد که همه با هم رویِ یک استخرِ ۲تایی مسابقه می‌دهند، و از *داخلِ* هر ریسمانی که واقعاً یک منبع گرفت، مطمئن می‌شود تعدادِ فعلاً‌گرفته‌شده هیچ‌وقت از ۲ بیشتر نمی‌شود. این تست (یا کلِ مجموعه‌ی تست‌ها) را چند بار پشتِ‌سرِهم اجرا کن، نه فقط یک‌بار — این همان انضباطی است که هر تستِ با ریسمان‌های واقعی می‌خواهد، و دقیقاً همان چیزی است که این assertion را ارزش‌مند می‌کند.

## چیزی که این درس واقعاً درباره‌اش بود

- **یک حلقه‌ی `compare_exchange` یک `Mutex`ِ مبدل نیست — واقعاً بدونِ‌قفل است.** هیچ ریسمانی این‌جا هیچ‌وقت منتظرِ ریسمانِ دیگری نمی‌ماند؛ یک `try_claim()`ِ بازنده فوراً `false` برمی‌گرداند و صداکننده تصمیم می‌گیرد بعدش چه‌کار کند.
- **`OnceLock` فقط برایِ `static`ها نیست.** یک فیلدِ ساختار که در آن پیچیده شده، به *هر نمونه* محاسبه‌ی یک‌بارِ خودش را می‌دهد، کاملاً مستقل از نمونه‌های دیگر — شکلِ `static` که در درس دیدی و این شکلِ نمونه‌ای، همان مسئله را در دو دامنه‌ی متفاوت حل می‌کنند.
- **برابریِ اشاره‌گر راهِ معتبری است برایِ اثباتِ «هیچ کاری انجام نشد».** تستِ `greeting_is_computed_once_and_then_cached` هیچ‌وقت تعدادِ صدازدن‌ها را نمی‌شمارد؛ فقط چک می‌کند صدازدنِ دوم دقیقاً همان حافظه را پس داده، که فقط وقتی ممکن است چیزی دوباره ساخته نشده باشد.
