# ۳.۱.۲ — پارسرِ دست‌سازِ HTTP

## در یک نگاه

بعد از این درس می‌توانی:

- چهار قاعده‌ی دقیقِ فرمتِ رویِ سیمِ یک درخواستِ HTTP/1.1 را توضیح بدهی — و اگر یکی‌شان (جداکننده‌ی `\r\n`) را اشتباه بزنی، دقیقاً بگویی چه خرابی‌ای تولید می‌شود.
- یک پارسر و یک سریالایزرِ HTTP را کاملاً با دست بنویسی: بایتِ خام می‌گیری، یک `Result<HttpRequest, HttpParseError>` پس می‌دهی، و هر جور بدشکلی‌ای گونه‌ی نام‌دارِ خودش را می‌گیرد، نه یک خطایِ عمومیِ یک‌دست.
- بگویی چرا `axum`، دو درسِ بعد، وقتی `Json<T>` یا `Path<T>` جور در نیاید یک ۴۰۰ خودکار برمی‌گرداند — چون امروز دقیقاً همین مسیر را خودت با دست رفتی.

**زمان:** حدود ۷۰ دقیقه · **پیش‌نیاز:**
[۳.۱.۱ — سرور اکوی TCP](../01-tcp-echo-server/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل بهت یک `TcpStream` داد — یک لوله‌ی بایتِ خام، بدونِ هیچ ساختاری. همان‌جا هم گفتیم که این دقیقاً همان چیزی است که سرورِ WSGI/ASGI جنگو زیرِ پوستش می‌بیند، پیش از آنکه یک آبجکتِ `request` تروتمیز با `.method` و `.path` و `.headers` به ویویِ تو تحویل بدهد. این درس همان فاصله را پر می‌کند: از یک `&[u8]`ِ خام تا یک `HttpRequest`ِ ساختاریافته، و برعکس.

نکته این نیست که HTTP پیچیده است — نیست. نکته این است که HTTP یک **پروتکلِ متنی و خط‌محور** است، دقیقاً به همان اندازه که یک فایلِ CSV یا یک فایلِ پیکربندی است، فقط رویِ یک سوکت به‌جایِ رویِ دیسک. هر چیزی که Django، DRF، gunicorn یا `axum` (دو درسِ بعد) به‌عنوانِ «جادو» به‌نظر می‌رسد، در واقع همین است: خواندنِ متن، تکه‌کردنش رویِ چند جداکننده‌ی ثابت، و مدیریتِ صادقانه‌ی هر جایی که متن می‌توانست بدشکل باشد. امروز این کار را یک‌بار، دستی، خودت انجام می‌دهی — تا از این به بعد دیگر برایت جادو نباشد.

---

## مفهوم

### شکلِ یک درخواستِ HTTP/1.1

یک درخواستِ HTTP/1.1 واقعی، رویِ سیم، دقیقاً همین شکل است:

```text
GET /anime?status=watching HTTP/1.1\r\n
Host: localhost:7879\r\n
User-Agent: curl/8.4.0\r\n
Accept: */*\r\n
\r\n
```

این دقیقاً همان چیزی است که به آن **فرمتِ رویِ سیم (wire format)** می‌گویند: شکلِ عینیِ بایت‌به‌بایتی که واقعاً رویِ شبکه فرستاده می‌شود، جدا از هر نوعِ Rustی‌ای که بعداً بازنمایی‌اش می‌کند. چهار نکته‌اش را دقیق ببین:

۱. **خط‌ها با `\r\n` از هم جدا می‌شوند** (carriage return + line feed)، نه فقط با `\n` — میراثی از پروتکل‌های متنیِ قدیمی‌تر، و یک تله‌ی واقعی اگر فراموشش کنی: اگر با دست فقط رویِ `\n` تکه کنی، آن `\r` رویِ چیزی که درست قبلش می‌آید باقی می‌ماند.
۲. **خطِ اول** («خطِ درخواست» یا request line) این شکل را دارد: `{METHOD} {PATH} {VERSION}` — سه توکنِ جداشده با یک فاصله.
۳. **هر خطِ هدر** این شکل را دارد: `Name: value` — یک دونقطه، بعد مقدار. اسمِ هدرها طبقِ اسپک حساس به حروف نیستند (`Host` و `host` دقیقاً یک هدرند) — کلاینت‌ها و پراکسی‌های واقعی رویِ همین حساب باز می‌کنند.
۴. **یک خطِ کاملاً خالی** (`\r\n\r\n` در انتها) پایانِ هدرها را نشان می‌دهد. برایِ یک `GET`، بعدش هیچ بدنه‌ای نیست — این درس عمداً همین‌جا متوقف می‌شود. بدنه‌ی `POST`/`PUT`، و هدرِ `Content-Length` که می‌گوید چند بایت برایِ آن باید خواند، کارِ [ماژولِ ۲ — `axum`](../../02-axum-and-rest-api-design/README.fa.md) است.

### از بایت به متن: چرا اول باید اعتبارسنجی کنی

خواندن از رویِ سوکت به تو `&[u8]` می‌دهد، نه `&str`. اولین کاری که هر پارسری باید بکند، تأیید این است که این بایت‌ها اصلاً UTF-8 معتبرند — یک کلاینت (یا یک مهاجم) می‌تواند هر بایتِ آشغالی که دلش بخواهد بفرستد:

```rust
fn bytes_off_a_socket() -> Vec<u8> {
    vec![0x47, 0x45, 0x54, 0xff, 0xfe]
}

let good: &[u8] = b"GET / HTTP/1.1";
match std::str::from_utf8(good) {
    Ok(text) => println!("valid:   {text:?}"),
    Err(e) => println!("invalid: {e}"),
}
let bad = bytes_off_a_socket();
match std::str::from_utf8(&bad) {
    Ok(text) => println!("valid:   {text:?}"),
    Err(e) => println!("invalid: {e}"),
}
```

```text
valid:   "GET / HTTP/1.1"
invalid: invalid utf-8 sequence of 1 bytes from index 3
```

`std::str::from_utf8` دقیقاً همین قول را می‌دهد: یا یک `&str` معتبر، یا یک `Err` که می‌گوید کجا خراب بود — نه یک پنیک، نه یک نادیده‌گرفتنِ ساکت. این اولین باری است در این فاز که `Result` رویِ داده‌ی واقعاً *نامعتمد* (نه یک ورودیِ فرضیِ تمرینی) کار می‌کند، و دقیقاً به همین دلیل هیچ‌جایِ دیگرِ این تابع اجازه ندارد فرض کند ورودی خوب است.

### هر جور بدشکلی، یک خطایِ نام‌دار

`parse_request` یک `Result<HttpRequest, HttpParseError>` برمی‌گرداند، و `HttpParseError` به‌جایِ یک `ParseFailed` عمومی، هر جور خرابی را گونه‌ی خودش می‌دهد:

```rust
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HttpParseError {
    #[error("request bytes are not valid UTF-8")]
    InvalidUtf8,
    #[error("request is empty")]
    EmptyRequest,
    #[error("malformed request line: {0:?}")]
    MalformedRequestLine(String),
    #[error("malformed header line: {0:?}")]
    MalformedHeaderLine(String),
}
```

این همان غریزه‌ای است که دیکشنریِ `.errors` سریالایزرِ DRF هم دنبالش می‌کند: به فراخواننده بگو *دقیقاً چه چیزی* خراب بود، نه فقط اینکه *یک چیزی* خراب بود — با این تفاوت که اینجا این کار را عرفِ برنامه‌نویسی انجام نمی‌دهد، خودِ سیستمِ نوع‌ها انجامش می‌دهد. کدی که این خطا را می‌گیرد می‌تواند رویِ گونه‌اش `match` بزند و دقیقاً بفهمد کدام مرحله شکست خورد.

```senpai-visual
{"kind":"result","labels":["parse_request(bytes)","Err(InvalidUtf8)","Err(MalformedRequestLine)","Err(MalformedHeaderLine)","Ok(HttpRequest)"]}
```

### نامِ هدرها حساسِ حروف نیستند

طبقِ اسپکِ HTTP، *اسمِ* هدرها بین حروفِ کوچک و بزرگ فرقی نمی‌گذارد. `str::eq_ignore_ascii_case` همین مقایسه را می‌کند، بدونِ اینکه لازم باشد یک نسخه‌ی کوچک‌شده از هیچ‌کدامِ دو طرف بسازی:

```rust
let sent_by_client = "Host";
for candidate in ["host", "HOST", "Host", "Content-Type"] {
    println!(
        "{candidate:?} matches {sent_by_client:?}: {}",
        candidate.eq_ignore_ascii_case(sent_by_client)
    );
}
```

```text
"host" matches "Host": true
"HOST" matches "Host": true
"Host" matches "Host": true
"Content-Type" matches "Host": false
```

این قاعده فقط شاملِ *اسمِ* هدرهاست، نه چیزِ دیگری. `Method`، برایِ نمونه، طبقِ اسپک حساسِ حروف است — `"get"` همان `Method::Get` نیست؛ همین حالا در `src/lib.rs` می‌بینیش، و در بخشِ «چالش» رویش کار می‌کنی.

### ساختنِ پاسخ: آینه‌ی دقیقِ همان مسیر

مسیرِ برعکس — دیتای Rust به بایتِ HTTP — دقیقاً همان قاعده‌هاست، فقط برعکس:

```rust
fn response_bytes(status: u16, reason: &str, headers: &[(&str, &str)], body: &str) -> Vec<u8> {
    let mut out = format!("HTTP/1.1 {status} {reason}\r\n");
    for (name, value) in headers {
        out.push_str(&format!("{name}: {value}\r\n"));
    }
    out.push_str(&format!("Content-Length: {}\r\n", body.len()));
    out.push_str("\r\n");
    out.push_str(body);
    out.into_bytes()
}
```

```text
HTTP/1.1 200 OK
Content-Type: text/plain
Content-Length: 14

Hello, world!
```

خطِ وضعیت، هدرها (که حتماً باید شاملِ `Content-Length` باشند — بدونش کلاینتی که منتظرِ بدنه است هیچ راهی ندارد بفهمد کِی تمام می‌شود، چون خودِ اتصال می‌تواند باز بماند)، یک خطِ خالی، و بعدش بدنه. و `Content-Length` باید طولِ **بایتِ** بدنه باشد، نه `.chars().count()`ش — یک کاراکترِ چندبایتیِ UTF-8 بیش از یک بایت است، و کلاینت دقیقاً همان تعداد بایت را از رویِ سیم می‌خواند.

```senpai-visual
{"kind":"network","labels":["بایت‌های TCP ورودی","parse_request","HttpRequest","HttpResponse::to_bytes","بایت‌های TCP خروجی"]}
```

---

## دست‌به‌کد

```sh
cargo run -p p3-01-02-hand-rolled-http-parser --example 01-bytes-must-be-validated
cargo run -p p3-01-02-hand-rolled-http-parser --example 02-splitting-the-wire-format
cargo run -p p3-01-02-hand-rolled-http-parser --example 03-case-insensitive-header-names
cargo run -p p3-01-02-hand-rolled-http-parser --example 04-building-a-response-by-hand
```

بعد دوتای خراب:

```sh
cargo run -p p3-01-02-hand-rolled-http-parser --example 05-newline-only-split-broken --features broken
cargo build -p p3-01-02-hand-rolled-http-parser --example 06-header-returns-string-not-str-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `02-splitting-the-wire-format`، خطِ درخواست را به `"GET / HTTP/1.1"` (بدونِ کوئری) عوض کن — خروجیِ `target` چطور فرق می‌کند؟
۲. در `03-case-insensitive-header-names`، یک کاندیدِ دیگر مثلِ `"HoSt"` به آرایه اضافه کن — حدس بزن قبل از اجرا کردن.
۳. در `04-building-a-response-by-hand`، بدنه را به یک رشته‌ی حاویِ یک حرفِ فارسی (مثلاً `"سلام"`) عوض کن. آیا `Content-Length` هنوز با تعدادِ **کاراکترها** برابر است؟

---

## خطاهایی که خواهی دید

### یک پنیکِ زمان‌اجرا — `\r`ی که از جداکننده‌ی اشتباه باقی مانده

```text
thread 'main' (9568) panicked at phase3-backend-foundations\01-networking-and-http-from-scratch\02-hand-rolled-http-parser\examples\05-newline-only-split-broken.rs:20:5:
assertion `left == right` failed: the version should not carry a stray \r
  left: "HTTP/1.1\r"
 right: "HTTP/1.1"
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(آن عددِ داخلِ پرانتز شناسه‌ی خودِ ریسمان است و هر بار اجرا عوض می‌شود؛ بقیه‌ی پیام همیشه یکی است.)

**دلیلِ خرابی:** `examples/05-newline-only-split-broken.rs` عمداً رویِ `\n` تنها تکه می‌کند، نه `\r\n`. وقتی خطِ درخواست را این‌طور جدا کنی، آن `\r`ی که واقعاً جزوِ جداکننده‌ی `\r\n` بود، رویِ آخرین توکنِ همان خط — اینجا `version` — باقی می‌ماند. کد کامپایل می‌شود و اجرا می‌شود؛ فقط دقیقاً همان‌جایی که `assert_eq!` مقدارِ واقعی را با `"HTTP/1.1"` مقایسه می‌کند، پنیک می‌گیرد.

**راهِ حل:** تکه‌کردن را از `raw.split('\n')` به `raw.split("\r\n")` عوض کن.

**چرا این راهِ حل است:** حالا خودِ `\r\n` کاملاً به‌عنوانِ جداکننده مصرف می‌شود و در هیچ توکنی باقی نمی‌ماند. این دقیقاً همان چیزی است که `parse_request` خودت هم باید انجام بدهد — و همان دلیلی است که مشخصاتِ آن تابع صراحتاً می‌گوید «رویِ `\r\n`، نه بایِ خالی».

### `E0308` — `header` یک `&String` برمی‌گرداند، نه `&str`

```text
error[E0308]: mismatched types
  --> phase3-backend-foundations\01-networking-and-http-from-scratch\02-hand-rolled-http-parser\examples\06-header-returns-string-not-str-broken.rs:14:9
   |
13 |       fn get(&self, name: &str) -> Option<&str> {
   |                                    ------------ expected `Option<&str>` because of return type
14 | /         self.entries
15 | |             .iter()
16 | |             .find(|(k, _)| k.eq_ignore_ascii_case(name))
17 | |             .map(|(_, v)| v)
   | |____________________________^ expected `Option<&str>`, found `Option<&String>`
   |
   = note: expected enum `Option<&str>`
              found enum `Option<&String>`
help: try converting the passed type into a `&str`
   |
17 |             .map(|(_, v)| v).map(|x| x.as_str())
   |                             ++++++++++++++++++++

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** `self.entries` از نوعِ `Vec<(String, String)>` است، پس بعدِ `.find()` و `.map(|(_, v)| v)`، مقداری که بیرون می‌آید `Option<&String>` است — یک ارجاع به همان `String`ی که داخلِ تاپل نشسته، نه یک `&str`. امضایِ تابع می‌گوید `Option<&str>`؛ این دو نوعِ متفاوتند، و Rust از `&String` به `&str` داخلِ یک `Option` خودکار تبدیل نمی‌کند (برخلافِ محلِ آرگومان، جایی که این تبدیل رایگان اتفاق می‌افتد).

**راهِ حل:** پیشنهادِ خودِ کامپایلر — یک `.map(|x| x.as_str())` دیگر — کار می‌کند، ولی تمیزترش این است که مستقیم بنویسی `v.as_str()` داخلِ همان کلوژرِ اول:

```rust
.map(|(_, v)| v.as_str())
```

**چرا این راهِ حل است:** `.as_str()` همان قرضِ `&String` را به `&str` می‌بیند، بدونِ هیچ کلونی. این دقیقاً همان چیزی است که فاز ۲ زیرِ عنوانِ deref coercion به‌ات یاد داد: تبدیل خودکار سرِ محلِ فراخوانی اتفاق می‌افتد، نه وقتی نوع از داخلِ یک `Option` بیرون می‌آید.

---

## تمرین

### گرم‌کردن

<details>
<summary>آیا <code>Host</code> همان هدرِ <code>host</code> است، طبقِ اسپکِ HTTP؟</summary>

بله. اسمِ هدرها طبقِ اسپک حساسِ حروف نیستند — این دو دقیقاً یک هدرند.

</details>

<details>
<summary>اگر کلاینتی بایت‌هایی بفرستد که UTF-8 معتبر نیستند، <code>parse_request</code> چه کار می‌کند؟</summary>

فکرش را قبل از دیدنِ جواب بکن.

</details>

<details>
<summary>پاسخ</summary>

نه پنیک می‌کند، نه بی‌سروصدا کنارشان می‌گذارد — یک `Err(HttpParseError::InvalidUtf8)` برمی‌گرداند. `std::str::from_utf8` این چک را اولین قدمِ تابع می‌کند.

</details>

<details>
<summary>خط‌هایِ یک درخواستِ HTTP با چی از هم جدا می‌شوند — <code>\n</code> تنها، یا <code>\r\n</code>؟</summary>

فکرش را قبل از دیدنِ جواب بکن.

</details>

<details>
<summary>پاسخ</summary>

`\r\n`. اگر فقط رویِ `\n` تکه کنی، آن `\r` رویِ آخرین توکنِ خطِ قبلی باقی می‌ماند — دقیقاً همان چیزی که `examples/05-newline-only-split-broken.rs` نشانت می‌دهد.

</details>

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let m = Method::parse("get");
println!("{m:?}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
Other("get")
```

متدهایِ HTTP، برخلافِ اسمِ هدرها، طبقِ اسپک حساسِ حروف‌اند. `"get"` هیچ بازویِ نام‌داری را جور در نمی‌آورد، پس می‌افتد به `Other`.

</details>

### تعمیر

هر دو مثالِ خراب را درست کن:

۱. `examples/05-newline-only-split-broken.rs` را طوری درست کن که پنیک نگیرد — جداکننده را از `'\n'` به `"\r\n"` عوض کن.
۲. `examples/06-header-returns-string-not-str-broken.rs` را طوری درست کن که کامپایل شود — بدونِ عوض‌کردنِ نوعِ برگشتیِ `get`.

### پیاده‌سازی

چهار تابع در `src/lib.rs`:

```sh
cargo test -p p3-01-02-hand-rolled-http-parser
```

هر کدام کاملاً در کامنتِ مستنداتِ خودش مشخص شده‌اند — لازم نیست فایلِ تست را باز کنی تا بفهمی باید چه بنویسی:

- `Method::parse` — یک توکنِ خطِ درخواست مثلِ `"GET"` را به گونه‌ی متناظرش نگاشت کن.
- `parse_request` — بایتِ خام را به یک `HttpRequest` معتبر، یا خطایِ دقیقِ همان چیزی که خراب بود، تبدیل کن.
- `HttpRequest::header` — جست‌وجویِ هدر، حساس‌نبودن‌به‌حروف.
- `HttpResponse::to_bytes` — سریالایز به فرمتِ رویِ سیم، با `Content-Length` درست.

### بساز

یک متدِ `pub fn query_params(&self) -> Vec<(String, String)>` رویِ `HttpRequest` اضافه کن که رشته‌ی کوئری را به جفت‌هایِ کلید/مقدار تبدیل کند — همان کاری که `request.GET` تویِ جنگو برایت خودکار انجام می‌دهد.

مشخصات: اگر `self.query` برابرِ `None` باشد، یک `Vec` خالی برگردان. در غیرِ این صورت، کوئری را رویِ `&` به جفت‌هایِ جداگانه تکه کن؛ هر جفت را رویِ *اولین* `=` به یک کلید و یک مقدار تکه کن — جفتی که هیچ `=`ی ندارد، مقدارش یک رشته‌ی خالی می‌شود. ترتیب و تکرار را عیناً نگه‌دار: `?tag=a&tag=b` باید دو ورودیِ جداگانه بدهد، و همین دلیلِ اینکه خروجی `Vec` است، نه `HashMap`.

### چالش (اختیاری)

خطِ درخواست را طوری سخت‌گیرانه‌تر پارس کن که فاصله‌های پیاپیِ اشتباهی هم خرابش نکنند — مثلاً `"GET  / HTTP/1.1"` با دو فاصله بینِ `GET` و `/`. `parse_request`ِ خودت را در `src/lib.rs` طوری عوض کن که این هم درست پارس شود، بدونِ اینکه فاصله‌ی تکی را بین توکن‌هایِ درست بشکند. (این تمرین، درست‌کردنِ کامل‌ترِ کدهایِ وضعیت و رفتارهایِ اسپک‌محورترِ HTTP را برایِ [۳.۱.۳](../03-http-semantics-you-must-know/README.fa.md) نگه می‌دارد — همین یک لبه‌ی کوچک از سخت‌گیریِ پارسر کافی است.)

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| فرمتِ رویِ سیم (wire format) | شکلِ عینیِ بایت‌به‌بایتیِ چیزی که واقعاً رویِ شبکه فرستاده می‌شود | خواندن/نوشتنِ هر پروتکلِ متنی، نه فقط HTTP |
| `std::str::from_utf8` | اعتبارسنجیِ خطاپذیرِ `&[u8]` به `&str` | اولین قدم برایِ هر بایتِ نامعتمد |
| `HttpParseError` | یک گونه‌ی جداگانه به‌ازایِ هر جور بدشکلی، نه یک خطایِ عمومی | مطابقت‌دادنِ دقیقِ خطا با علتش |
| `eq_ignore_ascii_case` | مقایسه‌ی رشته بدونِ حساسیت به حروف، بدونِ تخصیصِ اضافه | جست‌وجویِ اسمِ هدر |
| `Content-Length` | طولِ **بایتِ** بدنه، نه تعدادِ کاراکتر | ساختنِ هر پاسخِ HTTP دستی |

### الان می‌دانی

- چهار قاعده‌ی فرمتِ رویِ سیمِ یک درخواستِ HTTP/1.1 — جداکننده‌ی `\r\n`، شکلِ خطِ درخواست، حساس‌نبودنِ اسمِ هدرها به حروف، خطِ خالیِ پایان‌دهنده.
- چرا اولین کارِ `parse_request` تأیید کردنِ UTF-8 است، و اگر این چک نبود چه چیزی می‌توانست خراب شود.
- چرا `HttpParseError` به‌جایِ یک گونه‌ی عمومی، یک گونه‌ی جداگانه به‌ازایِ هر جور بدشکلی دارد.
- اسمِ هدرها طبقِ اسپک حساسِ حروف نیستند؛ متدهایِ HTTP هستند — و این دو را دیگر با هم قاطی نمی‌کنی.
- ساختنِ یک پاسخِ HTTP با دست، شاملِ محاسبه‌ی درستِ `Content-Length` از رویِ طولِ بایت.

### بعداً کامل‌تر می‌بینی

- **فهرستِ کاملِ کدهایِ وضعیت، content negotiation، keep-alive، و رمزگذاریِ chunked** — [۳.۱.۳ — چیزهایی از HTTP که باید بدونی](../03-http-semantics-you-must-know/README.fa.md)
- **بدنه‌یِ `POST`/`PUT` و خواندنِ آن بر اساسِ `Content-Length`** — [ماژولِ ۲ — `axum` و طراحیِ REST API](../../02-axum-and-rest-api-design/README.fa.md)
- **همین رشته‌ی کوئری، این‌بار خودکار decode‌شده — با اکسترکتورِ `Query<T>` در `axum`** — [۳.۲.۱ — روتینگ، هندلرها، اکسترکتورها](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `parse_request` باید *اول* بایت‌ها را به‌عنوانِ UTF-8 اعتبارسنجی کند، پیش از هر کارِ دیگری؟
- خط‌هایِ HTTP با چی جدا می‌شوند، و اگر با جداکننده‌ی اشتباه تکه کنی دقیقاً چه اتفاقی می‌افتد؟
- چرا اسمِ هدرها حساسِ حروف نیستند ولی متدهایِ HTTP هستند؟
- `HttpParseError` چرا چهار گونه‌ی جداگانه دارد، به‌جایِ یک `ParseFailed`ِ واحد؟
- `Content-Length` باید طولِ بایت باشد یا طولِ کاراکتر؟ چرا این تفاوت واقعاً مهم است؟

---

## بیشتر

- [MDN — HTTP messages](https://developer.mozilla.org/en-US/docs/Web/HTTP/Messages) — همین شکلِ رویِ سیم، با تصویر.
- [RFC 9112 — HTTP/1.1](https://www.rfc-editor.org/rfc/rfc9112.html) — اسپکِ رسمی؛ بخشِ ۳ (request line) و بخشِ ۵ (field syntax) دقیقاً همان چیزی است که امروز پیاده کردی.
- [`std::str::from_utf8` مستندات](https://doc.rust-lang.org/std/str/fn.from_utf8.html) — امضایِ کامل و شکلِ `Utf8Error`.
- [`str::eq_ignore_ascii_case` مستندات](https://doc.rust-lang.org/std/primitive.str.html#method.eq_ignore_ascii_case) — چرا این یکی به‌جایِ `.to_lowercase() ==` بهتر است (بدونِ تخصیصِ حافظه‌ی تازه).
