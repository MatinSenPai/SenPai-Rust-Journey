# ۳.۴.۳ — خاموشیِ آرام، health و readiness

## در یک نگاه

بعد از این درس می‌توانی:

- یک سرورِ `axum` را با `with_graceful_shutdown` طوری متوقف کنی که درخواست‌هایِ در حالِ اجرا تمام شوند، اتصال‌هایِ تازه رد شوند، و وقتی آخری تمام شد فراخوانی `Ok(())` برگرداند.
- خودت future خاموشی را بسازی، تا یک سیگنال، یک کانال یا یک تست بتواند آن را راه بیندازد، و توضیح بدهی چرا به صفحه‌کلید گره نخورده است.
- `/health` (زنده‌بودن) و `/ready` (آمادگی) را بنویسی و بگویی چرا یک نمونه‌ی در حالِ تخلیه در دومی شکست می‌خورد و در اولی نه.
- `E0277` و `E0373` را که از دو اشتباهِ کلاسیکِ خاموشی می‌آیند بخوانی، و اشتباهی را که هیچ خطایی نمی‌دهد تشخیص بدهی.

**زمان:** حدود ۱۰۰ دقیقه · **پیش‌نیاز:**
[۳.۴.۲ — وضعیتِ اپلیکیشن و سیم‌کشیِ وابستگی‌ها](../02-app-state-and-dependency-wiring/README.fa.md)،
[۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md)،
[۳.۱.۳ — چیزهایی از HTTP که باید بدانی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)

---

## چرا اهمیت دارد

هر دیپلوی با متوقف‌شدنِ پردازشِ تو تمام می‌شود. Kubernetes، systemd، Docker و `kill` همه یک‌جور این کار را می‌کنند: `SIGTERM` می‌فرستند، چند ثانیه صبر می‌کنند، بعد `SIGKILL`. اگر برنامه‌ات `SIGTERM` را نادیده بگیرد، سیگنالِ دوم وسطِ درخواست قطعش می‌کند، و کلاینتی که نیمه‌ی یک پرداخت یا آپلود بود یک اتصالِ ریست‌شده می‌گیرد. در Django به‌ندرت به این فکر می‌کنی چون gunicorn برایت انجامش می‌دهد: با `SIGTERM` پردازشِ مادر به هر worker می‌گوید درخواستِ جاری را تمام کند و خارج شود (`--graceful-timeout` همان دستگیره‌ی «بعدش `SIGKILL`» است). در `axum` تا نخواهی هیچ‌چیز انجام نمی‌شود. این درس یادت می‌دهد چطور بخواهی.

نیمه‌ی دیگرِ ماجرا لودبالانسرِ جلوی توست. او باید بداند کی *دیگر* به تو ترافیک نفرستد، و فقط همان را می‌داند که تو از راهِ HTTP بگویی. `/health` و `/ready` برایِ همین‌اند. شبیه هم‌اند و قاطی‌کردنشان یک قطعیِ کلاسیک است: سرویسی که فقط مشغول است و می‌گوید «مُرده‌ام»، یا دیتابیسش رفته و می‌گوید «سالمم».

این درس ماژولِ ۳.۴ را می‌بندد. [۳.۴.۱](../01-config-and-secrets/README.fa.md) کاری کرد پردازش تنظیماتش را بخواند، [۳.۴.۲](../02-app-state-and-dependency-wiring/README.fa.md) یک جا برایِ حالتِ مشترک داد، و این درس یادش می‌دهد تمیز بایستد. بعدش [۳.۵.۱ — اتصال و pooling](../../05-postgres-and-sqlx/01-connecting-and-pooling/README.fa.md) چیزی واقعی برایِ بررسیِ `/ready` به دستت می‌دهد.

---

## مفهوم

### سرور وقتی از او می‌خواهند بایستد چه می‌کند

