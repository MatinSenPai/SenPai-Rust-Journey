# ۰۲.۳ — CORS و اتصال frontend

## باگی که باگ نیست

API درس قبل با `curl` و test درست کار می‌کند، اما frontend روی `http://localhost:5173` هنگام `fetch("http://localhost:3001/anime")` خطای «blocked by CORS policy» می‌گیرد. Rust خراب نشده: response ساخته و ارسال شده، اما **browser آن را به JavaScript صفحه نمی‌دهد**. same-origin policy را browser enforce می‌کند، نه server. server فقط با header اعلام می‌کند به کدام origin اعتماد دارد.

origin ترکیب scheme، host و port است. نسبت به `http://localhost:3000`، هر کدام متفاوت‌اند: `https://localhost:3000` به‌خاطر scheme، `http://localhost:5173` به‌خاطر port و `http://127.0.0.1:3000` به‌خاطر host. `curl`، test یا backend دیگر CORS را enforce نمی‌کنند؛ CORS access control API نیست.

## preflight: request `OPTIONS` بدون handler تو

request ساده مانند GET بدون header سفارشی مستقیم ارسال می‌شود. بقیه ابتدا preflight می‌گیرند: browser یک `OPTIONS` همراه `Origin`، `Access-Control-Request-Method` و در صورت لزوم `Access-Control-Request-Headers` می‌فرستد و فقط با تأیید response، request واقعی را ارسال می‌کند. methodهای `PUT`/`PATCH`/`DELETE`، headerهایی مانند `Authorization` و حتی `Content-Type: application/json` preflight می‌خواهند.

| header | معنا | کجا می‌آید |
|---|---|---|
| `access-control-allow-origin` | کدام origin می‌تواند پاسخ را بخواند | preflight و پاسخ واقعی |
| `access-control-allow-methods` | methodهای مجاز request واقعی | فقط preflight |
| `access-control-allow-headers` | headerهای مجاز request | فقط preflight |

`CorsLayer` preflight را پیش از router intercept و خود پاسخ می‌دهد؛ به همین دلیل `.route(..., options(...))` لازم نیست و body پاسخ خالی است.

## ترکیب ممنوع

`Access-Control-Allow-Origin: *` با `Access-Control-Allow-Credentials: true` ممنوع است. این ترکیب به هر سایت اجازه می‌دهد requestهای دارای cookie کاربر را بفرستد و response را بخواند؛ یک گزینه‌ی آماده برای سرقت session. `tower-http` برای جلوگیری از انتشار تصادفی این حفره panic می‌کند.

## dev در برابر production

- در dev originها دائم عوض می‌شوند؛ local را permissive نگه دار.
- در production فقط origin واقعی frontend، methodها و headerهای لازم را اجازه بده.

```senpai-visual
{"kind":"network","labels":["frontend origin","OPTIONS preflight","CorsLayer","API route","browser اجازه/رد"]}
```

مثل گیت ورودی است که browser بلیت origin را می‌بیند. مرز تشبیه: server client بدخواه را متوقف نمی‌کند؛ فقط browser را وادار می‌کند response را در اختیار صفحه نگذارد.

## test بدون browser

preflight فقط یک HTTP request است. با `oneshot` آن را بساز و روی headerهای `access-control-allow-*` assertion بزن. unknown origin نیز معمولاً `200` بدون allow-origin می‌گیرد؛ «اعتماد نمی‌کنم» error HTTP نیست.

## تمرین تو

`dev_cors()` برای هر origin/method/header و `prod_cors(allowed_origin)` برای یک origin، فقط `GET`/`POST` و فقط `content-type` را پیاده کن. سپس test package و checkpoint را اجرا کن.
