# ۳.۴.۲ — وضعیتِ اپلیکیشن و سیم‌کشیِ وابستگی‌ها

## در یک نگاه

بعد از این درس می‌توانی:

- یک `AppState` بسازی که کلون‌کردنش ارزان است چون هر چیزِ مشترک پشتِ یک `Arc` است، و توضیح بدهی چرا کلون داده را کپی نمی‌کند و به همان داده اشاره می‌کند.
- با `#[derive(FromRef)]` و `State<Piece>` به هر هندلر فقط همان تکه‌ای از وضعیت را بدهی که لازم دارد، و خطاهایی را که از سیم‌کشیِ غلط می‌آید بخوانی.
- یک وابستگی مثلِ ساعت را پشتِ یک صفت (trait) بگذاری، بینِ نوعِ مشخص، جنریک و `Arc<dyn Trait>` انتخاب کنی، و کلِ اپ را با یک ساعتِ فیک از راهِ `oneshot` تست کنی.
- با دلیل تصمیم بگیری که یک وابستگیِ مشخص اصلاً سزاوارِ صفت هست یا نه.

**زمان:** حدود ۱۰۰ دقیقه · **پیش‌نیاز:**
[۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md)،
[۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)،
[۲.۳.۷ — ارسالِ ایستا در برابرِ ارسالِ پویا](../../../phase2-intermediate/03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md)،
[۲.۸.۴ — `Send` و `Sync`](../../../phase2-intermediate/08-concurrency/04-send-and-sync/README.fa.md)

---

## چرا اهمیت دارد

هر سرویس چیزهایی دارد که هندلرهایِ زیادی لازمشان دارند و هیچ‌کس نباید برایِ هر درخواست از نو بسازدشان: ذخیره‌گاه، تنظیمات، ساعت، و بعداً استخرِ اتصال به دیتابیس و یک ارسال‌کننده‌ی ایمیل. ۳.۲.۱ یک `Arc<Mutex<..>>` را از راهِ `State` رد کرد، و ۳.۲.۳ یک ذخیره‌گاه را. این برایِ یک وابستگی کافی است. وقتی به پنج وابستگی رسیدی، یک قاعده لازم داری: کجا زندگی می‌کنند، هندلر چطور یکی را می‌خواهد، و تست چطور نمونه‌ی واقعی را با یک فیک عوض می‌کند.

در Django هیچ‌وقت مجبور نیستی تصمیم بگیری. ویو فقط می‌نویسد `from django.conf import settings` یا `timezone.now()` را صدا می‌زند: وابستگی یک import در سطحِ ماژول است که با نام پیدا می‌شود. این راحت است، و کار می‌کند چون پایتون اجازه می‌دهد تست بعداً نام را عوض کند (`mock.patch("app.views.timezone")` یا `freezegun`). Rust چنین درِ پشتی‌ای ندارد. تابعی که `SystemTime::now()` را صدا بزند، در هر تست و برای همیشه همان را صدا می‌زند، و هیچ تستی نمی‌تواند بگوید «وانمود کن سال ۲۰۳۰ است». اگر می‌خواهی زمان را کنترل کنی، ساعت باید *از راهِ ورودی‌هایِ تابع* برسد. «تزریقِ وابستگی» اینجا همین است و بس: نه یک فریم‌ورک، فقط یک پارامتر.

این درس آن پارامتر را یک بار می‌سازد، رویِ یک لاگِ تماشایِ کوچک (`POST /watch` و `GET /watch/recent`) که جواب‌هایش به زمان بستگی دارد. همین شکل در ماژولِ ۵ استخرِ دیتابیس را هم می‌برد. و پرسشی را می‌پرسد که دنیایِ Java کمتر می‌پرسد: این کار کِی می‌ارزد و کِی فقط تشریفات است؟

---

## مفهوم

### یک struct، برایِ هر درخواست یک کلون

`axum` به هر درخواست کپیِ خودش از وضعیت را می‌دهد. پس نوعِ وضعیت باید `Clone` باشد و کلون‌کردنش باید ارزان باشد. راهِ داشتنِ هر دو همان قاعده‌ی ۳.۲.۳ است: هر چیزِ مشترک را پشتِ یک `Arc` بگذار، تا کلون یک اشاره‌گر را کپی کند و یک شمارنده را بالا ببرد. `examples/01-clone-shares-the-store.rs` آن را با `Arc::strong_count` نشان می‌دهد:

```rust
#[derive(Clone, Default)]
struct AppState {
    titles: Arc<Mutex<Vec<String>>>,
}
// main: let per_request = state.clone();
//       per_request.titles.lock().unwrap().push("Frieren".to_string());
```

```text
handles after creating the state: 1
handles after one clone:          2
seen through the original:        ["Frieren"]
same allocation:                  true
handles after the clone is gone:  1
```

```senpai-visual
{"kind":"ownership","labels":["AppState (اصلی)","AppState (کلونِ درخواستِ ۱)","AppState (کلونِ درخواستِ ۲)","یک تخصیصِ Arc: ذخیره‌گاه","شمارنده‌ی قوی = ۳"]}
```

کلونِ `AppState` یک struct تازه است که فیلدهایِ `Arc`اش به همان تخصیص اشاره می‌کنند. چیزی پشتِ اشاره‌گر کپی نمی‌شود، و یک `push` از راهِ یک دسته از همه‌ی دسته‌هایِ دیگر دیده می‌شود. وقتی درخواست تمام می‌شود و کلونش drop می‌شود، شمارنده پایین می‌آید. ذخیره‌گاه مشترک است چون *اشاره‌گر* مشترک است، و `Mutex`ِ داخلش (۳.۲.۳، ۲.۸.۱) است که تغییردادنِ آن را از پشتِ یک دسته‌ی مشترک ایمن می‌کند.

فیلدی که پشتِ `Arc` نیست فرق دارد: در هر کلون کپی می‌شود. برایِ یک `Config` کوچک با دو عدد این هیچ هزینه‌ای ندارد. برایِ یک struct بزرگ هزینه‌ی واقعی است، و راهِ رفعش همان است: بپیچش در `Arc`.

### گرفتنِ یک تکه: `FromRef` و `State<Piece>`