`axum::serve(listener, app)` تا مرگِ پردازش اجرا می‌شود. `.with_graceful_shutdown(signal)` یک چیز اضافه می‌کند: futureای که وقتی تمام شود یعنی «پذیرفتن را متوقف کن، آنچه داری را تمام کن». این برنامه آن را از هر دو طرف نشان می‌دهد. یک سرور بالا می‌آورد، یک درخواستِ کند شروع می‌کند، وقتی هنوز داخلِ هندلرش است خاموشی را راه می‌اندازد، و چاپ می‌کند چه شد (`examples/02-graceful-shutdown-timeline.rs`):

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 02-graceful-shutdown-timeline
```

```text
1. the slow request is inside its handler
2. shutdown triggered
3. a new connection is refused
4. server finished yet? false
5. slow request got: HTTP/1.1 200 OK / body "slow done"
6. server returned: Ok(())
```

ترتیب را بخوان. بعد از گامِ ۲ شنونده (listener) رفته است، پس گامِ ۳ یک اتصالِ ردشده است، با این حال درخواستِ کند، که *پیش از* راه‌افتادنِ خاموشی پذیرفته شده بود، هنوز `200` خودش را می‌گیرد. تسکِ سرور در گامِ ۴ تمام نشده است چون یک درخواست در حالِ اجراست. فقط در گامِ ۶، بعد از آخرین پاسخ، `Ok(())` برمی‌گرداند. کلِ قرارداد همین است.

```senpai-visual
{"kind": "async", "labels": ["تکمیلِ future خاموشی", "شنونده بسته می‌شود: اتصالِ تازه رد", "درخواست‌هایِ در حالِ اجرا ادامه می‌دهند", "اتصال‌هایِ بیکارِ keep-alive بسته می‌شوند", "آخرین پاسخ فرستاده می‌شود", "serve مقدارِ Ok برمی‌گرداند"]}
```

### future خاموشی فقط یک future است

`with_graceful_shutdown` هر `Future<Output = ()> + Send + 'static` را می‌گیرد. مثالِ بالا یک گیرنده‌ی `tokio::sync::oneshot` به کار برد که خودِ برنامه فعالش می‌کرد. یک سرویسِ واقعی به‌جایِ آن سیگنالِ سیستم‌عامل را به کار می‌برد:

```rust
async fn os_signal() {
    let ctrl_c = async { tokio::signal::ctrl_c().await.expect("install Ctrl-C handler") };
    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler").recv().await;
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! { _ = ctrl_c => {} _ = terminate => {} }
}
```

این تابع در `src/lib.rs` است و از قبل نوشته شده. `tokio::signal::ctrl_c()` رویِ لینوکس، macOS و ویندوز کار می‌کند. `SIGTERM`، سیگنالی که دیپلوی‌ها واقعاً می‌فرستند، فقط رویِ یونیکس هست، پس آن شاخه فقط آنجا کامپایل می‌شود؛ رویِ ویندوز شاخه‌ی دیگر futureای است که هرگز تمام نمی‌شود (`pending`). این ماشین ویندوز است، و برایِ همین هیچ تستی از آن استفاده نمی‌کند. یک تست نمی‌تواند Ctrl-C بزند و رویِ ویندوز نمی‌تواند `SIGTERM` بفرستد. پس قاعده‌ی طراحیِ این درس این است: **کدی که سرو می‌کند، راه‌انداز را به‌عنوانِ پارامتر می‌گیرد**. `main`ِ واقعی `os_signal()` را می‌دهد و یک تست گیرنده‌ی `oneshot`ای را که خودش کنترل می‌کند. `select!` از [۲.۹.۲](../../../phase2-intermediate/09-async-in-practice/02-select-and-cancellation-safety/README.fa.md) است: به‌محضِ تمام‌شدنِ یکی از شاخه‌ها تمام می‌شود.

### زنده‌بودن و آمادگی دو پرسشِ متفاوت‌اند

`examples/01-health-and-ready-server.rs` هر دو را رویِ پورتِ `3160` سرو می‌کند، با خاموشیِ آرام روی Ctrl-C:

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 01-health-and-ready-server
```

```text
listening on http://127.0.0.1:3160
```

در یک ترمینالِ دوم:

```sh
curl -i http://127.0.0.1:3160/health
curl -i http://127.0.0.1:3160/ready
```

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 2
date: Sun, 04 Oct 2026 11:14:33 GMT

ok
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
content-length: 5
date: Sun, 04 Oct 2026 11:14:33 GMT

ready
```

(هدرِ `date` زمانِ اجرای توست.) الان هر دو `200` می‌گویند. وقتی چیزی خراب شود از هم جدا می‌شوند، و به دو پرسشِ متفاوت جواب می‌دهند:

