# راه‌حل

```rust
async fn create_widget(
    State(state): State<AppState>,
    Json(req): Json<CreateWidgetRequest>,
) -> (StatusCode, Json<Widget>) {
    let start = Instant::now();

    let id = state.next_id.fetch_add(1, Ordering::SeqCst);
    let widget = Widget { id, name: req.name };
    state.widgets.lock().unwrap().insert(id, widget.clone());

    metrics::counter!("widgets_created_total").increment(1);
    metrics::histogram!("widget_create_duration_seconds").record(start.elapsed().as_secs_f64());

    (StatusCode::CREATED, Json(widget))
}

async fn get_widget(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<Json<Widget>, StatusCode> {
    let start = Instant::now();

    let found = state.widgets.lock().unwrap().get(&id).cloned();

    metrics::histogram!("widget_lookup_duration_seconds").record(start.elapsed().as_secs_f64());

    match found {
        Some(widget) => {
            metrics::counter!("widget_lookups_total", "result" => "hit").increment(1);
            Ok(Json(widget))
        }
        None => {
            metrics::counter!("widget_lookups_total", "result" => "miss").increment(1);
            Err(StatusCode::NOT_FOUND)
        }
    }
}
```

## شمارنده‌ها (Counters) در برابر هیستوگرام‌ها (histograms)، در عمل و واقعیت

متغیرهای `widgets_created_total` و `widget_lookups_total` هر دوتاشون شمارنده (counters) هستن — اعدادی که فقط رو به بالا می‌رن، و جوابگویِ سؤالِ "کلاً چند تا" هستن. اما `widget_create_duration_seconds` و `widget_lookup_duration_seconds` هیستوگرامن — کریتِ `metrics-exporter-prometheus` به جای اینکه فقط یه عدد خشک‌وخالی نگه داره، تک‌تکِ مقادیری که ثبت شده (تو اینجا، همون مدت زمانِ اجرای هندلر تو دنیای واقعی) رو تو سطل‌های مختلف (buckets) دسته‌بندی می‌کنه تا بعداً یه کوئری از سمت Prometheus بتونه سؤال کنه "تاخیر p99مون چقدره"، نه فقط اینکه بپرسه "میانگین چقدره" — چون میانگین همیشه اون داده‌های پرتِ کُند (slow outliers) رو قایم می‌کنه، در حالی که توزیعِ سطلی (bucket distribution) تو یه هیستوگرام دستِ همشونو رو می‌کنه. متدِ `get_widget` هیستوگرامش رو کاملاً عامدانه *قبل از* اینکه بیفته تو شاخه‌هایِ (branch) پیدا-شد/پیدا-نشد ثبت می‌کنه: چون اندازه‌گیری زمانِ تاخیر (latency) واسه هر دو نتیجه کاملاً ارزش داره و مهمه، اما اون کانتر یا شمارنده‌ای که اون زیره به یه برچسبِ `result` نیاز داره تا بتونه این دو تا حالت رو از هم تفکیک کنه، چون سؤالِ "چند تا جستجو کلاً اتفاق افتاده" با سؤالِ "چند تا از اون جستجوها با موفقیت همراه بودن (hits)" دو تا سؤال کاملاً متفاوتن که یه داشبورد واسه جواب دادن بهشون نیاز داره اینا رو جدا از هم بدونه.

## چرا تک‌تک تست‌ها می‌تونن با خیال راحت تابع `()build_router` رو صدا بزنن

تابعِ `()PrometheusBuilder::install_recorder` میاد یه رکوردرِ (recorder) *جهانی (global)* رو برای کُلِ پروسه تنظیم و سِت می‌کنه — و از اون به بعد ماکروهایِ کریتِ `metrics` (مثل `!metrics::counter` و بقیه) دیتای خودشونو می‌نویسن رو هر رکوردری که به عنوان آخرین رکوردر تو کل سیستم نصب شده بوده، نه اینکه روی چیزی محدود به یه `Router`ِ خاص یا یه تست خاص کار کنن. اگه یه بار دیگه بیای `()install_recorder` رو صدا بزنی درجا پنیک (panics) می‌کنه. ازونجایی که تست‌های این درس هرکدومشون به طور کاملاً مستقل میان تابعِ `()build_router` رو واسه خودشون صدا می‌زنن (چون هر تست واسه خودش یه `HashMap` تازه و ترتمیز واسه ویجت‌ها می‌خواد)، اگه یه `()build_router`ِ خام و ساده‌لوحانه می‌نوشتیم که خودش مستقیم `()install_recorder` رو صدا می‌زد، روی دومین تست کلاً برنامه‌رو پنیک و متوقف می‌کرد. راه‌حلِ این مشکل — که دقیقاً کلمه‌به‌کلمه از `capstone-taskforge/taskforge-api` کپی‌پیست شده، چون موقع ساختِ اون کریت هم دقیقاً تو همین تله افتاده بودیم — استفاده از ساختارِ `OnceLock` هستش: تابعِ `()metrics_handle` رکوردر رو رویِ *اولین* فراخوانی‌ای که تو کُلِ پروسه اتفاق می‌افته نصب می‌کنه، و بعد از اون روی هر فراخوانی، خیلی راحت یه `.clone()`ِ کاملاً سبک و بی‌هزینه از همون دستگیره‌ی اولیه (handle) برمی‌گردونه، در نتیجه هر تعداد از فراخوانی‌هایِ `()build_router` که تو همون یه باینری تست باشن، همه‌شون با امنیت کامل کلاً به یه رکوردر زیربناییِ مشترک دسترسی پیدا می‌کنن.