هندلر می‌تواند کلِ وضعیت را بگیرد، `State<AppState>`، و در آن دست ببرد. کار می‌کند، ولی آن‌وقت امضایِ هر هندلر می‌گوید «شاید از هر چیزی استفاده کنم»، و تستِ یک هندلر باید همه‌چیز را بسازد. `axum` ابزارِ بهتری دارد: `FromRef`. نوعی که `FromRef<AppState>` را پیاده کند یعنی می‌شود آن را از یک `&AppState` بیرون کشید، و آن‌وقت `State<ThatType>` در هندلر کار می‌کند، با اینکه وضعیتِ روتر `AppState` است. derive این impl‌ها را برایت می‌نویسد، یکی برایِ هر فیلد. `examples/02-fromref-substates.rs`:

```rust
#[derive(Clone, FromRef)]
struct AppState {
    greeting: Greeting,
    hits: Arc<AtomicU32>,
}

async fn count(State(hits): State<Arc<AtomicU32>>) -> String { /* ... */ }
async fn hello(State(greeting): State<Greeting>) -> String { /* ... */ }
// .route("/count", get(count)).route("/hello", get(hello)).with_state(state)
```

```text
GET /hello -> konnichiwa
GET /count -> hit number 1
GET /count -> hit number 2
GET /both -> konnichiwa (hits so far: 2)
```

`/both` دو آرگومانِ `State` را هم‌زمان می‌گیرد. هرکدام با `FromRef`ِ خودش از همان `AppState` کشیده می‌شود. `State` یک اکسترکتور است مثلِ آن‌هایی که در ۳.۲.۱ دیدی (فقط هرگز شکست نمی‌خورد)، پس یک هندلر هر تعداد که بخواهد می‌تواند داشته باشد. در سورسِ واقعیِ `axum` نسخه‌ی ۰.۸.۹ قاعده یک impl است: `State<Inner>` وقتی `FromRequestParts<Outer>` است که `Inner: FromRef<Outer>` برقرار باشد. و `axum-core` یک impl فراگیر دارد: `impl<T: Clone> FromRef<T> for T`. به همین دلیل `State<AppState>` ساده همیشه کار کرده است: یک وضعیت را همیشه می‌شود از خودش گرفت.

دو پیامد که در «خطاهایی که خواهی دید» به آن‌ها می‌رسی. derive بر اساسِ *نوعِ* فیلد کلید می‌زند، پس دو فیلد با یک نوع با هم برخورد می‌کنند. و `FromRef` فیلد را *کلون* می‌کند و بیرون می‌دهد: از یک فیلدِ `Arc` یک دسته‌ی ارزان می‌گیری، از یک فیلدِ `Config` یک کپیِ تازه‌ی تنظیمات.

چالشِ ۳.۲.۲ از طرفِ دیگرِ همین صفت استفاده کرد: یک اکسترکتور که برایِ هر وضعیتِ `S` با کرانِ `KeyStore: FromRef<S>` جنریک است، تا یک اکسترکتورِ کتابخانه‌ای در اپی کار کند که هرگز ندیده. derive اینجا راهی است که اپ می‌گوید «بله، می‌توانم یک `KeyStore` تحویل بدهم».

### یک وابستگی پشتِ یک صفت: `Clock`

لاگِ تماشا «اکنون» را لازم دارد. این هم وابستگی، به شکلِ یک صفت:

```rust
pub trait Clock: Send + Sync {
    fn now(&self) -> u64; // seconds since the Unix epoch
}
```

`Send + Sync` ابرصفت‌ها (supertrait، ۲.۳.۶) هستند، آن هم عمداً. وضعیت بینِ ریسمان‌هایِ کارگری که هندلرها را اجرا می‌کنند مشترک است، پس هر چیزِ داخلش باید `Send + Sync` باشد. اگر صفت این قول را ندهد، `Arc<dyn Clock>` هم آن را ندارد («خطاهایی که خواهی دید» پیام را نشان می‌دهد). دو پیاده‌سازی هست: `SystemClock` سیستم‌عامل را می‌خواند، و `FakeClock` یک عدد است که فقط وقتی تست بگوید حرکت می‌کند. وضعیت `Arc<dyn Clock>` را نگه می‌دارد، ذخیره‌گاه فقط داده دارد، و هندلرها هر دو را از راهِ `State` می‌گیرند:

```rust
#[derive(Clone, FromRef)]
pub struct AppState {
    pub clock: Arc<dyn Clock>,
    pub store: Arc<WatchStore>,
    pub config: Config,
}
```

ذخیره‌گاه از ساعت خبر ندارد: `store.add(title, at)` را به او می‌گویند *کِی*. این ذخیره‌گاه را یک ساختارِ دادهِ ساده نگه می‌دارد با تست‌هایِ بی‌دردسر (`tests/store_test.rs` نه ساعت می‌خواهد نه HTTP)، و تصمیمِ «الان ساعت چند است» را در یک جا نگه می‌دارد: هندلر.

```senpai-visual
{"kind":"concept","labels":["main: SystemClock","تست: FakeClock","Arc از dyn Clock در AppState","هندلر State از Arc dyn Clock را می‌خواند","همان هندلر، با هر ساعتی"]}
```

### نوعِ مشخص، جنریک، یا شیءِ صفتی؟

یک ساعت را می‌شود سه‌جور سیم‌کشی کرد. `examples/03-three-ways-to-hold-a-clock.rs` هر سه را می‌سازد:

```text
concrete: 100
its type: 03_three_ways_to_hold_a_clock::ConcreteState
generic:  200
its type: 03_three_ways_to_hold_a_clock::GenericState<03_three_ways_to_hold_a_clock::Fixed>
dyn:      300
its type: 03_three_ways_to_hold_a_clock::DynState
```

رفتار یکی است. چیزی که فرق می‌کند نوع است، و نوع همان چیزی است که در کدت پخش می‌شود:

| | مشخص (`Arc<SystemClock>`) | جنریک (`AppState<C: Clock>`) | شیءِ صفتی (`Arc<dyn Clock>`) |
|---|---|---|---|
| انتخاب کِی انجام می‌شود | هنگامِ کامپایل، ثابت | هنگامِ کامپایل، به‌ازایِ هر نمونه‌سازی | هنگامِ اجرا |
| نوعِ وضعیت | `AppState` | `AppState<Fixed>`، `AppState<SystemClock>` | `AppState` |
| در تست عوض می‌شود؟ | نه | بله | بله |
| هزینه | هیچ | به‌ازایِ هر فراخوانی هیچ؛ یک نسخه‌ی کد برایِ هر نوع | یک پرشِ اشاره‌گر در هر فراخوانی (فراخوانیِ vtable) |
| به چه سرایت می‌کند | به هیچ‌چیز | هر تابع، هندلر و `app()` که وضعیت را نام ببرد یک `<C>` و یک کران می‌گیرد | به هیچ‌چیز: نوع هنوز `AppState` است |