| پروب | پرسش | `200` یعنی | `503` یعنی | پلتفرم واکنش نشان می‌دهد با |
|---|---|---|---|---|
| `/health` (زنده‌بودن) | آیا این پردازش زنده است و اصلاً می‌تواند جواب بدهد؟ | بله | (هرگز نمی‌فرستی‌اش: پردازشی که نتواند جواب بدهد، پاسخی هم ندارد) | ری‌استارتِ پردازش |
| `/ready` (آمادگی) | آیا همین الان باید به این نمونه ترافیک فرستاد؟ | بله | الان نه: در حالِ تخلیه است، یا یک وابستگی خراب است | بیرون‌کشیدنش از استخر، **نه** ری‌استارت |

دو واکنش همه‌چیز را توضیح می‌دهند. اگر `/health` هر وقت دیتابیس خراب بود شکست می‌خورد، پلتفرم همه‌ی نمونه‌ها را ری‌استارت می‌کرد، و چون ری‌استارت دیتابیس را درست نمی‌کند، روی قطعی یک حلقه‌ی ری‌استارت هم می‌گرفتی. اگر `/ready` موقعِ تخلیه `200` می‌داد، لودبالانسر درخواست‌هایِ تازه را به نمونه‌ای می‌فرستاد که شنونده‌اش را بسته است. همان کدِ وضعیت، `503 Service Unavailable` (۳.۱.۳: یک `5xx` که می‌گوید «جایِ دیگر یا بعداً امتحان کن»)، برایِ آمادگی درست است چون خرابی موقتی است و تقصیرِ کلاینت نیست.

```senpai-visual
{"kind": "network", "labels": ["لودبالانسر", "پروبِ /ready", "نمونه‌ی A: ۲۰۰ آماده", "نمونه‌ی B: ۵۰۳ در حالِ تخلیه", "ترافیک فقط به A می‌رود"]}
```

### تخلیه: اول آمادگی می‌چرخد، بعد شنونده بسته می‌شود

یک ظرافت هست. لحظه‌ای که future خاموشی تمام شود، شنونده بسته می‌شود، پس لودبالانسری که درست بعدش `/ready` را بپرسد «اتصال رد شد» می‌گیرد، نه یک `503`ِ مرتب. یک سرویسِ آرام برای همین سه گام برمی‌دارد، با یک فاصله پیش از آخری:

۱. روی سیگنال، آمادگی را به `503` بچرخان («در حالِ تخلیه»)؛
۲. یک *مهلتِ آرامش* کوتاه صبر کن تا پروبِ بعدیِ لودبالانسر آن را ببیند و ترافیک را قطع کند؛
۳. فقط بعد اجازه بده future خاموشی تمام شود، که شنونده را می‌بندد.

تمرینِ `shutdown_future` دقیقاً همین است. حالتی که می‌چرخاند یک struct کوچکِ مشترک بینِ هندلرها و future است، پس پشتِ یک `Arc` می‌نشیند، با پرچم‌هایِ اتمیک ([۲.۸.۲](../../../phase2-intermediate/08-concurrency/02-rwlock-semaphore-oncelock-atomics/README.fa.md)) درونش:

```rust
pub struct Lifecycle {
    draining: AtomicBool,
    dependency_up: AtomicBool,
}
```

`dependency_up` راهی است که `/ready` وقتی دیتابیس بیاید («دیتابیسم رفته») گزارش بدهد (ماژولِ ۳.۵). هیچ‌چیز در این درس به دیتابیس نیاز ندارد.

### خاموشی با اتصال‌هایِ بیکار چه می‌کند

