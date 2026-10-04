# راه‌حل — ۳.۴.۳ خاموشیِ آرام، health و readiness

کدِ کامل `solution/src/lib.rs` است؛ همه‌ی تست‌هایِ `solution/tests/` و تست‌هایِ واحدِ داخلِ `lib.rs` را پاس می‌کند.

## `Readiness`

```rust
pub fn from_flags(draining: bool, dependency_up: bool) -> Readiness {
    if draining {
        Readiness::Draining
    } else if !dependency_up {
        Readiness::DependencyDown
    } else {
        Readiness::Ready
    }
}
```

ترتیبِ شاخه‌ها خودِ قاعده است: اول `draining` بررسی می‌شود، پس نمونه‌ی در حالِ تخلیه‌ای که وابستگی‌اش هم خراب است `Draining` گزارش می‌دهد (`draining_beats_a_down_dependency`). `status` یک `match` است که `Ready` را `200 OK` و دو حالتِ دیگر را `503 Service Unavailable` می‌کند؛ `body` یک `match` است که سه رشته‌ی دقیق را برمی‌گرداند. حکم یک enum با سه واریانت است و `bool` نیست چون *دلیلِ* `503` ارزشِ نوشتن در بدنه را دارد: اپراتوری که `draining` می‌خواند و اپراتوری که `dependency down` می‌خواند کارِ بعدیِ متفاوتی می‌کنند.

## `shutdown_future`

```rust
pub async fn shutdown_future(
    trigger: impl Future<Output = ()>,
    lifecycle: Arc<Lifecycle>,
    grace: Duration,
) {
    trigger.await;
    lifecycle.start_draining();
    tokio::time::sleep(grace).await;
}
```

سه گام به همان ترتیبِ مشخصات. تا `trigger` تمام نشود هیچ اتفاقی نمی‌افتد. `start_draining` همان لحظه اجرا می‌شود، پس `/ready` فوراً `503` می‌شود. خوابِ بعدش مهلتِ آرامش است: `axum` تا وقتی این future تمام نشود شنونده را نمی‌بندد، پس شنونده در این مدت باز می‌ماند و لودبالانسر هنوز می‌تواند `503` را ببیند. تستِ واحد از `#[tokio::test(start_paused = true)]` استفاده می‌کند، ساعتی که فقط وقتی همه‌ی تسک‌ها بیکارند جلو می‌رود، پس مهلتِ «۱۰ ثانیه‌ای» هیچ زمانِ واقعی نمی‌برد.

## `app`

```rust
pub fn app(lifecycle: Arc<Lifecycle>) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/ready", get(ready))
        .with_state(lifecycle)
}

async fn ready(State(lifecycle): State<Arc<Lifecycle>>) -> (StatusCode, &'static str) {
    let readiness = lifecycle.readiness();
    (readiness.status(), readiness.body())
}
```

`/health` هرگز به lifecycle نگاه نمی‌کند: زنده‌بودن یعنی «می‌توانم جواب بدهم»، و جواب‌دادن خودش دلیل است. `/ready` حالت را در هر درخواست می‌خواند، پس تغییرِ پرچم‌ها در پروبِ بعدی دیده می‌شود. توپلِ `(StatusCode, &str)` از قبل `IntoResponse` دارد.

## `run`

```rust
pub async fn run(listener: TcpListener, router: Router, lifecycle: Arc<Lifecycle>,
                 trigger: impl Future<Output = ()> + Send + 'static, grace: Duration) -> io::Result<()> {
    let shutdown = shutdown_future(trigger, lifecycle, grace);
    axum::serve(listener, router).with_graceful_shutdown(shutdown).await
}
```

`shutdown_future(...)` *صدا زده‌شده* داده می‌شود، نه با اسم (همان `E0277`ِ درس). قیدهایِ `Send + 'static` رویِ `trigger` هست چون `with_graceful_shutdown` آن را از کلِ future می‌خواهد و این future خودِ `trigger` را در بر دارد. `.await` وقتی تمام می‌شود که future خاموشی تمام شده باشد و همه‌ی درخواست‌هایِ در حالِ اجرا پایان یافته باشند.

## چالش، `run_with_deadline`

```rust
let (drain_started, drain_started_rx) = oneshot::channel::<()>();
let shutdown = async move {
    shutdown_future(trigger, lifecycle, grace).await;
    let _ = drain_started.send(());
};
let serve = axum::serve(listener, router).with_graceful_shutdown(shutdown);
tokio::select! {
    result = serve => result,
    _ = async { let _ = drain_started_rx.await; tokio::time::sleep(deadline).await; }
        => Err(io::ErrorKind::TimedOut.into()),
}
```

ضرب‌الاجل باید از لحظه‌ای شروع شود که future خاموشی *تمام می‌شود*، نه از شروعِ سرور، پس یک `oneshot` آن لحظه را از درونِ future خاموشی به شاخه‌ی دومِ `select!` می‌رساند. آن شاخه منتظرِ سیگنال می‌ماند، بعد `deadline` می‌خوابد. اگر `serve` اول تمام شود نتیجه‌ی آن برنده است و خواب دراپ می‌شود؛ اگر خواب اول تمام شود، `select!` مقدارِ `serve` را دراپ می‌کند، که اتصال‌هایِ گیرکرده را می‌بندد، و فراخواننده `TimedOut` می‌بیند. `let _ = drain_started.send(())` خطایی را که گیرنده‌ی دراپ‌شده می‌دهد نادیده می‌گیرد: آن فقط وقتی پیش می‌آید که `serve` از قبل برنده شده باشد.

## چرا تست‌ها ناپایدار نیستند

- هر انتظار «تا وقتی چیزی دیدنی رخ دهد» است، نه یک مدتِ حدسی. جفتِ `Notify` باعث می‌شود هندلرِ کند بگوید کی شروع شد و منتظر بماند تا به او بگویند می‌تواند تمام کند، پس «درخواست در حالِ اجرا، بعد خاموشی، بعد آزادسازی» تنها ترتیبِ ممکن است.
- پورت‌ها `127.0.0.1:0` هستند، پس تست‌هایِ موازی نمی‌توانند با هم برخورد کنند.
- `ready_turns_503_while_draining_but_health_stays_200` عمداً یک مهلتِ آرامشِ ۶۰ ثانیه‌ای دارد، تا شنونده برایِ پروب‌ها باز بماند؛ تست هرگز منتظرش نمی‌ماند، چون وقتی تست تمام می‌شود runtime تسکِ سرور را دراپ می‌کند.
- یک `connect`ِ ردشده رویِ ویندوز حدودِ دو ثانیه طول می‌کشد، پس `in_flight_request_finishes_and_new_connections_are_refused` تقریباً همین‌قدر طول می‌کشد. مجموعه‌ی تست‌ها سه بار پشتِ‌سرِ هم اجرا شد و هر بار سبز بود.