## خروجیِ `/metrics` واقعاً چه شکلیه

دستورِ `()state.metrics_handle.render` میاد و همون فرمتِ متنی و نمایشیِ (text exposition format) مختصِ Prometheus رو تولید می‌کنه — یعنی متن‌های ساده (plain text)، که به ازای هر متریک یه بلوک دارن، مثلاً اینطوری:

```
# TYPE widgets_created_total counter
widgets_created_total 3
# TYPE widget_lookups_total counter
widget_lookups_total{result="hit"} 2
widget_lookups_total{result="miss"} 1
```

این یه فرمت سفارشی و کاستوم (custom format) نیست که ما فقط واسه این درس اختراعش کرده باشیم — این دقیقاً و مو به مو همون بدنه‌یِ (body) پاسخی هستش که وقتی یه سرور واقعی Prometheus با وظیفه‌ی اسکرپ (scrape job) میاد تو دوره‌های زمانی مشخص نقطه‌پایانیِ `/metrics` تو سرور رو هدف می‌گیره و پُول (polls) می‌کنه، دریافتش می‌کنه و پارس می‌کنه. تستِ `metrics_endpoint_exposes_the_expected_counters_and_histograms` که میاد رو پیدا شدنِ یه سری زیررشته‌ها (substrings) تو بدنه‌ی پاسخ (با `(...)body.contains`) شرط (assert) می‌ذاره، در واقع داره دقیقاً همون چیزی رو چک می‌کنه که یه استقرارِ (deployment) واقعی از Prometheus تو همون لحظه می‌تونه بکشه بیرون (scrape) و تبدیلش کنه به نمودار.

## لاگ‌ها در برابر متریک‌ها، به صورت کاملاً ملموس تو همین درس

بیا فراخوانی‌های `!metrics::counter`/`!histogram` تو این درس رو مقایسه کن با فراخوانی‌های `!tracing::info` تو درس قبلی: یه رویدادِ (event) مربوط به `tracing` دقیقاً یک بار به ازای هر بار وقوع شلیک (fires) می‌شه و با خودش کُلی کانتکست و جزئیاتِ کاملاً مختصِ به همون درخواست رو حمل می‌کنه (مثلاً میگه دقیقاً کدوم آیدی سفارش، کدوم یوزر) — که واسه سؤال "سر این یه دونه درخواست چه اتفاقی افتاد" عالی و بی‌نظیره، اما اگه بخوای رو میلیون‌ها رویداد به صورت تجمیعی (aggregate) کوئری بزنی به شدت کُند و پرهزینه درمیاد. از طرف دیگه یه شمارنده/هیستوگرام از جنسِ `metrics` میاد تک‌تکِ اون وقوع‌ها و اتفاقات رو تو یه مجموعِ ادامه‌دار و در-حال-اجرا (running aggregate) کُلَپس (collapses) و خلاصه‌سازی می‌کنه (یعنی تبدیلش می‌کنه به یه عدد کل، یا یه توزیع تو سطل‌های مختلف) — که خب واسه جواب دادن به "سر این یه دونه درخواست چه اتفاقی افتاد" کلاً افتضاح و بی‌مصرفه (چون دیگه حتی یه دونه جزئیات اختصاصی هم از اون درخواست توش باقی نمونده)، اما دقیقاً و دقیقاً همون چیزیه که واسه جواب دادن به "چند تا درخواست تو ثانیه داریم"، "تاخیر p99 تو این یک ساعت چقدر بوده"، و کلاً اون دست سؤالاتی که یه داشبورد یا یه سیستم هشدارِ (alert threshold) نیازمندشه تا حتی زیرِ فشارِ یه حجمِ وحشتناک از درخواست هم بتونه خیلی سریع و ارزون بهت جواب بده، ساخته شده. سیستم‌ها و سرویس‌های پروداکشنِ تو دنیای واقعی دقیقاً به همین دلیل هر دوتایِ اینا رو، کنارِ هم و دوش‌به‌دوش هم ران می‌کنن — هیچ‌کدومشون جایِ اون یکی رو نمی‌گیره.