۳.۱.۳ گفت اتصالِ HTTP/1.1 بینِ درخواست‌ها باز می‌ماند (keep-alive). کلاینتی که چنین اتصالی را نگه دارد و هیچ درخواستی رویش نداشته باشد، یک خاموشیِ ساده‌لوحانه را برای همیشه بلوکه می‌کرد، پس سرور موقعِ خاموشی اتصال‌هایِ بیکار را می‌بندد. `examples/03-keep-alive-closed-on-shutdown.rs` نشانش می‌دهد:

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 03-keep-alive-closed-on-shutdown
```

```text
request 1 on the connection: HTTP/1.1 200 OK
request 2, same connection:  HTTP/1.1 200 OK
after shutdown, read() returned 0 bytes (end of stream)
server returned: Ok(())
```

دو درخواست یک اتصال را شریک بودند، بعد خاموشی تمامش کرد: برگشتنِ `read()` با `0` بایت همان پایانِ جریانِ TCP از ۳.۱.۱ است. اتصالی که وسطِ یک درخواست است حالتِ دیگری است: بسته *نمی‌شود*، که همان مثالِ بالاتر است.

### تستِ خاموشی بدونِ صفحه‌کلید

تست‌هایِ این درس یک سرورِ واقعی رویِ `127.0.0.1:0` بالا می‌آورند (پورتِ `0` یعنی «سیستم‌عامل یک پورتِ آزاد بدهد»، پس تست‌ها هرگز با هم برخورد نمی‌کنند)، با یک `TcpStream` ساده با آن حرف می‌زنند، و هرگز به‌اندازه‌ی یک مدتِ حدسی نمی‌خوابند. دو ابزار این را قطعی می‌کند:

- یک `tokio::sync::Notify` به هندلرِ کند اجازه می‌دهد بگوید «شروع شدم» و منتظرِ «می‌توانی تمام کنی» بماند. تست درخواست را شروع می‌کند، منتظرِ «شروع شد» می‌ماند، خاموشی را راه می‌اندازد، و فقط بعد هندلر را آزاد می‌کند. بینِ این سه رقابتی نیست.
- «اتصالِ تازه رد می‌شود» با پولینگِ `connect` تا وقتی شکست بخورد بررسی می‌شود، نه با صبرِ یک مدتِ حدسی.

رویِ ویندوز یک `connect`ِ ردشده حدودِ دو ثانیه طول می‌کشد تا شکست بخورد، پس تستی که منتظرش می‌ماند حدودِ دو ثانیه طول می‌کشد. آن سیستم‌عامل است، نه یک sleep در تست.

---

## دست‌به‌کد

سه مثالِ کارکننده برنامه‌هایِ مستقل‌اند؛ `src/lib.rs` نسخه‌ی کتابخانه‌ایِ همین ایده‌هاست که در تمرین‌ها می‌سازی. همه را به همین ترتیب اجرا کن و مقایسه کن:

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 01-health-and-ready-server
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 02-graceful-shutdown-timeline
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 03-keep-alive-closed-on-shutdown
```

برایِ `01`، در ترمینالش Ctrl-C بزن. دو خطِ `println!` بعد از `serve` در `main` نشان می‌دهند خاموشی انجام شد: اول اینکه سیگنالی رسید، بعد اینکه همه‌ی درخواست‌ها تمام شدند.

بعد `src/lib.rs` را بخوان: کامنتِ مستندِ هر تابع مشخصاتِ کاملِ آن است.

---

## خطاهایی که خواهی دید

### `E0277` — تابع، نه future

یک `async fn shutdown_signal()` قشنگ نوشتی و *اسمش* را دادی. `examples/04-fn-item-not-a-future-broken.rs`:

```sh
cargo build -p p3-04-03-graceful-shutdown-health-readiness --example 04-fn-item-not-a-future-broken --features broken
```

