# ۰۱.۲ — parser دستی HTTP

## `axum` چه چیزی را از تو پنهان می‌کند؟

درخواست HTTP فقط **متنی روی اتصال TCP** است، با قالبی خط‌محور. وقتی Django، gunicorn یا `axum` objectای با `.method`، `.path` و `.headers` می‌دهد، همین parsing را قبلاً انجام داده است. یک بار آگاهانه دستی انجامش می‌دهیم تا دیگر black box نباشد.

## شکل درخواست HTTP/1.1

```text
GET /anime?status=watching HTTP/1.1\r\n
Host: localhost:7878\r\n
User-Agent: curl/8.4.0\r\n
Accept: */*\r\n
\r\n
```

چهار نکته مهم است:

۱. خط‌ها با `\r\n` جدا می‌شوند، نه فقط `\n`. اگر فراموشش کنی، `\r` انتهای خط مقایسه‌ها را بی‌صدا خراب می‌کند.
۲. request line سه token دارد: `{METHOD} {PATH} {VERSION}`.
۳. header به‌شکل `Name: value` است و نام header طبق specification به بزرگی/کوچکی حروف حساس نیست.
۴. یک خط خالی پایان headerهاست. این درس body مربوط به `POST`/`PUT` و `Content-Length` درخواست را عمداً وارد نمی‌کند؛ `axum` از ماژول بعد عهده‌دار آن است.

## byte وارد، داده‌ی ساخت‌یافته خارج

خواندن socket یک `&[u8]` می‌دهد، نه `&str`. parser باید ابتدا با `std::str::from_utf8` معتبر‌بودن UTF-8 را بررسی کند؛ client یا attacker هر byteای می‌تواند بفرستد. `parse_request` با `Result<HttpRequest, HttpParseError>` هر خرابی را با variant مشخص بازمی‌گرداند: UTF-8 نامعتبر، request line ناقص یا header بدون colon. مانند `.errors` در serializerهای DRF، اما type system این قرارداد را الزام می‌کند.

## ساخت پاسخ دستی

پاسخ جهت معکوس همان جریان است:

```text
HTTP/1.1 200 OK\r\n
Content-Length: 13\r\n
\r\n
Hello, world!
```

status line، headerها، خط خالی و body. `Content-Length` حیاتی است؛ client برای فهمیدن پایان body روی اتصال باز به آن نیاز دارد.

```senpai-visual
{"kind":"network","labels":["byte خام","parse_request","HttpRequest","HttpResponse","byte پاسخ"]}
```

تشبیه باجه‌ی پذیرش خوب است: فرم خام را به رکورد ساخت‌یافته تبدیل می‌کند. مرز تشبیه این است که malformed input حالت عادی اینترنت است و parser باید هر مرحله را با error مشخص رد کند.

## تمرین تو

`Method::parse`، `parse_request`، جست‌وجوی header غیرحساس به case و `HttpResponse::to_bytes` با `Content-Length` درست را کامل کن. `src/main.rs` سرور کوچکی روی `127.0.0.1:7879` دارد؛ با `curl -v` پاسخ خام `GET /` و `GET /nope` را ببین.

## ایست بازرسی

اول `CHECKPOINT.fa.md` و بعد `solution/SOLUTION.fa.md` را بخوان.
