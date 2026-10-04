# ۳.۴.۱ — پیکربندیِ ۱۲فاکتوری و سکرت‌ها

## در یک نگاه

بعد از این درس می‌توانی:

- قاعده‌ی دوازده‌فاکتوری «پیکربندی در محیط زندگی می‌کند» را توضیح بدهی و بگویی کدام تنظیم در کد، کدام در فایل و کدام در متغیرِ محیطی می‌نشیند.
- سه لایه (پیش‌فرض‌ها، یک فایلِ TOML، متغیرهایِ محیطی) را در یک `Config`ِ تایپ‌دار روی هم بچینی، به‌طوری‌که بالاترین لایه فیلد به فیلد برنده شود.
- یک تنظیمِ گم‌شده یا با نوعِ غلط را هنگامِ شروع به‌صورتِ یک `ConfigError`ِ تایپ‌دار گزارش بدهی، بدونِ `unwrap` و بدونِ پنیک.
- یک رمز را در `secrecy::SecretString` نگه داری تا `{:?}`، یک خطِ لاگ یا پیامِ پنیک نتواند آن را چاپ کند.
- پیکربندی را بدونِ دست‌زدن به محیطِ واقعی تست کنی، به این شکل که محیط را به‌صورتِ یک `HashMap` پارامتر بدهی.

**زمان:** حدود ۷۵ دقیقه · **پیش‌نیازها:**
[۳.۲.۵ — CORS و اتصال به فرانت‌اند](../../02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.fa.md)،
[۳.۲.۳ — عملیاتِ CRUD روی کاتالوگِ انیمه (در حافظه)](../../02-axum-and-rest-api-design/03-anime-catalog-crud-in-memory/README.fa.md)

---

## چرا اهمیت دارد

هر سروری که تا اینجا در این فاز ساختی تنظیماتش را داخلِ سورس نوشته بود: `127.0.0.1:3002`، `"http://localhost:5173"`. این روی لپ‌تاپِ تو کار می‌کند و جای دیگر نه. همین بیلد باید روی ماشینِ تو، در staging و در production اجرا شود، با یک پورتِ متفاوت، یک مبدأِ فرانت‌اندِ متفاوت و یک رمزِ دیتابیسِ متفاوت در هرکدام. اگر این تفاوت‌ها در سورس باشند، برایِ هر محیط دوباره بیلد می‌کنی، و رمزِ production در git می‌نشیند.

نسخه‌ی جنگویِ این ماجرا `settings.py` است: می‌نویسی `SECRET_KEY = os.environ["SECRET_KEY"]` و `DEBUG = os.environ.get("DEBUG") == "1"`، و یک کلیدِ گم‌شده هنگامِ import شدنِ ماژول با `KeyError` می‌ترکد. جنگو همچنین بی‌سروصدا هر تنظیمی را که اسمش `PASSWORD` یا `SECRET` داشته باشد از صفحه‌ی خطایِ debug پنهان می‌کند، چون چاپ‌کردنِ تنظیمات دقیقاً همان راهی است که سکرت‌ها لو می‌روند. Rust هیچ `settings.py`ای ندارد. این درس نسخه‌ی کوچکِ آن را می‌سازد، و بخشِ Rustیِ ماجرا این است که نتیجه یک struct واقعی با نوع‌هایِ واقعی است که یک‌بار هنگامِ شروع اعتبارسنجی می‌شود.

[۳.۲.۵](../../02-axum-and-rest-api-design/05-cors-and-frontend-integration/README.fa.md) یک قول داده بود: مبدأهایِ مجازِ CORS یک رشته‌ی هاردکد بودند، و «مبدأهایی که از محیط می‌آیند» به اینجا موکول شد. تا آخرِ درس، `APP_ALLOWED_ORIGINS` همان `CorsLayer` را تغذیه می‌کند.

---

## مفهوم

### پیکربندیِ دوازده‌فاکتوری