```text
error[E0277]: `fn() -> impl Future<Output = ()> {shutdown_signal}` is not a future
   --> phase3-backend-foundations\04-configuration-and-app-structure\03-graceful-shutdown-health-readiness\examples\04-fn-item-not-a-future-broken.rs:15:33
    |
 15 |         .with_graceful_shutdown(shutdown_signal)
    |          ---------------------- ^^^^^^^^^^^^^^^ `fn() -> impl Future<Output = ()> {shutdown_signal}` is not a future
    |          |
    |          required by a bound introduced by this call
    |
    = help: the trait `Future` is not implemented for fn item `fn() -> impl Future<Output = ()> {shutdown_signal}`
note: required by a bound in `Serve::<L, M, S>::with_graceful_shutdown`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\serve\mod.rs:153:12
    |
151 |     pub fn with_graceful_shutdown<F>(self, signal: F) -> WithGracefulShutdown<L, M, S, F>
    |            ---------------------- required by a bound in this associated function
152 |     where
153 |         F: Future<Output = ()> + Send + 'static,
    |            ^^^^^^^^^^^^^^^^^^^ required by this bound in `Serve::<L, M, S>::with_graceful_shutdown`
help: use parentheses to call this function
    |
 15 |         .with_graceful_shutdown(shutdown_signal())
    |                                                ++

error[E0277]: `WithGracefulShutdown<TcpListener, Router, Router, ...>` is not a future
   --> phase3-backend-foundations\04-configuration-and-app-structure\03-graceful-shutdown-health-readiness\examples\04-fn-item-not-a-future-broken.rs:16:10
    |
 16 |         .await
    |          ^^^^^ `WithGracefulShutdown<TcpListener, Router, Router, ...>` is not a future
    |
    = help: the trait `IntoFuture` is not implemented for `WithGracefulShutdown<TcpListener, Router, Router, ...>`
    = note: WithGracefulShutdown<TcpListener, Router, Router, ...> must be a future or must implement `IntoFuture` to be awaited
help: the trait `IntoFuture` is implemented for `WithGracefulShutdown<L, M, S, F>`
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\serve\mod.rs:332:1
    |
332 | / impl<L, M, S, F> IntoFuture for WithGracefulShutdown<L, M, S, F>
333 | | where
334 | |     L: Listener,
335 | |     L::Addr: Debug,
...   |
339 | |     S::Future: Send,
340 | |     F: Future<Output = ()> + Send + 'static,
    | |____________________________________________^
    = note: the full name for the type has been written to 'M:\SenPai-Rust-Journey\target\debug\examples\04_fn_item_not_a_future_broken.long-type-17115680415784593178.txt'
    = note: consider using `--verbose` to print the full type name to the console
help: remove the `.await`
    |
 16 -         .await
    |

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-04-03-graceful-shutdown-health-readiness` (example "04-fn-item-not-a-future-broken") due to 2 previous errors
```

(عددِ داخلِ نامِ فایلِ `long-type-...txt` هر اجرا عوض می‌شود.)

**کامپایلر به چه اعتراض دارد.** یک `async fn` تابعی است که یک future *برمی‌گرداند*. اسمش خودِ تابع است (یک «fn item»)، نه یک future. `with_graceful_shutdown` future می‌خواهد. خطای دوم اثرِ دومینویی است: چون آرگومانِ اول نامعتبر بود، کلِ builder هم قابلِ `await` نیست.

**رفع.** صدایش بزن: `.with_graceful_shutdown(shutdown_signal())`. `help`ِ خودِ کامپایلر هم همین را می‌گوید.

**چرا این رفع است.** صدازدنِ یک `async fn` هیچ‌کدام از بدنه‌اش را اجرا نمی‌کند؛ futureای را می‌سازد که آن را اجرا خواهد کرد. `axum` آن future را نگه می‌دارد و خودش `poll` می‌کند. دادنِ اسم یعنی دادنِ دستورِ پخت به‌جایِ غذا.

### `E0373` — بلوکِ async چیزی را قرض می‌گیرد که `main` مالکش است

می‌خواهی future خاموشی پرچمِ draining را هم بچرخاند، پس یک بلوکِ `async` می‌نویسی که از `draining` استفاده می‌کند. `examples/05-async-block-borrows-state-broken.rs`:

```sh
cargo build -p p3-04-03-graceful-shutdown-health-readiness --example 05-async-block-borrows-state-broken --features broken
```

```text
error[E0373]: async block may outlive the current function, but it borrows `draining`, which is owned by the current function
  --> phase3-backend-foundations\04-configuration-and-app-structure\03-graceful-shutdown-health-readiness\examples\05-async-block-borrows-state-broken.rs:15:33
   |
