# ۰۲.۱ — routing، handler و extractor

## کاری که ماژول یک دستی کرد، حالا کار library است

ماژول یک حلقه‌ی `TcpListener`، خواندن خط‌به‌خط request و parser HTTP را دستی ساخت. `axum` که روی `tokio` و `hyper` بنا شده همه را برایت انجام می‌دهد: اتصال را می‌پذیرد، request را می‌خواند و parse می‌کند، path را به function مناسب می‌رساند و خروجی را HTTP response می‌کند. سهم تو همان بخش Django است: **routeها و viewها** که اینجا handler نام دارند.

| Django / DRF | `axum` |
|---|---|
| `urls.py` و `path(...)` | `Router::new().route("/greet/{name}", get(greet))` |
| `def greet(request, name): ...` | `async fn greet(Path(name): Path<String>) -> String` |
| serializer روی `request.data` | extractor مانند `Json<EchoRequest>` |
| `request.GET["q"]` و URL kwargs | `Path<String>` و `Query<...>` |
| `MIDDLEWARE` | `tower::Layer` دور `Router` |

## handlerها تابع async با parameter نوع‌دارند

```rust
async fn greet(Path(name): Path<String>) -> String {
    format!("Hello, {name}!")
}
```

دو نکته مهم است:

۱. هر handler روی runtime async مربوط به Tokio اجرا می‌شود. هنوز بدنه‌ی این درس `.await` ندارد، اما `axum` handler را async می‌خواهد. از ماژول چهار، query database را await می‌کنی.
۲. فهرست parameterها خود request است، به‌شکل destructureشده بر اساس type. `axum` برای هر type، `FromRequest` یا `FromRequestParts` را صدا می‌زند: `Path<String>` segment URL، `Json<T>` body JSON و `State<S>` state متصل به router را بیرون می‌کشد. dictionary دستی `request.GET[...]` نداری؛ نیازت را در signature اعلام می‌کنی و داده یا از پیش parse‌شده تحویل می‌گیری یا request پیش از ورود به body رد می‌شود.

## extractorهای `Path`، `Json` و `State`

- `Path<T>` segmentهای `{name}` route را به `T` parse می‌کند؛ برای چند segment می‌تواند tuple یا struct باشد.
- `Json<T>` body را با `serde::Deserialize` به `T` تبدیل می‌کند. برگرداندن `Json<T>` عمل معکوس است: serialization و header درست `Content-Type` خودکار می‌شود.
- `State<S>` clone مقداری را می‌گیرد که هنگام ساخت router به `Router::with_state(s)` داده‌ای. connection pool، store درون‌حافظه‌ای و config مشترک را بدون global mutable در اختیار handler می‌گذارد.

extractorها به ترتیب signature اجرا می‌شوند. اگر یکی شکست بخورد—`Path<i64>` با text نامعتبر یا `Json<T>` غیرقابل-deserialize—`axum` پیش از اجرای body پاسخ error می‌دهد. request نیمه‌parse‌شده هرگز وارد تابع نمی‌شود؛ برخلاف DRF که باید یادت باشد `serializer.is_valid()` را صدا بزنی.

```senpai-visual
{"kind":"network","labels":["HTTP request","Path / Json / State","handler","Router","HTTP response"]}
```

## ساختن `Router`

```rust
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(hello))
        .route("/greet/{name}", get(greet))
        .route("/echo", post(echo))
        .route("/counter", get(get_counter))
        .route("/counter/increment", post(increment_counter))
        .with_state(state)
}
```

هر `.route(path, method_router)` یک path و methodهایش را ثبت می‌کند و `.with_state` state مشترک را به همه‌ی `State<AppState>`ها می‌دهد.

## test بدون socket واقعی

`axum::Router` trait به نام `tower::Service` را پیاده می‌کند. با `tower::ServiceExt::oneshot(request)` کل مسیر routing، extraction، handler و serialization را در test اجرا می‌کنی؛ نه port bind می‌شود و نه network واقعی داریم. همان قاعده‌ی «I/O را به لبه ببر و core را testable نگه دار» است.

مثل نگهبان ورودی مترو است که بلیت را پیش از ورود بررسی می‌کند؛ handler فقط مسافر معتبر را می‌بیند. مرز تشبیه: extractorها می‌توانند body را مصرف کنند، پس ترتیبشان فقط ظاهری نیست.

## تمرین تو

`greet`، `echo`، خواندن و افزایش counter و تابع `app` را کامل کن. `main.rs` از قبل app را روی `127.0.0.1:3000` با Tokio serve می‌کند.

```sh
cargo run -p p3-02-01-routing-handlers-extractors &
curl http://127.0.0.1:3000/greet/senpai
curl -X POST -H 'content-type: application/json' -d '{"message":"hi"}' http://127.0.0.1:3000/echo
```

## ایست بازرسی

پس از تمرین `CHECKPOINT.fa.md` و سپس `solution/SOLUTION.fa.md` را بخوان.