[اپِ دوازده‌فاکتوری](https://12factor.net/config) یک چک‌لیست برایِ سرویس‌هایی است که تمیز دیپلوی می‌شوند. قاعده‌ی پیکربندیِ آن کوتاه است: هر چیزی که بینِ دیپلوی‌ها فرق می‌کند (پورت‌ها، URLها، اعتبارنامه‌ها) در **متغیرهایِ محیطی** نگه داشته می‌شود، نه در کد. آزمونی که می‌دهد این است: آیا می‌توانی همین الان ریپو را اپن‌سورس کنی بدونِ اینکه اعتبارنامه‌ای لو برود؟ اگر نه، پیکربندی به کد نشت کرده است.

سه نوع تنظیم، سه خانه:

| نوع | مثال | کجا زندگی می‌کند |
|---|---|---|
| بینِ دیپلوی‌ها هیچ‌وقت عوض نمی‌شود | جدولِ مسیرها، اسمِ فیلدهایِ جیسون | کد |
| گاهی عوض می‌شود، خواندنش بی‌خطر است | پورتِ پیش‌فرض، قالبِ لاگ | یک فایلِ پیکربندی، در git |
| در هر دیپلوی فرق می‌کند، یا سکرت است | رمزِ دیتابیس، مبدأهایِ مجاز | متغیرهایِ محیطی |

### لایه‌ها: پیش‌فرض < فایل < محیط

سرویس‌هایِ واقعی هر سه را هم‌زمان به‌کار می‌برند. اولویت از کم‌اختصاصی به پراختصاصی می‌رود: یک **پیش‌فرض** که در کد کامپایل شده، بعد یک **فایل**، بعد **محیط**، تا یک اپراتور بتواند یک تنظیم را برایِ یک دیپلوی بدونِ ویرایشِ فایل بازنویسی کند. هر لایه فقط بعضی از تنظیم‌ها را مشخص می‌کند، پس یک لایه یک struct است که هر فیلدش یک `Option` است:

```rust
#[derive(Debug, Default, Clone)]
pub struct Layer {
    pub port: Option<u16>,
    pub allowed_origins: Option<Vec<String>>,
    pub debug: Option<bool>,
    pub db_password: Option<SecretString>,
}
```

`None` یعنی «این لایه نظری ندارد». روی‌هم‌چیدنِ دو لایه یک `or`ِ فیلد به فیلد است: `self.port.or(lower.port)` اگر `self` مقداری دارد همان است، وگرنه مقدارِ `lower`. همین یک متد، `over`، کلِ سازوکارِ اولویت است.

```senpai-visual
{"kind":"concept","labels":["پیش‌فرض‌ها در کد","فایلِ config.toml","متغیرهایِ محیطیِ APP_","چیدنِ فیلد به فیلد","Config تایپ‌دار یا یک ConfigError"]}
```

بعد از چیدن، `finish` `None`هایِ باقی‌مانده را با پیش‌فرض پر می‌کند و نتیجه را به یک `Config` تبدیل می‌کند که فیلدهایش نوع‌هایِ ساده و غیرانتخابی‌اند. تنها تنظیمی که پیش‌فرض ندارد رمز است: سرویسی که بدونِ آن شروع می‌شود باید از شروع‌کردن امتناع کند.

### خطا، نه پنیک

`std::env::var("APP_PORT")` یک `Result<String, VarError>` برمی‌گرداند، و واکنشِ تنبل `.unwrap()` است. هنگامِ شروع این بدترین جا برایِ پنیک است: اپراتور `called Result::unwrap() on an Err value: NotPresent` را می‌بیند که نه اسمِ تنظیم را می‌گوید نه راهِ درست‌کردنش را («خطاهایی که خواهی دید» آن را نشان می‌دهد). پس هر شکست یک واریانتِ یک enum است که می‌گوید کدام تنظیم و چه مشکلی:

```rust
pub enum ConfigError {
    Missing { key: &'static str },
    Invalid { key: &'static str, value: String, expected: &'static str },
    File(String),
}
```

`main` خطا را با `Display` چاپ می‌کند و با کدِ غیرصفر خارج می‌شود. کلِ راهبردِ خطا همین است: پیکربندی یک‌بار، پیش از اینکه هر درخواستی سرو شود، خوانده می‌شود، پس یک مقدارِ بد باید فرایند را بلند و فوری متوقف کند.

### محیط فقط یک آرگومان است

خواندنِ `std::env::var` در دلِ کدِ پارس‌کننده آن را تست‌ناپذیر می‌کند: محیطِ فرایند سراسری است، بینِ تست‌هایی که موازی اجرا می‌شوند مشترک است، و تغییرش از نظرِ ریسمان‌ها ایمن نیست. به‌جایش پارس‌کردن محیط را پارامتر می‌گیرد:

```rust
pub fn from_env(env: &HashMap<String, String>) -> Result<Layer, ConfigError>
```

یک تست یک `HashMap` با دقیقاً متغیرهایی که می‌خواهد می‌سازد. فقط `main` به چیزِ واقعی دست می‌زند، با `std::env::vars().collect()`. این همان ایده‌ی دادنِ یک `Router` به `oneshot` به‌جایِ بایندکردنِ یک سوکت در [۳.۲.۱](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md) است: چیزِ سراسری را به لبه ببر، و همه‌چیزِ داخل یک تابعِ ساده می‌شود.

### سکرت‌هایی که نمی‌توانند لو بروند

طراحیِ بدیهی را امتحان کن: `db_password: String` در یک struct که `Debug` را derive می‌کند. اولین باری که کسی `println!("{config:?}")` یا `tracing::info!(?config)` بنویسد تا ببیند سرویس با چه چیزی بالا آمده، رمز در لاگ است. `examples/01-debug-leak.rs` همین struct را دو بار چاپ می‌کند:

```text
Leaky { port: 8080, db_password: "hunter2" }
Careful { port: 8080, db_password: SecretBox<str>([REDACTED]) }
on purpose: hunter2
```

`secrecy::SecretString` (کریتِ `secrecy` نسخه‌ی ۰٫۱۰، یک نامِ مستعار برایِ `SecretBox<str>`) متن را نگه می‌دارد، `Debug` را به‌شکلِ `[REDACTED]` پیاده می‌کند، و نه `Display` دارد نه `Serialize`، پس `{}`ِ اشتباه یک خطای کامپایل است نه یک نشت. برایِ خواندنِ مقدار باید `expose_secret()` را از صفتِ `ExposeSecret` صدا بزنی:

```rust
use secrecy::ExposeSecret;
let url = format!("postgres://app:{}@db/anime", config.db_password.expose_secret());
```

دو نتیجه می‌گیریم. اسمِ متد ردِ بازرسی است: `grep expose_secret` هر جایی را که سکرت می‌تواند از آن بیرون برود فهرست می‌کند. و حافظه هنگامِ drop شدنِ مقدار صفر می‌شود، که یک امتیازِ اضافه است نه اصلِ ماجرا. اصل این است که رفتارِ ایمن پیش‌فرض است و رفتارِ ناایمن یک صدازدنِ آگاهانه و قابلِ جست‌وجو می‌خواهد. توجه کن که secrecy از سکرتی که خودت کپی کنی محافظت نمی‌کند: `expose_secret().to_string()` در یک خطِ لاگ هنوز نشت است.

### لایه‌ی فایل، و غلطِ املایی

لایه‌ی فایل از `toml` و derive در `serde` استفاده می‌کند، همان‌طور که جیسون را پارس کردی. یک اتریبیوت جایش را ثابت می‌کند: `#[serde(deny_unknown_fields)]` یک کلیدِ غلط‌نوشته را به‌جایِ یک خطِ بی‌صدا نادیده‌گرفته‌شده به خطا تبدیل می‌کند:

```text
Err(File("TOML parse error at line 1, column 1\n  |\n1 | prot = 7000\n  | ^^^^\nunknown field `prot`, expected one of `port`, `allowed_origins`, `debug`, `db_password`\n"))
```

بدونِ آن، `prot = 7000` پذیرفته می‌شد، `port`ِ واقعی روی پیش‌فرض می‌ماند، و تو یک شب را سپری می‌کردی تا بفهمی چرا تنظیمت هیچ اثری ندارد.

---

## دست‌به‌کد

دو مثال روی کدی که به تو داده شده کار می‌کنند:

```sh
cargo run -p p3-04-01-config-and-secrets --example 01-debug-leak
cargo run -p p3-04-01-config-and-secrets --example 02-file-layer
```

اولی همان سه خطِ بالا را چاپ می‌کند. دومی با `Layer::from_toml`ِ داده‌شده سه تکه TOML را پارس می‌کند، یکی درست، یکی با غلطِ املایی، یکی با نوعِ غلط:

```text
Ok(Layer { port: Some(7000), allowed_origins: Some(["https://anime.example.com"]), debug: None, db_password: None })
Err(File("TOML parse error at line 1, column 1\n  |\n1 | prot = 7000\n  | ^^^^\nunknown field `prot`, expected one of `port`, `allowed_origins`, `debug`, `db_password`\n"))
Err(File("TOML parse error at line 1, column 8\n  |\n1 | port = \"seven thousand\"\n  |        ^^^^^^^^^^^^^^^^\ninvalid type: string \"seven thousand\", expected u16\n"))
```

(تا وقتی تمرین‌ها را تمام نکرده‌ای، `cargo` برایِ تابع‌هایی که هنوز ننوشته‌ای هشدارِ `unused variable` هم چاپ می‌کند. نادیده‌شان بگیر.)

دو مثالِ بعدی به پله‌هایِ پیاده‌سازی و بساز نیاز دارند، چون تابع‌هایِ تو را صدا می‌زنند. این‌ها خروجیِ آن‌ها با راه‌حلِ کامل است. `03-load-layers` یک فایل و چند محیطِ ساختگی را بارگذاری می‌کند، بدونِ هیچ محیطِ واقعی:

```sh
cargo run -p p3-04-01-config-and-secrets --example 03-load-layers
```

```text
Ok(Config { port: 7000, allowed_origins: ["http://localhost:5173"], debug: true, db_password: SecretBox<str>([REDACTED]) })
Ok(Config { port: 9000, allowed_origins: ["http://localhost:5173"], debug: true, db_password: SecretBox<str>([REDACTED]) })
Err(Missing { key: "db_password" })
Err(Invalid { key: "APP_PORT", value: "eighty", expected: "a port number from 0 to 65535" })
missing required setting `db_password`
```

خطِ ۱ فقط فایل است، خطِ ۲ `APP_PORT=9000` را اضافه می‌کند و از `7000`ِ فایل جلو می‌زند، و `allowed_origins` پیش‌فرض است چون کسی آن را تنظیم نکرده. `04-serve-from-env` چیزِ واقعی است: محیطِ واقعی، یک پورتِ واقعی، و `CorsLayer`ِ ۳.۲.۵ که از پیکربندی تغذیه می‌شود. آن را روی پورتِ ۳۱۴۰ در یک ترمینال بالا بیاور و بگذار بماند:

```sh
APP_PORT=3140 APP_DB_PASSWORD=hunter2 APP_ALLOWED_ORIGINS=https://anime.example.com \
  cargo run -p p3-04-01-config-and-secrets --example 04-serve-from-env
```

```text
Config { port: 3140, allowed_origins: ["https://anime.example.com"], debug: false, db_password: SecretBox<str>([REDACTED]) }
listening on http://127.0.0.1:3140
```

در ترمینالِ دوم یک درخواست از مبدأِ تنظیم‌شده بفرست، و بعد یکی از مبدأیی دیگر:

```sh
curl -si http://127.0.0.1:3140/ -H "Origin: https://anime.example.com"
curl -si http://127.0.0.1:3140/ -H "Origin: http://localhost:5173"
```

```text
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
vary: origin, access-control-request-method, access-control-request-headers
access-control-allow-origin: https://anime.example.com
content-length: 3
date: Sun, 04 Oct 2026 11:14:04 GMT

ok
```

درخواستِ دوم همان `200` و `ok` را می‌گیرد ولی بدونِ خطِ `access-control-allow-origin`، دقیقاً همان رفتارِ «خودداری از ضمانت» در ۳.۲.۵، که حالا یک متغیرِ محیطی آن را می‌راند. (`date` هر بار عوض می‌شود.) سرور را با Ctrl+C متوقف کن. حالا آن را بدونِ رمز بالا بیاور، و بعد با یک پورتِ بد:

```text
cannot start: missing required setting `db_password`
cannot start: invalid value "eighty" for APP_PORT: expected a port number from 0 to 65535
```

هر دو پیش از بایندشدنِ هر سوکتی با کدِ ۱ خارج می‌شوند. بعد این‌ها را امتحان کن:

۱. یک `config.toml` کنارِ جایی که مثال را اجرا می‌کنی بساز با `port = 3142` و `db_password = "from-file"`، و بدونِ هیچ محیطی اجرایش کن. از کدام پورت استفاده می‌کند؟ حالا `APP_PORT=3143` را اضافه کن.
۲. آن را با `APP_DEBUG=yes` اجرا کن. خطا می‌گوید مقدارهایِ پذیرفته کدام‌اند؟

---

## خطاهایی که خواهی دید

### یک پنیکِ زمانِ اجرا: `unwrap` رویِ متغیرِ گم‌شده

```text
thread 'main' (42364) panicked at phase3-backend-foundations\04-configuration-and-app-structure\01-config-and-secrets\examples\05-unwrap-env-broken.rs:10:47:
called `Result::unwrap()` on an `Err` value: NotPresent
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(عددِ داخلِ پرانتز شناسه‌ی ریسمان است و هر بار عوض می‌شود.)

**مشکلِ واقعی چیست:** `std::env::var("APP_PORT").unwrap()` فرض می‌کند متغیر تنظیم شده است. نشده، پس برنامه پنیک می‌کند، و پیام نه `APP_PORT` را نام می‌برد نه می‌گوید چه باید کرد: `NotPresent` واریانتِ `VarError` است، و شماره‌ی خط به کدِ تو اشاره می‌کند نه به تنظیم. یک مقدارِ غلط یک `unwrap` بعد همین‌طور شکست می‌خورد، به‌شکلِ یک `ParseIntError`.

**راه‌حل:** یک خطای تایپ‌دار برگردان که اسمِ تنظیم را همراه دارد:

```rust
let raw = std::env::var("APP_PORT").map_err(|_| ConfigError::Missing { key: "APP_PORT" })?;
```

**چرا این راه‌حل است:** اپراتوری که لاگ را می‌خواند کلید و مشکل را لازم دارد، و `Display` رویِ `ConfigError` هر دو را در یک خط می‌دهد. در کدِ درس پارس‌کردن در `Layer::from_env` است، که اسم و مقدارِ اولین متغیرِ بد را در خطا می‌گذارد.

### E0277: چاپِ سکرت مثلِ یک `String`

```text
error[E0277]: `SecretBox<str>` doesn't implement `std::fmt::Display`
  --> phase3-backend-foundations\04-configuration-and-app-structure\01-config-and-secrets\examples\06-print-secret-broken.rs:11:45
   |
11 |     println!("connecting with password {}", password);
   |                                        --   ^^^^^^^^ `SecretBox<str>` cannot be formatted with the default formatter
   |                                        |
   |                                        required by this formatting parameter
   |
   = help: the trait `std::fmt::Display` is not implemented for `SecretBox<str>`
   = note: in format strings you may be able to use `{:?}` (or {:#?} for pretty-print) instead

For more information about this error, try `rustc --explain E0277`.
error: could not compile `p3-04-01-config-and-secrets` (example "06-print-secret-broken") due to 1 previous error
```

**کامپایلر به چه ایراد می‌گیرد:** `{}` به `Display` نیاز دارد، و `SecretString` عمداً آن را پیاده نمی‌کند. خطِ `help` پیشنهادِ `{:?}` می‌دهد، و این‌جا بی‌خطر است: `[REDACTED]` چاپ می‌کند.

**راه‌حل:** بیشترِ وقت اصلاً آن را چاپ نکن. اگر برنامه واقعاً متن را لازم دارد، مثلاً برایِ ساختنِ رشته‌ی اتصال، در کد بگو:

```rust
println!("connecting with password {}", password.expose_secret());
```

**چرا این راه‌حل است:** خطای کامپایل خودِ قابلیت است. یک `String` اجازه می‌دهد نشت در هر `{}`ای اتفاقی رخ بدهد؛ `SecretString` نشت را به یک صدازدنِ عمدی تبدیل می‌کند که خودت نوشته‌ای و با `grep` پیدایش می‌کنی.

### هیچ خطایی نیست: متغیرِ خالی که همه را بیرون می‌اندازد

```text
Config { port: 3141, allowed_origins: [], debug: false, db_password: SecretBox<str>([REDACTED]) }
listening on http://127.0.0.1:3141
HTTP/1.1 200 OK
content-type: text/plain; charset=utf-8
vary: origin, access-control-request-method, access-control-request-headers
content-length: 3
date: Sun, 04 Oct 2026 11:14:54 GMT

ok
```

**مشکلِ واقعی چیست:** سرور با `APP_ALLOWED_ORIGINS=` (تنظیم‌شده، ولی خالی) بالا آمد. متغیرِ حاضر ولی خالی هنوز حاضر است، پس پیش‌فرض را بازنویسی می‌کند و فهرستِ مبدأهایِ مجاز خالی می‌شود: از هیچ فرانت‌اندی ضمانت نمی‌شود، کدِ وضعیت `200` است، و تنها علامت یک کنسولِ مرورگرِ پر از خطایِ CORS است. این دقیقاً همان شکستِ بی‌صدایِ مبدأیی است که در ۳.۲.۵ هیچ‌وقت جور نمی‌شد، این بار از راهِ پیکربندی.

**راه‌حل:** تصمیم بگیر مقدارِ خالی یعنی چه و آن را در نوع بگو. یا فهرستِ خالی را `Missing` حساب کن، یا مثلِ این درس آن را به‌عنوانِ «هیچ مبدأیی» بپذیر و `Config`ِ بارگذاری‌شده را هنگامِ شروع چاپ کن، همان‌طور که `main` این‌جا می‌کند، تا فهرستِ خالی در اولین خطِ لاگ دیده شود.

**چرا این راه‌حل است:** کامپایلر نمی‌تواند «عمداً هیچ» را از «فراموش کردم پرش کنم» تشخیص بدهد، ولی یک خطِ شروع که مقدارهایِ نهایی را نشان می‌دهد می‌تواند. چاپ‌کردنِ پیکربندی هنگامِ شروع فقط به این دلیل ایمن است که رمز یک `SecretString` است.

---

## تمرین

### گرم‌کردن

<details>
<summary>قاعده‌ی دوازده‌فاکتوری می‌گوید پیکربندی در متغیرهایِ محیطی بنشیند. چرا رمزِ دیتابیس را در یک فایلِ داخلِ ریپو نگذاریم؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

چون فایلِ داخلِ ریپو در git است، و تاریخچه‌ی git برای همیشه می‌ماند و هر کسی که دسترسی دارد آن را می‌خواند. رمز در هر دیپلوی فرق می‌کند و سکرت است، پس باید در لحظه‌ی شروعِ فرایند از بیرونِ کد بیاید. یک فایل می‌تواند پیش‌فرض‌هایِ غیرسکرت را نگه دارد، ولی مقدارهایِ مخصوصِ دیپلوی و سکرت در محیط می‌نشینند.

</details>

<details>
<summary>یک struct که <code>Debug</code> را derive می‌کند یک <code>db_password: String</code> دارد. بدونِ اینکه کسی رویش <code>println!</code> بنویسد رمز کجا می‌تواند برسد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

هر جایی که کلِ struct با `{:?}` قالب‌بندی شود: یک خطِ `tracing::info!(?config)`، یک `unwrap()` رویِ یک `Result` که آن را دارد، یک پیامِ پنیک، یک گزارشِ خطا که وضعیتِ برنامه را شامل می‌شود. هرکدام یک لاگ یا گزارشِ کرش است که افراد زیادی می‌توانند بخوانند.

</details>

<details>
<summary>فایل می‌گوید <code>port = 7000</code> و محیط <code>APP_PORT=9000</code> دارد. کدام برنده است، و کدام متد تصمیم می‌گیرد؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

`9000`. لایه‌ی محیط لایه‌ی با اولویتِ بالاتر است، و `env_layer.over(file_layer)` هر وقت `Some` باشد مقدارِ `self` را برمی‌دارد. فیلدی که محیط تنظیم نکرده، `None`، به مقدارِ فایل برمی‌گردد، و بعد به پیش‌فرض.

</details>

<details>
<summary>چرا <code>from_env</code> یک <code>&amp;HashMap</code> می‌گیرد به‌جایِ اینکه خودش <code>std::env::var</code> را صدا بزند؟</summary>

قبل از اینکه جواب را ببینی، خودت فکرش را بکن.

</details>

<details>
<summary>جواب</summary>

چون محیطِ واقعی سراسری است، بینِ همه‌ی ریسمان‌هایِ تست مشترک است، و تغییرش از نظرِ ریسمان‌ها ایمن نیست. با یک map به‌عنوانِ پارامتر هر تست محیطِ خودش را می‌سازد و هیچ‌چیز به فرایند دست نمی‌زند. فقط `main` محیطِ واقعی را می‌خواند، یک‌بار.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/05-unwrap-env-broken.rs` وقتی `APP_PORT` نیست یا عدد نیست پنیک نکند: یک خطِ خوانا چاپ کند که `APP_PORT` را نام ببرد و با کدِ ۱ خارج شود.
۲. `examples/06-print-secret-broken.rs` کامپایل شود و رمز را به‌شکلِ `[REDACTED]` چاپ کند، بدونِ صدازدنِ `expose_secret`.

### پیاده‌سازی

دو تابعِ کوچک در `src/lib.rs`، که هرکدام کاملاً با کامنتِ مستنداتش مشخص شده:

```sh
cargo test -p p3-04-01-config-and-secrets
```

- `parse_bool`: نوشتارهایِ پذیرفته‌شده‌ی یک تنظیمِ بولی، و خطا برایِ هر چیزِ دیگر.
- `parse_origins`: یک رشته‌ی جداشده با ویرگول به یک فهرستِ تمیزِ مبدأها.

تست‌هایِ `todo!()` در همین فایل تا وقتی این دو تمام نشده‌اند شکست می‌خورند؛ سه تستِ داخلِ `tests/` از همان اول پاس می‌شوند، چون فقط از کدِ داده‌شده استفاده می‌کنند.

### بساز

خطِ لوله، در چهار تکه. هرکدام یک کامنتِ مستندات دارد که دقیقاً می‌گوید چه می‌کند:

- `Layer::from_env`: چهار متغیرِ `APP_` به یک `Layer`، با خطاهایِ تایپ‌دار.
- `Layer::over`: روی‌هم‌چیدنِ دو لایه.
- `Layer::finish`: پیش‌فرض‌ها، و تنها تنظیمِ الزامی.
- `Config::load`: کلِ خطِ لوله در یک صدازدن. وقتی هر چهار پاس شدند، `03-load-layers` و `04-serve-from-env` را اجرا کن.

### چالش (اختیاری)

دو مبدأ که فقط در یک اسلشِ آخر فرق دارند هیچ‌وقت جور نمی‌شوند، همان‌طور که در ۳.۲.۵ دیدی. `finish` را طوری کن که یک مدخلِ `allowed_origins` که به `/` ختم می‌شود را رد کند، با `ConfigError::Invalid { key: "APP_ALLOWED_ORIGINS", .. }` که آن را نام ببرد. تستش را در یک فایلِ جدید زیرِ `tests/` بنویس. بعد فکر کن: آیا `finish` جایِ درستش است، یا `from_env` و `from_toml` باید هرکدام این کار را بکنند؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| پیکربندیِ دوازده‌فاکتوری | تنظیم‌هایِ مخصوصِ هر دیپلوی در محیط‌اند، نه در کد | هر سرویسِ دیپلوی‌شده |
| پیکربندیِ لایه‌ای | پیش‌فرض < فایل < محیط، فیلد به فیلد ادغام می‌شود | `Layer::over` |
| `ConfigError` | یک شکستِ تایپ‌دار که تنظیم و مشکل را نام می‌برد | هنگامِ شروع، به‌جایِ `unwrap` |
| `SecretString` | رشته‌ای که `Debug`اش `[REDACTED]` است و `Display` ندارد | رمزها، توکن‌ها، کلیدهایِ API |
| `expose_secret()` | تنها صدازدنِ آگاهانه‌ای که یک سکرت را می‌خواند | ساختنِ رشته‌ی اتصال |
| `deny_unknown_fields` | اتریبیوتِ serde که غلطِ املایی را به خطا تبدیل می‌کند | هر فایلِ پیکربندی |

### الان می‌دانی

- پیکربندیِ مخصوصِ هر دیپلوی در محیط می‌نشیند؛ پیش‌فرض و مقدارهایِ فایل زیرِ آن‌اند، و بالاترین لایه فیلد به فیلد برنده می‌شود.
- یک تنظیمِ گم‌شده یا با نوعِ غلط یک خطای تایپ‌دار است که هنگامِ شروع پیدا می‌شود، هیچ‌وقت یک پنیک با `NotPresent`.
- محیط یک پارامتر است، پس تست‌ها محیطِ خودشان را می‌سازند و هیچ‌وقت به فرایند دست نمی‌زنند.
- `SecretString` کارِ ایمن را پیش‌فرض می‌کند: `{:?}` پنهان می‌کند، `{}` کامپایل نمی‌شود، و خواندن یک `expose_secret()`ِ قابلِ جست‌وجو است.
- یک متغیرِ خالی هنوز یک متغیر است، و `deny_unknown_fields` همان چیزی است که جلوی پنهان‌ماندنِ یک غلطِ املایی در فایل را می‌گیرد.

### بعداً کامل‌تر می‌بینی

- **گذاشتنِ `Config`ِ تمام‌شده جایی که هر هندلر به آن برسد** — [۳.۴.۲ — وضعیتِ اپلیکیشن و سیم‌کشیِ وابستگی‌ها](../02-app-state-and-dependency-wiring/README.fa.md)
- **فرایندی که تمیز متوقف می‌شود، و endpointِ `/health` که یک دیپلوی بازخوانی‌اش می‌کند** — [۳.۴.۳ — خاموشیِ آرام، health و readiness](../03-graceful-shutdown-health-readiness/README.fa.md)
- **آدرسِ دیتابیس و اندازه‌ی pool هم پیکربندی‌اند** — [ماژول ۵ — دیتابیس PostgreSQL و `sqlx`](../../05-postgres-and-sqlx/README.fa.md)
- **رمزی که اصلاً نباید ذخیره‌اش کنی، فقط هشِ آن** — [۳.۷.۱ — هش‌کردنِ پسورد با argon2](../../07-auth-and-security/01-password-hashing-argon2/README.fa.md)

بازگشت به ماژول: [فهرستِ ماژول ۴](../README.fa.md).

### می‌توانی توضیح بدهی؟

- سه نوع تنظیم کدام‌اند، و هرکدام کجا زندگی می‌کند؟
- چرا `over` یک `or`ِ فیلد به فیلد است و نه «کلِ لایه‌ی بالاتر را بردار اگر چیزی دارد»؟
- چرا `unwrap` رویِ `std::env::var` هنگامِ شروع ابزارِ غلطی است، حتی اگر «کار کند»؟
- چه چیزی جلوی چاپِ رمز با `{:?}` رویِ یک `Config` را می‌گیرد، و چه چیزی جلوی کامپایل‌شدنِ `{}` را؟
- `deny_unknown_fields` از چه چیزی محافظت می‌کند؟

---

## بیشتر

- [The Twelve-Factor App — III. Config](https://12factor.net/config): قاعده‌ی اصلی و استدلالش، در یک صفحه.
- [`secrecy` 0.10 on docs.rs](https://docs.rs/secrecy/0.10.3/secrecy/): `SecretBox`، `SecretString` و صفتِ `ExposeSecret`.
- [`toml` on docs.rs](https://docs.rs/toml/0.8/toml/): پارسرِ لایه‌ی فایل، و نوعِ خطایش.
- [فهرستِ ماژول ۴](../README.fa.md): سه درسِ این ماژول به ترتیب.