15 |         .with_graceful_shutdown(async {
   |                                 ^^^^^ may outlive borrowed value `draining`
16 |             tokio::signal::ctrl_c().await.unwrap();
17 |             draining.store(true, Ordering::SeqCst);
   |             -------- `draining` is borrowed here
   |
   = note: async blocks are not executed immediately and must either take a reference or ownership of outside variables they use
help: to force the async block to take ownership of `draining` (and any other referenced variables), use the `move` keyword
   |
15 |         .with_graceful_shutdown(async move {
   |                                       ++++

For more information about this error, try `rustc --explain E0373`.
error: could not compile `p3-04-03-graceful-shutdown-health-readiness` (example "05-async-block-borrows-state-broken") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد.** قیدِ `'static` است: future نباید چیزی از پشته‌ی `main` قرض بگیرد. بلوک، بدونِ `move`، `draining` را قرض می‌گیرد. تا جایی که نوع‌ها می‌دانند، `axum` ممکن است future را بیشتر از متغیرهایِ محلیِ `main` زنده نگه دارد.

**رفع.** `async move { ... }`، دقیقاً مثلِ `help`. اگر همان `Arc` جایِ دیگری هم لازم است (هندلرها)، اول `clone()` کن و کلون را داخلش منتقل کن، همان‌طور که `examples/01-health-and-ready-server.rs` می‌کند.

**چرا این رفع است.** `move` باعث می‌شود بلوک مالکِ متغیرهایِ گرفته‌شده‌اش باشد، پس قرضی نمی‌ماند و `'static` برآورده می‌شود. کلون‌کردنِ `Arc` ارزان است، برای همین راهِ معمولِ دادنِ یک پرچم به چند مالک است.

### هیچ خطایی نیست: فرستنده‌ی دراپ‌شده سرور را یکباره خاموش می‌کند

این یکی کامپایل و اجرا می‌شود. `examples/06-dropped-sender-instant-shutdown-trap.rs` خاموشی را با `stopped.await.ok();` به گیرنده‌ی یک `oneshot` وصل می‌کند، اما فرستنده را نگه نداشته و دراپ کرده است:

```sh
cargo run -p p3-04-03-graceful-shutdown-health-readiness --example 06-dropped-sender-instant-shutdown-trap
```

```text
server finished on its own: Ok(Ok(()))
a client trying to connect gets: ConnectionRefused
```

**چه شد.** `rx.await` وقتی فرستنده‌اش دراپ شود `Err` برمی‌گرداند، و `.ok()` آن خطا را دور انداخت. future تمام شد، و `axum` آن را «همین حالا خاموش شو» می‌خواند. سرور پیش از سرو‌کردنِ حتی یک درخواست متوقف شد.

**رفع.** فرق بگذار بینِ «فرستنده فعال کرد» و «فرستنده ناپدید شد». `src/lib.rs` تابعِ `fired(rx)` را دارد که برایِ فرستنده‌ی دراپ‌شده به‌جایِ تمام‌شدن تا ابد منتظر می‌ماند. و فرستنده را زنده نگه دار: آن را همان‌جایی نگه دار که کارش راه‌انداختنِ خاموشی است.

**چرا این رفع است.** یک راه‌انداز باید فقط عمداً فعال شود. تبدیلِ «دیگر چیزی نمی‌شنوم» به «بایست» راهی بی‌صداست برایِ اینکه یک باگِ بی‌ربط (فرستنده‌ای که زود دراپ شد) تبدیل به قطعی شود.

---

## تمرین

همه‌ی کد در `src/lib.rs` (یک `todo!()` برایِ هر تابع، با مشخصاتِ کاملش در کامنتِ مستند) و در `tests/` است. آخرین اجرایِ تست پیش از شروع این‌طور است:

```sh
cargo test -p p3-04-03-graceful-shutdown-health-readiness --lib only_ready
```

```text
running 1 test
test tests::only_ready_is_200 ... FAILED

failures:

---- tests::only_ready_is_200 stdout ----

thread 'tests::only_ready_is_200' (23020) panicked at phase3-backend-foundations\04-configuration-and-app-structure\03-graceful-shutdown-health-readiness\src\lib.rs:87:9:
not yet implemented: map each verdict to its status code
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    tests::only_ready_is_200

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

(عددِ داخلِ پرانتز بعد از `thread` شناسه‌ی ریسمان است و هر اجرا عوض می‌شود.)

### گرم‌کردن

بدونِ تایپ. جواب‌ها زیرِ هر پرسش است.

<details>
<summary>۱. کلاینتی درخواستی می‌فرستد که ۵ ثانیه طول می‌کشد. یک ثانیه بعد future خاموشی تمام می‌شود. کلاینت چه می‌گیرد و <code>serve</code> کی برمی‌گردد؟</summary>

**جواب.** کلاینت بعد از ۴ ثانیه‌ی باقی‌مانده پاسخِ عادیِ خودش را می‌گیرد؛ `serve` درست بعد از آن پاسخ `Ok(())` برمی‌گرداند. خاموشیِ آرام *پذیرفتن* را متوقف می‌کند؛ درخواست‌هایِ در حالِ اجرا را لغو نمی‌کند.

</details>

<details>
<summary>۲. همان وضعیت، اما کلاینتِ دومی ۲ ثانیه بعد از تمام‌شدنِ future خاموشی وصل می‌شود. چه می‌شود؟</summary>

**جواب.** اتصالش رد می‌شود، چون شنونده با تمام‌شدنِ future دراپ شده بود. (اگر در `shutdown_future` مقدارِ `grace` بزرگ‌تر از صفر باشد، شنونده به‌همان‌اندازه باز می‌ماند، چون تمام‌شدن به تأخیر می‌افتد.)

</details>

<details>
<summary>۳. دیتابیس در دسترس نیست اما پردازش سالم اجرا می‌شود. کدام‌یک از <code>/health</code> و <code>/ready</code> باید <code>503</code> بدهد، و چرا آن یکی نه؟</summary>

**جواب.** `/ready`. نمونه نباید ترافیک بگیرد، اما ری‌استارت کمکی نمی‌کند، و شکستِ `/health` پلتفرم را در یک حلقه‌ی ری‌استارت می‌اندازد.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن: هرکدام را با `--features broken` بساز، خطا را با بخشِ «خطاهایی که خواهی دید» بالا مقایسه کن، و فایل را ویرایش کن تا با `cargo build -p p3-04-03-graceful-shutdown-health-readiness --example <name> --features broken` ساخته شود.

- `examples/04-fn-item-not-a-future-broken.rs`
- `examples/05-async-block-borrows-state-broken.rs`

### پیاده‌سازی

در `src/lib.rs`، `Readiness::from_flags`، `Readiness::status`، `Readiness::body` و `async fn shutdown_future` را پیاده کن. کامنتِ مستندِ هرکدام مشخصاتِ کاملش است. تست‌ها: چهار تستِ داخلِ `src/lib.rs`.

```sh
cargo test -p p3-04-03-graceful-shutdown-health-readiness --lib
```

### بساز

`app` (روتر با `/health` و `/ready`) و `run` (سرو روی یک listener و توقفِ آرام وقتی `shutdown_future` تمام شود) را پیاده کن. تست‌ها: `tests/server_test.rs`، که سرورهایِ واقعی رویِ `127.0.0.1:0` بالا می‌آورد و بررسی می‌کند درخواستِ در حالِ اجرا تمام می‌شود، بعدش اتصالِ تازه رد می‌شود، `/ready` در حینِ تخلیه `503` می‌شود در حالی که `/health` `200` می‌ماند، و یک اتصالِ بیکارِ keep-alive بسته می‌شود.

```sh
cargo test -p p3-04-03-graceful-shutdown-health-readiness --test server_test
```

### چالش (اختیاری)

`run_with_deadline`: درخواستی که هرگز تمام نشود `run` را تا ابد منتظر نگه می‌دارد، و `SIGKILL`ِ پلتفرم هرطور شده می‌رسد. به‌جایش بعد از یک ضرب‌الاجل با خطای `TimedOut` تسلیم شو. تست‌ها: `tests/challenge_test.rs`. از `tokio::select!` استفاده می‌کند و به ایده‌ای که در ماژولِ ۳.۸ دوباره می‌آید اشاره دارد: اینکه خاموشی را هم باید رصد و لاگ کرد.

```sh
cargo test -p p3-04-03-graceful-shutdown-health-readiness --test challenge_test
```

یک راه‌حلِ نمونه، با استدلالش، در [`solution/SOLUTION.fa.md`](solution/SOLUTION.fa.md) است.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کار می‌آید |
|---|---|---|
| خاموشیِ آرام (graceful shutdown) | پذیرفتن را متوقف کن، بگذار درخواست‌هایِ در حالِ اجرا تمام شوند، بعد خارج شو | هر سرویسِ دیپلوی‌شده |
| future خاموشی (shutdown future) | futureای که `with_graceful_shutdown` منتظرش می‌ماند | تست‌ها تزریقش می‌کنند؛ `main` سیگنالِ سیستم‌عامل را می‌دهد |
| زنده‌بودن (liveness، `/health`) | آیا پردازش زنده است؟ شکستش پردازش را ری‌استارت می‌کند | پروب‌هایِ پلتفرم |
| آمادگی (readiness، `/ready`) | آیا باید ترافیک بگیرد؟ شکستش فقط از استخر بیرونش می‌کشد | لودبالانسرها؛ ماژولِ ۳.۵ بررسیِ دیتابیس را اضافه می‌کند |
| تخلیه (draining) | فاصله‌ی بینِ «خاموشی شروع شد» و «شنونده بسته شد» | مهلتِ آرامش در `shutdown_future` |
| `SIGTERM` | سیگنالِ یونیکسی که دیپلوی‌ها اول می‌فرستند؛ ویندوز اینجا معادلی ندارد | `tokio::signal::unix` |

### الان می‌دانی

- `with_graceful_shutdown` اتصال‌هایِ تازه را فوراً رد می‌کند، می‌گذارد درخواست‌هایِ در حالِ اجرا تمام شوند، اتصال‌هایِ بیکارِ keep-alive را می‌بندد، و بعد `Ok(())` برمی‌گرداند.
- راه‌انداز یک future معمولی است، پس سرورِ واقعی سیگنالِ سیستم‌عامل را می‌دهد و یک تست گیرنده‌ی `oneshot` را.
- `/health` و `/ready` به پرسش‌هایِ متفاوت جواب می‌دهند، و نمونه‌ی در حالِ تخلیه باید فقط در دومی شکست بخورد.
- اگر `.await.ok()` بنویسی، یک فرستنده‌ی `oneshot`ِ دراپ‌شده می‌تواند بی‌صدا خاموشی را راه بیندازد.

### بعداً کامل‌تر می‌بینی

- آمادگی در [۳.۵.۱ — اتصال و pooling](../../05-postgres-and-sqlx/01-connecting-and-pooling/README.fa.md) وابستگیِ واقعی می‌گیرد: `dependency_up` می‌شود «آیا می‌توانم از استخر اتصال بگیرم».
- لاگ‌کردنِ خاموشی (کی شروع شد، منتظرِ چند درخواست ماند) به درس‌هایِ tracing در [ماژولِ ۳.۸](../../08-error-handling-and-testing-at-scale/README.fa.md) برمی‌گردد.
- حالتِ مشترکی که اینجا پشتِ `Arc` گذاشتی در [۳.۴.۲](../02-app-state-and-dependency-wiring/README.fa.md) طراحی شد؛ تنظیمات (مهلتِ آرامش، پورت) از [۳.۴.۱](../01-config-and-secrets/README.fa.md) می‌آیند.

### می‌توانی توضیح بدهی؟

- با صدایِ بلند توضیح بده وقتی `SIGTERM` می‌رسد برایِ درخواستی که نیمه‌کاره است، و برایِ کلاینتی که درست بعدش وصل می‌شود، قدم‌به‌قدم چه می‌شود.
- چرا دیتابیسِ خراب `/ready` را شکست می‌دهد و `/health` را نه؟
- چرا `shutdown_future` راه‌انداز را به‌عنوانِ پارامتر می‌گیرد و خودش `tokio::signal::ctrl_c()` را صدا نمی‌زند؟
- نمونه‌ی در حالِ تخلیه پیش از بسته‌شدنِ شنونده منتظرِ چه می‌ماند، و آن انتظار چرا هست؟

---

## بیشتر

- [`axum::serve::WithGracefulShutdown`](https://docs.rs/axum/0.8.9/axum/serve/struct.WithGracefulShutdown.html)، نوعی که `with_graceful_shutdown` برمی‌گرداند.
- [`tokio::signal`](https://docs.rs/tokio/latest/tokio/signal/index.html): `ctrl_c`، به‌علاوه‌ی مجموعه سیگنال‌هایِ یونیکس و ویندوز.
- [Kubernetes: پیکربندیِ پروب‌هایِ liveness، readiness و startup](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)، مصرف‌کننده‌ی معمولِ این دو endpoint.
- [Gunicorn: مدیریتِ سیگنال](https://docs.gunicorn.org/en/stable/signals.html)، نسخه‌ی پایتونیِ همین قرارداد.