برایِ وابستگی‌ای که I/O می‌کند یا ساعت می‌خواند، پرشِ اشاره‌گر در برابرِ کارِ اطرافش قابلِ اندازه‌گیری نیست، پس شیءِ صفتی از نظرِ سادگی برنده است. پارامترِ جنریک رویِ وضعیت وقتی ابزارِ درست است که فراخوانی در یک حلقه‌ی داغ باشد، یا بخواهی کامپایلر از میانِ فراخوانی را ببیند. دو نکته درباره‌ی وضعیتِ جنریک که خوب است بدانی. `#[derive(FromRef)]` آن را یکسره ردّ می‌کند (ماکرو خودش می‌گوید «`#[derive(FromRef)]` doesn't support generics»)، پس هر impl از `FromRef` را باید خودت بنویسی. و `#[derive(Clone)]` رویِ `AppState<C>` یک کرانِ `C: Clone` اضافه می‌کند که `Arc<C>` هرگز لازمش نداشت. شیءِ صفتی هیچ‌کدام از این دو خراش را ندارد. این درس `Arc<dyn Clock>` را به کار می‌برد، و چالش اجازه می‌دهد نسخه‌ی جنریک را خودت حس کنی.

### تست با ساعتِ فیک

وقتی ساعت تزریق شده باشد، تست اپ را دورِ یک `FakeClock` می‌سازد و دسته‌ی خودش را نگه می‌دارد. چون `FakeClock` تغییرپذیریِ درونی دارد (یک atomic، همه‌ی متدها `&self`)، تست می‌تواند زمان را *بعد از ساختنِ اپ* جلو ببرد:

```rust
fn fixture(start: u64, config: Config) -> (Router, Arc<FakeClock>) {
    let clock = Arc::new(FakeClock::at(start));
    let state = AppState::new(clock.clone(), config);
    (app(state), clock)
}
// in a test:
//   post "first", clock.advance(60), post "second", clock.advance(30)
//   GET /watch/recent  ->  ["first", "second"]
//   clock.advance(20); GET /watch/recent  ->  ["second"]
```

تست هیچ‌وقت نمی‌خوابد: «یک ساعت بعد» یک `advance(3600)` است. و می‌تواند دقیقاً به هر مرز برسد (ورودیِ دقیقاً یک پنجره‌یِ قدیمی هنوز حساب می‌شود؛ یک ثانیه بیشتر و رفته است)، چیزی که ساعتِ واقعی فقط به‌اتفاق به آن می‌رسد. خلأیی که Django با `freezegun` پر می‌کند، Rust از بنیاد پر می‌کند، به قیمتِ اینکه ساعت را بدهی دستِ تابع. تشبیه کجا می‌شکند: `freezegun` زمان را برایِ *همه‌ی* کدِ پردازش عوض می‌کند، کتابخانه‌ها هم. اینجا فقط کدی که این ساعت را گرفته آن را می‌بیند، و این هم ویژگی است (هیچ‌چیز پشتِ سرت فیک نمی‌شود) و هم محدودیت (کتابخانه‌ای که خودش `SystemTime::now()` را صدا بزند واقعی می‌ماند).

### کِی تزریق می‌ارزد و کِی تشریفات است

نظرِ صریح، تا بتوانی با دلیل مخالفت کنی. یک وابستگی را وقتی پشتِ صفت بگذار که دست‌کم یکی از این‌ها برقرار باشد:

۱. **قطعی نیست**: ساعت، مولدِ عددِ تصادفی، مولدِ شناسه. تست نمی‌تواند رویِ چیزی که عوض می‌شود assert کند.
۲. **با دنیایِ بیرون حرف می‌زند**: ارسال‌کننده‌ی ایمیل، درگاهِ پرداخت، فراخوانِ webhook، سرویسِ دیگری رویِ HTTP. تست‌ها نباید ایمیلِ واقعی بفرستند، و حالتِ شکست («درگاه از کار افتاده») باید هر وقت خواستی قابلِ تولید باشد.
۳. **دو پیاده‌سازیِ واقعی دارد**: یک ذخیره‌گاهِ در حافظه و یکی رویِ Postgres. اینجاست که یک صفتِ مخزن (repository) می‌ارزد، و موضوعِ [۳.۵.۵ — الگویِ repository](../../05-postgres-and-sqlx/05-repository-pattern/README.fa.md) است.

و وقتی نه، که:

- **صفت دقیقاً یک پیاده‌سازی دارد و هیچ تستی فیکش نمی‌کند.** صفت تشریفاتِ محض است: یک فایلِ بیشتر، یک نامِ بیشتر، یک لایه‌ی بیشتر برایِ خواندن، برایِ درزی که هیچ‌کس از آن استفاده نمی‌کند. صفت را اولین باری اضافه کن که یک تست واقعاً لازم دارد فیکش کند، نه زودتر. اضافه‌کردنِ دیرتر یک تغییرِ مکانیکی است؛ برداشتنِ یک صفتِ حدسی هزینه‌ی مشابهی دارد، و هزینه‌ی حدس را هر روز تا آن زمان می‌پردازی.
- **وابستگی منطقِ محض است.** تابعی که یک عنوان را اعتبارسنجی می‌کند، تابعی که قیمت‌ها را جمع می‌زند. صدایش بزن. فیک‌کردنش فقط فیکِ خودت را تست می‌کند.
- **دنبالِ یک کانتینر می‌گردی.** کتابخانه‌هایِ «dependency injection» وجود دارند. کلِ سازوکارِ این درس یک struct است، یک `Arc` برایِ هر فیلد، و یک derive. اگر سیم‌کشیِ اپلیکیشنت در یک صفحه جا نمی‌شود، اپ مشکلِ ساختار دارد که کانتینر فقط پنهانش می‌کند.

به همین دلیل *ذخیره‌گاهِ* لاگِ تماشا یک struct مشخص است، نه یک صفت: در حافظه فعلاً همان چیزِ واقعی است و هیچ تستی دومی لازم ندارد. ساعت صفت است چون قاعده‌ی ۱ برقرار است. وقتی ۳.۵.۵ Postgres را می‌آورد، ذخیره‌گاه از نوعِ دوم می‌شود (قاعده‌ی ۳)، و نه پیش‌تر.

---

## دست‌به‌کد

سه مثالی را که کامپایل می‌شوند اجرا کن. `04` تا `07` عمداً خراب‌اند و پشتِ فیچرِ `broken` هستند؛ «خطاهایی که خواهی دید» هرکدام را نشان می‌دهد. (خروجیِ `01` و `02` و `03` همان بلوک‌هایِ «مفهوم» است.)

```sh
cargo run -p p3-04-02-app-state-and-dependency-wiring --example 01-clone-shares-the-store
cargo run -p p3-04-02-app-state-and-dependency-wiring --example 02-fromref-substates
cargo run -p p3-04-02-app-state-and-dependency-wiring --example 03-three-ways-to-hold-a-clock
```

تست‌ها قرمز شروع می‌شوند. `src/lib.rs` اسکلتِ کامل را دارد، و هر `todo!()` یک کامنتِ مستندی دارد که دقیقاً می‌گوید چه باید بکند. ببین کجایی:

```sh
for t in clock_test store_test api_test; do cargo test -p p3-04-02-app-state-and-dependency-wiring --test $t 2>&1 | grep 'test result'; done
```

```text
test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

وقتی هر سه سبز شدند، سرورِ واقعی را اجرا کن. رویِ `127.0.0.1:3150` گوش می‌دهد و `SystemClock` را سیم‌کشی می‌کند (`src/main.rs` تنها جایی است که وابستگی‌هایِ واقعی انتخاب می‌شوند). این خروجی‌ها رویِ راه‌حل گرفته شده‌اند:

```sh
cargo run -p p3-04-02-app-state-and-dependency-wiring &
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"  Frieren  "}' http://127.0.0.1:3150/watch
curl -s -w '\n%{http_code}\n' -X POST -H 'content-type: application/json' -d '{"title":"   "}' http://127.0.0.1:3150/watch
curl -s http://127.0.0.1:3150/watch/recent
```

```text
{"id":1,"title":"Frieren","watched_at":1791112811}
201
invalid title
422
[{"id":1,"title":"Frieren","watched_at":1791112811}]
```

(`watched_at` زمانِ واقعیِ درخواست است، پس عددِ تو فرق می‌کند.) فاصله‌های دورِ عنوان پیش از ذخیره‌شدن پاک شد، عنوانِ خالی با `422` ردّ شد، و درخواستِ دوم ورودیِ درخواستِ اول را دید: هر دو هندلر کلون‌هایی از یک وضعیت دارند. سرور را با `kill %1` متوقف کن. بعد این‌ها را امتحان کن:

۱. در `src/main.rs` به‌جایِ `SystemClock` یک `FakeClock::at(0)` بگذار. حالا `watched_at` چه می‌گوید، و به کدام فایلِ دیگر دست نزدی؟
۲. به `Config` مقدارِ `max_title_len` برابرِ `5` بده و عنوانی شش‌حرفی بفرست. کدام هندلر تصمیم می‌گیرد، و عدد را از کجا آورد؟
۳. `recent_watches` را طوری عوض کن که به‌جایِ سه `State` جدا، `State<AppState>` بگیرد. هنوز کامپایل می‌شود؟ امضا دیگر چه چیزی را نمی‌گفت؟

---

## خطاهایی که خواهی دید

هر مثالِ خرابِ زیر، پیش از خطا چند هشدارِ `unused variable` هم از اسکلتِ ناتمامِ `src/lib.rs` چاپ می‌کند. وقتی پیاده‌سازی‌اش کنی ناپدید می‌شوند، و در خروجی‌ها نیامده‌اند.

### `E0277` — وضعیت `Clone` نیست

```rust
struct AppState {
    visits: Arc<Mutex<u64>>,
}
```

`examples/04-state-not-clone-broken.rs` `#[derive(Clone)]` را فراموش کرده:

```text
error[E0277]: the trait bound `fn(State<AppState>) -> impl Future<Output = String> {visits}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:27:31
    |
 27 |         .route("/visits", get(visits))
    |                           --- ^^^^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(State<AppState>) -> impl Future<Output = String> {visits}`
    |                           |
    |                           required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:167:16
    |
167 |             H: Handler<T, S>,
    |                ^^^^^^^^^^^^^ required by this bound in `get`
...
441 | top_level_handler_fn!(get, GET);
    | -------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:26:24
    |
 26 |     let _app: Router = Router::new()
    |                        ^^^^^^^^^^^^^ the trait `Clone` is not implemented for `AppState`
    |
note: required by a bound in `Router::<S>::new`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:8
    |
140 |     S: Clone + Send + Sync + 'static,
    |        ^^^^^ required by this bound in `Router::<S>::new`
...
146 |     pub fn new() -> Self {
    |            --- required by a bound in this associated function
help: consider annotating `AppState` with `#[derive(Clone)]`
    |
 14 + #[derive(Clone)]
 15 | struct AppState {
    |

error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:27:10
    |
 27 |         .route("/visits", get(visits))
    |          ^^^^^ the trait `Clone` is not implemented for `AppState`
    |
note: required by a bound in `Router::<S>::route`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:8
    |
140 |     S: Clone + Send + Sync + 'static,
    |        ^^^^^ required by this bound in `Router::<S>::route`
...
178 |     pub fn route(self, path: &str, method_router: MethodRouter<S>) -> Self {
    |            ----- required by a bound in this associated function
help: consider annotating `AppState` with `#[derive(Clone)]`
    |
 14 + #[derive(Clone)]
 15 | struct AppState {
    |

error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:27:27
    |
 27 |         .route("/visits", get(visits))
    |                           ^^^^^^^^^^^ the trait `Clone` is not implemented for `AppState`
    |
note: required by a bound in `axum::routing::get`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:169:16
    |
169 |             S: Clone + Send + Sync + 'static,
    |                ^^^^^ required by this bound in `get`
...
441 | top_level_handler_fn!(get, GET);
    | -------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)
help: consider annotating `AppState` with `#[derive(Clone)]`
    |
 14 + #[derive(Clone)]
 15 | struct AppState {
    |

error[E0277]: the trait bound `AppState: Clone` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\04-state-not-clone-broken.rs:28:10
    |
 28 |         .with_state(state);
    |          ^^^^^^^^^^ the trait `Clone` is not implemented for `AppState`
    |
note: required by a bound in `Router::<S>::with_state`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:8
    |
140 |     S: Clone + Send + Sync + 'static,
    |        ^^^^^ required by this bound in `Router::<S>::with_state`
...
408 |     pub fn with_state<S2>(self, state: S) -> Router<S2> {
    |            ---------- required by a bound in this associated function
help: consider annotating `AppState` with `#[derive(Clone)]`
    |
 14 + #[derive(Clone)]
 15 | struct AppState {
    |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-04-02-app-state-and-dependency-wiring` (example "04-state-not-clone-broken") due to 5 previous errors

```

**کامپایلر به چه ایراد می‌گیرد:** `axum` وضعیت را برایِ هر درخواست کپی می‌کند، پس `Router` به `S: Clone + Send + Sync + 'static` نیاز دارد. وضعیت `Clone` ندارد. کامپایلر پنج خطا چاپ می‌کند. اولی دوباره همان «این `Handler` نیست»ِ مبهم است، چون هندلر `State<AppState>` را نام می‌برد. چهارتایِ دیگر همان شکایت‌اند، `AppState: Clone` برقرار نیست، یک‌بار در هر متدی که کران را دارد (`Router::new`، `route`، `get`، `with_state`)، و خطِ `help:` زیرِ هرکدام کلِ جواب است.

**رفع:** `#[derive(Clone)]` رویِ `AppState`.

**چرا این رفع است:** derive هر فیلد را به‌نوبت کلون می‌کند، و هر فیلد یک `Arc` است که کلونش همان کپیِ ارزانِ اشاره‌گر از ابتدایِ درس است. به آنچه نمی‌گوید دقت کن: نمی‌گوید *داده* `Clone` است. `Mutex<u64>` نیست، و لازم هم نیست باشد.

### `E0277` — `dyn Trait` بدونِ `Send + Sync`

```rust
trait Clock {
    fn now(&self) -> u64;
}
```

`examples/05-dyn-without-send-sync-broken.rs` یک `Arc<dyn Clock>` در وضعیتش دارد، ولی صفت درباره‌ی ریسمان‌ها قولی نمی‌دهد:

```text
error[E0277]: the trait bound `fn(State<AppState>) -> impl Future<Output = String> {now}: Handler<_, _>` is not satisfied
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:39:56
    |
 39 |     let _app: Router = Router::new().route("/now", get(now)).with_state(state);
    |                                                    --- ^^^ the trait `Handler<_, _>` is not implemented for fn item `fn(State<AppState>) -> impl Future<Output = String> {now}`
    |                                                    |
    |                                                    required by a bound introduced by this call
    |
    = note: Consider using `#[axum::debug_handler]` to improve the error message
note: required by a bound in `axum::routing::get`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\method_routing.rs:167:16
    |
167 |             H: Handler<T, S>,
    |                ^^^^^^^^^^^^^ required by this bound in `get`
...
441 | top_level_handler_fn!(get, GET);
    | -------------------------------
    | |                     |
    | |                     required by a bound in this function
    | in this macro invocation
    = note: this error originates in the macro `top_level_handler_fn` (in Nightly builds, run with -Z macro-backtrace for more info)

error[E0277]: `(dyn Clock + 'static)` cannot be shared between threads safely
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:39:24
    |
 39 |     let _app: Router = Router::new().route("/now", get(now)).with_state(state);
    |                        ^^^^^^^^^^^^^ `(dyn Clock + 'static)` cannot be shared between threads safely
    |
    = help: the trait `Sync` is not implemented for `(dyn Clock + 'static)`
    = note: required for `Arc<(dyn Clock + 'static)>` to implement `Send`
note: required because it appears within the type `AppState`
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:19:8
    |
 19 | struct AppState {
    |        ^^^^^^^^
note: required by a bound in `Router::<S>::new`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:16
    |
140 |     S: Clone + Send + Sync + 'static,
    |                ^^^^ required by this bound in `Router::<S>::new`
...
146 |     pub fn new() -> Self {
    |            --- required by a bound in this associated function

error[E0277]: `(dyn Clock + 'static)` cannot be sent between threads safely
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:39:24
    |
 39 |     let _app: Router = Router::new().route("/now", get(now)).with_state(state);
    |                        ^^^^^^^^^^^^^ `(dyn Clock + 'static)` cannot be sent between threads safely
    |
    = help: the trait `Send` is not implemented for `(dyn Clock + 'static)`
    = note: required for `Arc<(dyn Clock + 'static)>` to implement `Send`
note: required because it appears within the type `AppState`
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\05-dyn-without-send-sync-broken.rs:19:8
    |
 19 | struct AppState {
    |        ^^^^^^^^
note: required by a bound in `Router::<S>::new`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:140:16
    |
140 |     S: Clone + Send + Sync + 'static,
    |                ^^^^ required by this bound in `Router::<S>::new`
...
146 |     pub fn new() -> Self {
    |            --- required by a bound in this associated function

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-04-02-app-state-and-dependency-wiring` (example "05-dyn-without-send-sync-broken") due to 3 previous errors

```

**کامپایلر به چه ایراد می‌گیرد:** وضعیت باید `Send + Sync` باشد، و `Arc<T>` فقط وقتی `Send` و `Sync` است که `T` هر دو باشد (۲.۸.۴). `T` اینجا `dyn Clock` است، و یک `dyn Clock` ساده یک پیاده‌ساز *ناشناخته* است: می‌تواند نوعی باشد که یک `Rc` در خودش دارد. کامپایلر باید بدترین حالت را فرض کند، و پیامِ اول («این `Handler` نیست») باز همان پیامِ عمومی است، با علتِ واقعی در دو خطای `cannot be ...` زیرش.

**رفع:** قول را بخشی از صفت کن: `trait Clock: Send + Sync`. (یا در محلِ استفاده `Arc<dyn Clock + Send + Sync>` بنویس، که کار می‌کند ولی باید همه‌جا تکرارش کنی.)

**چرا این رفع است:** ابرصفت‌ها (۲.۳.۶) «هر `Clock` یک `Send + Sync` است» را به قاعده‌ای تبدیل می‌کنند که کامپایلر در هر `impl Clock for ...` وارسی می‌کند. فیکی که یک `Rc` نگه دارد همان‌جا ردّ می‌شود، جایی که نوشتی‌اش، نه به‌صورتِ خطایی سه لایه آن‌ورتر.

### `E0308` — یک زیروضعیت بدونِ `FromRef`

`examples/06-no-fromref-for-substate-broken.rs` هندلری دارد که `State<Config>` می‌گیرد، ولی وضعیت یک `AppState` است که فقط `Clone` را derive کرده:

```text
error[E0308]: mismatched types
   --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\06-no-fromref-for-substate-broken.rs:33:77
    |
 33 |     let _app: Router = Router::new().route("/hello", get(hello)).with_state(state);
    |                                                                  ---------- ^^^^^ expected `Config`, found `AppState`
    |                                                                  |
    |                                                                  arguments to this method are incorrect
    |
note: method defined here
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\routing\mod.rs:408:12
    |
408 |     pub fn with_state<S2>(self, state: S) -> Router<S2> {
    |            ^^^^^^^^^^

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p3-04-02-app-state-and-dependency-wiring` (example "06-no-fromref-for-substate-broken") due to 1 previous error

```

**کامپایلر به چه ایراد می‌گیرد:** این یکی غافل‌گیرکننده است: `E0277` نیست. بدونِ یک impl از `FromRef<AppState>` برایِ `Config`، تنها راهی که `State<Config>` کار کند impl فراگیر (`FromRef<T> for T`) است، که می‌گوید «از یک `Config` می‌شود یک `Config` گرفت». پس کامپایلر نتیجه می‌گیرد وضعیتِ روتر *همان* `Config` است، و `AppState`ای را که به `.with_state` می‌دهی با «انتظارِ `Config`، دریافتِ `AppState`» ردّ می‌کند. پیام علامت را توصیف می‌کند (نوعِ وضعیتِ غلط)، و علت derive‌ای است که نیست.

**رفع:** `#[derive(Clone, FromRef)]` رویِ `AppState` (فیچرِ `macros`ِ `axum` در `Cargo.toml`ِ این درس روشن است)، دقیقاً مثلِ `examples/02-fromref-substates.rs`.

**چرا این رفع است:** derive یک `impl FromRef<AppState> for Config` اضافه می‌کند، و وضعیتِ روتر دوباره `AppState` می‌شود. اگر هر وقت در یک فراخوانیِ `.with_state` «expected `X`, found `AppState`» دیدی، ببین کدام هندلر `State<X>` می‌خواهد. اگر هندلر `State<AppState>` بخواهد و `.with_state(...)` را اصلاً فراموش کرده باشی هم همین سردرگمی پیش می‌آید. ۳.۲.۱ آن حالت را پوشش می‌دهد؛ اینجا علت فرق دارد و پیام از همان خانواده است.

### `E0119` — دو فیلد با یک نوع

`examples/07-two-fields-same-type-broken.rs` دو شمارنده دارد، `hits` و `misses`، هر دو `Arc<AtomicU32>`:

```text
error[E0119]: conflicting implementations of trait `FromRef<AppState>` for type `Arc<Atomic<u32>>`
  --> phase3-backend-foundations\04-configuration-and-app-structure\02-app-state-and-dependency-wiring\examples\07-two-fields-same-type-broken.rs:19:13
   |
18 |     hits: Arc<AtomicU32>,
   |           --- first implementation here
19 |     misses: Arc<AtomicU32>,
   |             ^^^ conflicting implementation for `Arc<Atomic<u32>>`

For more information about this error, try `rustc --explain E0119`.
error: could not compile `p3-04-02-app-state-and-dependency-wiring` (example "07-two-fields-same-type-broken") due to 1 previous error

```

**کامپایلر به چه ایراد می‌گیرد:** derive برایِ هر فیلد یک impl از `FromRef<AppState>` می‌نویسد، و هدفِ impl *نوعِ* فیلد است. دو فیلدِ `Arc<AtomicU32>` یعنی دو impl از یک صفت برایِ یک نوع، و `State<Arc<AtomicU32>>` نمی‌تواند بگوید کدام را می‌خواهد. (نوع به‌صورتِ `Atomic<u32>` چاپ شده چون `AtomicU32` یک نامِ مستعار است.)

**رفع:** به هرکدام نوعِ خودش را بده، با یک newtype:

```rust
#[derive(Clone)]
struct Hits(Arc<AtomicU32>);
#[derive(Clone)]
struct Misses(Arc<AtomicU32>);

#[derive(Clone, FromRef)]
struct AppState { hits: Hits, misses: Misses }
```

**چرا این رفع است:** `State<Hits>` و `State<Misses>` حالا دو نوعِ متفاوت‌اند، پس هرکدام دقیقاً یک impl دارد، و هندلر نمی‌تواند `misses` را جایِ `hits` بگیرد. newtype سیم‌کشی را هم خودتوضیح‌گر می‌کند. (رفعِ دیگر این است که یکی‌یکی نخواهی: در هندلرهایی که هر دو را لازم دارند `State<AppState>` بگیر.)

---

## تمرین

### گرم‌کردن

<details>
<summary>هر درخواست کلونِ خودش از <code>AppState</code> را می‌گیرد. این ذخیره‌گاه را کپی می‌کند؟ پس چرا نوشتنِ هم‌زمانِ دو درخواست باز ایمن است؟</summary>

قبل از دیدنِ جواب، خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

نه. وضعیت یک `Arc<WatchStore>` نگه می‌دارد، پس کلون یک اشاره‌گر را کپی می‌کند و یک شمارنده را بالا می‌برد، و همه‌ی کلون‌ها به یک ذخیره‌گاه اشاره می‌کنند. نوشتنِ هم‌زمان ایمن است چون ذخیره‌گاه ورودی‌هایش را پشتِ یک `Mutex` نگه می‌دارد (۳.۲.۳، ۲.۸.۱)، و `Arc` فقط دسترسی را مشترک می‌کند، قفل نمی‌کند.

</details>

<details>
<summary><code>AppState</code> فیلدِ <code>config: Config</code> دارد و <code>FromRef</code> را derive می‌کند. هندلری <code>State&lt;Config&gt;</code> می‌گیرد. چه دریافت می‌کند: خودِ تنظیماتِ اصلی یا یک کپی؟</summary>

قبل از دیدنِ جواب، خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

یک کپی. `FromRef` فیلد را از وضعیت کلون می‌کند و بیرون می‌دهد (`from_ref(&AppState) -> Config`)، پس هندلر مالکِ یک `Config` تازه است. برایِ دو عدد این رایگان است. برایِ فیلدِ `Arc` «کپی» یک دسته‌ی ارزان به مقدارِ مشترک است، و برای همین چیزهایِ مشترک پشتِ `Arc` می‌روند و داده‌ی ساده می‌تواند ساده بماند.

</details>

<details>
<summary>تست اپ را با <code>FakeClock::at(1000)</code> می‌سازد، «a» را می‌فرستد، <code>advance(250)</code> را صدا می‌زند، «b» را می‌فرستد. <code>watched_at</code>ِ «b» چند است؟</summary>

قبل از دیدنِ جواب، خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

`1250`. ساعتِ فیک همان را می‌خواند که رویش گذاشته‌اند، به‌اضافه‌ی هرچه از آن به بعد جلو برده‌اند، و هندلر آن را در لحظه‌ی درخواست از همان دسته‌ی مشترک می‌خواند.

</details>

<details>
<summary>با پنجره‌ی ۱۰۰ ثانیه، ورودیِ ثبت‌شده در زمانِ ۵۰۰ و ساعتِ ۶۰۰: آیا <code>GET /watch/recent</code> آن را فهرست می‌کند؟ و در ۶۰۱؟</summary>

قبل از دیدنِ جواب، خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

در ۶۰۰ بله، در ۶۰۱ نه. نقطه‌ی برش `now - window` است، و ورودیِ دقیقاً در نقطه‌ی برش شامل می‌شود (مشخصات می‌گوید «هنوز حساب می‌شود»). در ۶۰۱ برش ۵۰۱ است، و ۵۰۰ قبل از آن است.

</details>

<details>
<summary>هم‌تیمی‌ات <code>trait Hasher</code> را اضافه می‌کند، با یک پیاده‌سازی که یک تابعِ محض را می‌پیچد و هرگز فیکش نمی‌شود. این صفت به درد می‌خورد؟</summary>

قبل از دیدنِ جواب، خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

نه. قطعی‌نبودن ندارد، به دنیایِ بیرون دست نمی‌زند، و یک پیاده‌سازی دارد. صفت یک نام، یک فایل و یک لایه اضافه می‌کند و درزی نمی‌خرد که کسی استفاده کند. تابع را مستقیم صدا بزن، و صفت را روزی اضافه کن که یک تست واقعاً لازم دارد فیکش کند.

</details>

<details>
<summary>در پایتون می‌توانی تابعی را که یک ویو import کرده <code>mock.patch</code> کنی. چرا در Rust نمی‌شود همین را با <code>SystemTime::now()</code> کرد؟</summary>

قبل از دیدنِ جواب، خودت فکر کن.

</details>

<details>
<summary>جواب</summary>

پایتون نام را هنگامِ فراخوانی از یک دیکشنریِ ماژول پیدا می‌کند که تست می‌تواند بازنویسی‌اش کند. Rust فراخوانی را هنگامِ کامپایل به یک تابعِ مشخص وصل می‌کند. چیزی برایِ بازنویسی نیست، پس تنها راهِ جایگزینی این است که *فراخوان* وابستگی را به‌عنوان ورودی بگیرد، که همان `Clock` در وضعیت است.

</details>

### تعمیر

هر چهار مثالِ خراب را درست کن. هرکدام را با `--features broken` بساز تا مطمئن شوی:

۱. `examples/04-state-not-clone-broken.rs` کامپایل شود.
۲. `examples/05-dyn-without-send-sync-broken.rs` کامپایل شود. رفع را در صفت بکن، نه در محلِ استفاده.
۳. `examples/06-no-fromref-for-substate-broken.rs` کامپایل شود، بدونِ اینکه هندلر را عوض کنی.
۴. `examples/07-two-fields-same-type-broken.rs` کامپایل شود، و هندلرِ `/hits` هنوز شمارنده‌ی `hits` را بخواند.

### پیاده‌سازی

هر چیزی در `src/lib.rs` که `todo!()` است: `SystemClock::now`، `FakeClock::at` و `advance` و `now`، سه متدِ `WatchStore`، `AppState::new`، دو هندلر، و `app`. هر کامنتِ مستند مشخصاتِ کامل است (کدهایِ وضعیت، شکلِ JSON، مرزِ پنجره، سرنوشتِ عنوانی که دورش فاصله دارد)، پس نباید لازم شود تست‌ها را باز کنی. از پایین به بالا کار کن: ساعت‌ها، بعد ذخیره‌گاه، بعد وضعیت و روتر.

```sh
cargo test -p p3-04-02-app-state-and-dependency-wiring
```

`tests/clock_test.rs` (۶ تست) و `tests/store_test.rs` (۸ تست) اصلاً HTTP نمی‌خواهند. `tests/api_test.rs` (۱۲ تست) کلِ روتر را با `oneshot` و یک ساعتِ فیک می‌راند.

### بساز

یک وابستگیِ دومِ تزریق‌شده اضافه کن، بدونِ دست‌زدن به اولی. یک صفتِ `Notifier` (باید `Send + Sync` باشد، با یک متد: `notify(&self, message: &str)`)، یک `NullNotifier` که هیچ کاری نمی‌کند، و یک فیلدِ `Arc<dyn Notifier>` به نامِ `notifier` در `AppState`. امضایِ `AppState::new` باید همان بماند و پیش‌فرضش `NullNotifier` باشد، تا تست‌هایِ داده‌شده سبز بمانند، و یک `AppState::with_notifier(self, notifier: Arc<dyn Notifier>) -> Self` جدید آن را عوض می‌کند. `POST /watch` به‌ازایِ هر تماشایِ *ثبت‌شده* دقیقاً یک بار `notify` را با متنِ `watched: <title>` (همان عنوانِ بدونِ فاصله‌های دورش) صدا می‌زند، و برایِ ردشده هرگز. `RecordingNotifier`ِ خودت را بنویس (یک `Mutex<Vec<String>>` بس است) و تست‌هایِ خودت را در یک `tests/build_test.rs` تازه: یکی که دو `POST` دو اعلان به‌ترتیب می‌دهد، و یکی که عنوانِ خالی کسی را خبر نمی‌کند.

### چالش (اختیاری)

ساعت را به‌جایِ شیءِ صفتی یک پارامترِ جنریک کن: `AppState<C: Clock>`، هندلرهایِ جنریک رویِ `C`، و `app<C>`. تست‌هایِ داده‌شده را با آن پاس کن (فیکسچرها را باید عوض کنی، که اولین یافته‌ات است). بعد چند خط به زبانِ خودت بنویس: چه چیزی در کد حالا `C` را نام می‌برد که قبلاً نمی‌برد، آیا `#[derive(FromRef)]` همان‌طور که بود کار کرد، و در چه وضعیتی این بها را می‌پردازی. تستِ داده‌شده‌ای نیست: این یک آزمایشِ طراحی است و جوابش مالِ توست.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کار می‌آید |
|---|---|---|
| وضعیتِ اپلیکیشن | یک struct از نوعِ `Clone` که هر وابستگیِ مشترک را نگه می‌دارد و با `.with_state` وصل می‌شود | هر سرویسِ واقعی |
| فیلدِ `Arc` | یک دسته‌ی مشترک، پس کلونِ وضعیت یک اشاره‌گر را کپی می‌کند نه داده را | ذخیره‌گاه، استخر، هر چیزِ بزرگ یا تغییرپذیر |
| `FromRef` | «این نوع را می‌شود از آن وضعیت بیرون کشید»؛ derive‌اش کن تا برایِ هر فیلد یک impl بگیری | `State<Piece>` در هندلرها، اکسترکتورهایِ کتابخانه‌ای |
| زیروضعیت (sub-state) | یک فیلدِ وضعیت، که با نوعِ خودش خواسته می‌شود | هندلرهایی که فقط آنچه استفاده می‌کنند را نام می‌برند |
| `Arc<dyn Trait>` | وابستگی‌ای که هنگامِ اجرا انتخاب می‌شود، پشتِ یک اشاره‌گر | ساعت، ارسال‌کننده‌ی ایمیل، درگاه‌ها |
| فیک (fake) | یک جایگزین با رفتارِ واقعیِ ساده، که از راهِ صفت جا می‌افتد | `FakeClock`، یک ذخیره‌گاهِ در حافظه |
| تزریق (injection) | سپردنِ یک وابستگی از بیرون (اینجا: ساختنِ وضعیت در `main` یا در تست) | هر جا که تست کنترل می‌خواهد |

### الان می‌دانی

- `AppState` از نوعِ `Clone` است و کلون‌کردنش ارزان است چون هر چیزِ مشترک پشتِ یک `Arc` است. کلون داده را به اشتراک می‌گذارد.
- `#[derive(FromRef)]` به هندلر اجازه می‌دهد به‌جایِ کلِ struct یک `State<Piece>` بخواهد. فیلد را کلون می‌کند و بیرون می‌دهد و بر اساسِ نوع کلید می‌زند، پس دو فیلد با یک نوع به newtype نیاز دارند.
- صفتی با ابرصفت‌هایِ `Send + Sync` چیزی است که `Arc<dyn Trait>` را درونِ وضعیت مجاز می‌کند.
- نوعِ مشخص، جنریک و شیءِ صفتی سه راهِ نگه‌داشتنِ یک وابستگی‌اند. شیءِ صفتی نوعِ وضعیت را ساده نگه می‌دارد، و جنریک هزینه‌ی فراخوانی را صفر نگه می‌دارد و یک پارامترِ نوع را در کدت پخش می‌کند.
- ساعتِ فیک رفتارِ وابسته‌به‌زمان را از راهِ `oneshot` تست‌پذیر می‌کند، بدونِ خوابیدن و با دسترسی به هر مرز.
- صفت برایِ چیزی که قطعی نیست، چیزی که به دنیایِ بیرون دست می‌زند، یا چیزی که دو پیاده‌سازیِ واقعی دارد جایش را دارد. در غیرِ این صورت تشریفات است.

### بعداً کامل‌تر می‌بینی

- **`.with_state(...)`ِ فراموش‌شده و اولین نگاه به `State`**: [۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md)
- **یک اکسترکتورِ کتابخانه‌ای که رویِ وضعیت جنریک است و کرانِ `FromRef` دارد**: [۳.۲.۲ — نوشتنِ اکسترکتورِ خودت (`FromRequestParts`)](../../02-axum-and-rest-api-design/02-writing-your-own-extractor/README.fa.md)
- **`Config` در یک سرویسِ واقعی از کجا می‌آید**: [۳.۴.۱ — پیکربندیِ ۱۲فاکتوری و سکرت‌ها](../01-config-and-secrets/README.fa.md)
- **تکه‌ی بعدیِ لوله‌کشی در همین ماژول**: [۳.۴.۳ — خاموشیِ آرام، health و readiness](../03-graceful-shutdown-health-readiness/README.fa.md)
- **ذخیره‌گاهِ در حافظه که با Postgres جایگزین می‌شود، پشتِ یک صفت**: [۳.۵.۵ — الگویِ repository](../../05-postgres-and-sqlx/05-repository-pattern/README.fa.md)
- **یک وابستگیِ فیک در یک مجموعه‌تستِ بزرگ‌تر**: [۳.۸.۴ — factory و fixture برایِ داده‌یِ تست](../../08-error-handling-and-testing-at-scale/04-test-data-factories-and-fixtures/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا وضعیت باید `Clone` باشد، و چرا کلون‌کردنش ذخیره‌گاه را کپی نمی‌کند؟
- `#[derive(FromRef)]` چه تولید می‌کند، و چرا دو فیلد با یک نوع با هم برخورد می‌کنند؟
- چرا `Arc<dyn Clock>` به `Send + Sync` نیاز دارد، و تمیزترین جا برایِ گفتنش کجاست؟
- کِی جنریک را به `Arc<dyn Clock>` ترجیح می‌دهی، و چه بهایی می‌دهی؟
- تست چطور زمان را جلو می‌برد، و چرا هندلر آن را می‌بیند؟
- یک وابستگی در یک سرویسِ خودت نام ببر که سزاوارِ صفت است و یکی که نیست، و دلیلش را بگو.

---

## بیشتر

- [`State` در مستنداتِ `axum`](https://docs.rs/axum/0.8.9/axum/extract/struct.State.html): بخشِ «substates» با `FromRef`، و توصیه‌ی `Arc` در برابرِ کلون که این درس دنبالش می‌کند.
- [`FromRef` در `axum-core`](https://docs.rs/axum-core/0.5.6/axum_core/extract/trait.FromRef.html): صفت و تنها impl فراگیرش.
- [۲.۳.۷ — ارسالِ ایستا در برابرِ ارسالِ پویا](../../../phase2-intermediate/03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md): بده‌بستانِ جدولِ بالا، از اصل.
- [`tower::ServiceExt::oneshot`](https://docs.rs/tower/0.5.3/tower/trait.ServiceExt.html#method.oneshot): هر تستِ این درس چطور بدونِ سوکت درخواست می‌فرستد.
