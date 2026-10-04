# ۳.۸.۵ — WebSocket و SSE در `axum`

## در یک نگاه

بعد از این درس می‌توانی:

- بگویی چرا پاسخِ Server-Sent Events یک پاسخِ HTTPِ معمولی است که هیچ‌وقت تمام نمی‌شود، و بایت‌هایِ خامش را بخوانی: بدونِ `Content-Length`، با `Transfer-Encoding: chunked`، و یک chunk برایِ هر رویداد.
- یک endpointِ SSE با `Sse`، `Event` و `KeepAlive` و یک endpointِ WebSocket با `WebSocketUpgrade` بنویسی، و بگویی دست‌دهیِ `101` بینِ یک درخواست و یک سوکت چه کار می‌کند.
- یک پیام را با `tokio::sync::broadcast` به چندین کلاینتِ وصل‌شده پخش کنی، و با مشترکی که عقب می‌افتد کنار بیایی.
- برایِ یک قابلیتِ مشخص بینِ SSE و WebSocket انتخاب کنی، و بگویی هرکدام چه چیزی را مجبورت می‌کند خودت بسازی.

**زمان:** حدود ۱۰۰ دقیقه · **پیش‌نیاز:**
[۳.۱.۳ — چیزهایی از HTTP که باید بدانی](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md)،
[۳.۲.۱ — مسیریابی، هندلرها، اکسترکتورها](../../02-axum-and-rest-api-design/01-routing-handlers-extractors/README.fa.md)،
[۲.۹.۳ — استریم‌ها](../../../phase2-intermediate/09-async-in-practice/03-streams/README.fa.md)

---

## چرا اهمیت دارد

هر endpointی که تا حالا نوشته‌ای یک پرسش بود و بعدش یک پاسخ: کلاینت می‌پرسد، سرور جواب می‌دهد، گفتگو تمام است. خیلی از قابلیت‌هایِ واقعی در این قالب جا نمی‌شوند. نوارِ پیشرفتِ یک import طولانی، نشانِ «۳ قسمتِ جدید»، یک چتِ زنده، داشبوردی که خودش به‌روز می‌شود: اینجا *سرور* است که می‌داند چیزی اتفاق افتاده، و کلاینت راهی ندارد بفهمد کِی باید بپرسد.

در جنگو سراغِ polling می‌رفتی (صفحه هر چند ثانیه یک endpoint را صدا می‌زند) یا سراغِ Channels، که زیرساختِ جداگانه‌ای است با پردازه‌هایِ کارگرِ خودش. `axum` هیچ‌کدام را نمی‌خواهد. یک هندلر می‌تواند اتصال را باز نگه دارد و مدام رویش بنویسد، چون هندلر فقط یک `async fn` است و محیطِ اجرا به هندلری که یک ساعت طول بکشد اعتراضی ندارد. این درس دو راهِ استانداردِ این کار را نشان می‌دهد. [۳.۱.۳](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md) با یک قول تمام شد: رمزگذاریِ chunked می‌گذارد پاسخ پیش از معلوم‌شدنِ طولش شروع شود، و «پاسخ‌هایِ استریمی که هرگز `Content-Length` نمی‌گذارند» به اینجا موکول شد. SSE دقیقاً همان است. WebSocketها یک قدم جلوتر می‌روند و قالبِ درخواست-و-پاسخِ HTTP را کلاً کنار می‌گذارند.

---

## مفهوم

### دو راه برایِ نگه‌داشتنِ اتصالِ فشارنده

**رویدادهایِ ارسال‌شده از سرور** (Server-Sent Events، SSE) راهِ ساده است. کلاینت یک `GET`ِ معمولی می‌فرستد. سرور با `200` و `Content-Type: text/event-stream` جواب می‌دهد و بدنه را هیچ‌وقت تمام نمی‌کند: هر بار که اتفاقی بیفتد یک بلوکِ متنیِ کوچکِ دیگر می‌نویسد، و `EventSource`ِ مرورگر هر بلوک را به جاوااسکریپتِ تو می‌دهد. داده فقط یک جهت می‌رود: از سرور به کلاینت.

**WebSocketها** اتصال را ارتقا می‌دهند. کلاینت یک `GET` می‌فرستد که می‌خواهد پروتکل عوض شود، سرور با `101` جواب می‌دهد، و از آن به بعد همان اتصالِ TCP پیام‌هایِ فریم‌شده را در *هر دو* جهت می‌برد، بدونِ هیچ HTTPای در آن.

```senpai-visual
{"kind":"network","labels":["SSE: کلاینت یک GET می‌فرستد","سرور با 200 و text/event-stream جواب می‌دهد","سرور رویداد، رویداد، رویداد می‌نویسد...","WebSocket: کلاینت GET با Upgrade می‌فرستد","سرور با 101 Switching Protocols جواب می‌دهد","فریم‌ها تا بسته‌شدن در دو جهت جاری‌اند"]}
```

### SSE یک پاسخِ chunkedِ تمام‌نشدنی است

`examples/01-sse-raw-response.rs` سه رویداد سرو می‌کند و بعد پاسخ را رویِ یک `TcpStream`ِ ساده می‌خواند، همان نوعی که در [۳.۱.۱](../../01-networking-and-http-from-scratch/01-tcp-echo-server/README.fa.md) دیدی. هندلر این است:

```rust
async fn ticks() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = (1..=3).map(|n| {
        Ok(Event::default().event("tick").data(format!("tick {n}")))
    });
    Sse::new(tokio_stream::iter(events))
}
```

مثال هر CRLF را به‌شکلِ `\r\n`ِ دیدنی چاپ می‌کند و هدرِ `date:` را که هر اجرا فرق می‌کند کنار می‌گذارد:

```text
HTTP/1.1 200 OK\r\n
content-type: text/event-stream\r\n
cache-control: no-cache\r\n
connection: close\r\n
transfer-encoding: chunked\r\n
\r\n
1A\r\n
event: tick
data: tick 1

\r\n
1A\r\n
event: tick
data: tick 2

\r\n
1A\r\n
event: tick
data: tick 3

\r\n
0\r\n
\r\n
```

`Content-Length` نیست، چون سرور نمی‌داند این ماجرا تا کِی ادامه دارد. `Transfer-Encoding: chunked` هست، همان سازوکاری که [۳.۱.۳](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md) با دست ساخت: هر chunk اندازه‌اش در مبنای شانزده، بعد `\r\n`، بعد داده، بعد `\r\n` است (`1A` یعنی ۲۶ بایت). هر رویداد یک chunk است. یک endpointِ واقعیِ SSE فقط chunkِ پایانیِ `0` را هیچ‌وقت نمی‌فرستد، پس پاسخ تا وقتی اتصال زنده است باز می‌ماند. (این یکی سه رویداد دارد و تمام می‌شود، چون استریم تمام می‌شود و درخواست `Connection: close` خواسته بود.)

پس SSE پروتکلِ تازه‌ای نیست. HTTP/1.1 است که کاری را می‌کند که همیشه اجازه‌اش را داشت. به همین دلیل بدونِ هیچ پشتیبانیِ ویژه‌ای از پروکسی‌ها، لودبالانسرها و `curl` رد می‌شود.

### قالبِ سیمیِ یک رویداد

بدنه‌یِ پاسخ متنِ ساده‌ای در یک قالبِ ثابت است. یکی از chunkهایِ بالا را ببین، بینِ خطِ اندازه و `\r\n`:

```text
event: tick
data: tick 1

```

رویداد گروهی از خطوطِ `field: value` است که با یک خطِ خالی تمام می‌شود. فیلدها این‌ها هستند: `data` (بار؛ چند خطِ `data:` را مرورگر با `\n` به هم می‌چسباند)، `event` (یک نام، تا جاوااسکریپت بتواند به `"tick"` جدا از رویدادهایِ دیگر گوش بدهد)، `id` (پایین‌تر در «اتصالِ دوباره») و `retry` (مرورگر پیش از وصل‌شدنِ دوباره چند میلی‌ثانیه صبر کند). خطی که با `:` شروع شود توضیح است و مرورگر نادیده‌اش می‌گیرد. `axum` اتصالِ ساکت را همین‌طور زنده نگه می‌دارد.

قالب‌ساز را خودت در تمرینِ «پیاده‌سازی» می‌نویسی، چون یک بار با دست‌نوشتنِ قالب سریع‌ترین راهِ دست‌برداشتن از جادو دیدنِ آن است.

### SSE در `axum`

`examples/02-sse-ticker-server.rs` یک ساعتِ تیک‌تیکِ بی‌پایان رویِ پورتِ ۳۲۴۰ است. بخش‌هایش:

```rust
let stream = IntervalStream::new(interval(Duration::from_secs(1))).map(move |_| {
    id += 1;
    Ok(Event::default().event("tick").id(id.to_string()).data(format!("tick {id}")))
});
Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
```

`Sse::new` یک `Stream` ([۲.۹.۳](../../../phase2-intermediate/09-async-in-practice/03-streams/README.fa.md)) می‌گیرد که آیتم‌هایش `Result<Event, E>` هستند. `Event` یک builder است: `.event(...)`، `.id(...)`، `.data(...)`، و `.json_data(...)` اگر بار را JSON بخواهی. `Result` برایِ این است که استریم بتواند شکست بخورد، ولی در یک ساعتِ تیک‌تیک چیزی شکست نمی‌خورد، پس نوعِ خطا `Infallible` است. در «خطاهایی که خواهی دید» `Result` را کنار می‌گذاری و می‌خوانی `axum` چه می‌گوید.

`.keep_alive(...)` باعث می‌شود `axum` هر وقت استریم به‌اندازه‌یِ این بازه ساکت بود یک خطِ توضیح (`:`) وارد کند. بازه‌یِ پیش‌فرض ۱۵ ثانیه است. بدونِ آن، استریمی که یک دقیقه ساکت بماند برایِ هر پروکسیِ سرِ راه مثلِ اتصالِ مرده است و بعضی‌ها می‌بندندش.

سرور را بالا بیاور و با `curl -N` بخوانش (`-N` بافرِ خروجیِ curl را خاموش می‌کند، `--max-time` بعد از ۲٫۵ ثانیه متوقفش می‌کند، و برایِ همین curl با کدِ ۲۸ خارج می‌شود):

```sh
curl -sNi --max-time 2.5 http://127.0.0.1:3240/ticker
```

```text
HTTP/1.1 200 OK
content-type: text/event-stream
cache-control: no-cache
transfer-encoding: chunked

event: tick
id: 1
data: tick 1

event: tick
id: 2
data: tick 2

event: tick
id: 3
data: tick 3

```

(خطِ هدرِ `date:` را کنار گذاشته‌ام چون هر اجرا فرق می‌کند، و `\r` هم حذف شده.) اولین تیک بلافاصله می‌رسد، چون اولین تیکِ یک `interval`ِ tokio فوری است.

### اتصالِ دوباره: `Last-Event-ID`

شبکه قطع می‌شود. وقتی اتصالِ یک `EventSource` می‌شکند، مرورگر خودش دوباره وصل می‌شود و `id`ِ آخرین رویدادی را که گرفته در هدرِ درخواستِ `Last-Event-ID` می‌فرستد. ساعتِ بالا این هدر را می‌خواند و از عددِ بعدی ادامه می‌دهد:

```sh
curl -sN --max-time 2.5 -H "Last-Event-ID: 41" http://127.0.0.1:3240/ticker
```

```text
event: tick
id: 42
data: tick 42

event: tick
id: 43
data: tick 43

event: tick
id: 44
data: tick 44

```

اتصالِ دوباره با ادامه از همان‌جا در خودِ SSE ساخته شده است. در WebSocket نه: وقتی سوکت بمیرد هیچ‌چیز دوباره وصل نمی‌شود و هیچ‌چیز یادش نیست کلاینت چه دیده بود. هر دو را باید خودت بنویسی. این بزرگ‌ترین تفاوتِ عملیِ این دو است، و جدولِ پایین آن را کنارِ بقیه‌یِ تفاوت‌ها می‌گذارد.

### WebSocket: دست‌دهی یک درخواستِ HTTP است

WebSocket با یک `GET`ِ معمولی و سه هدرِ اضافه شروع می‌شود: `Upgrade: websocket`، `Connection: Upgrade`، و یک `Sec-WebSocket-Key` که یک مقدارِ تصادفیِ ۱۶ بایتی به base64 است. پاسخِ سرور همان ردهٔ `1xx`ای است که در [۳.۱.۳](../../01-networking-and-http-from-scratch/03-http-semantics-you-must-know/README.fa.md) دیدی: `101 Switching Protocols`. `examples/03-ws-handshake-by-hand.rs` این را با یک `TcpStream` در برابرِ یک روتِ echoِ `axum` انجام می‌دهد:

```text
< HTTP/1.1 101 Switching Protocols
< connection: upgrade
< upgrade: websocket
< sec-websocket-accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
< 
> [81, 85, 01, 02, 03, 04, 69, 67, 6f, 68, 6e]
< [81, 05, 68, 65, 6c, 6c, 6f]  ("hello")
```

(هدرِ `date:` مثلِ قبل کنار گذاشته شده. خط‌هایِ `<` چیزی‌اند که سرور فرستاد، خطِ `>` تنها فریمی است که کلاینت فرستاد.)

`Sec-WebSocket-Accept` گواهِ سرور است که درخواست را فهمیده: هشِ کلیدِ کلاینت به‌علاوه‌یِ یک رشته‌یِ ثابت از RFC 6455 است. کلیدِ این مثال نمونه‌یِ خودِ RFC است و `s3pPLMBiTxaQ9kYGzzhZRbK+xOo=` جوابی است که RFC برایش آورده، و برایِ همین تست‌هایِ این درس بدونِ هیچ کدِ هش‌کردنی دست‌دهی را بررسی می‌کنند.

بعد از `101` دیگر HTTPای نیست. دو خطِ `>` و `<` فریم‌اند. بایتِ اولِ هرکدام را بخوان: `81` یعنی «آخرین تکه، opcode برابرِ ۱ = متن». بایتِ دوم طول است و بیتِ بالایش می‌گوید «ماسک‌شده». فریمِ کلاینت `85` است: ماسک‌شده، طولِ ۵، بعد ماسکِ ۴بایتی و پنج بایتِ بار که با آن XOR شده‌اند (`69 67 6f 68 6e` بعد از ماسک‌شدن همان `hello` است). فریمِ سرور `05` است: بدونِ ماسک، طولِ ۵، بعد `68 65 6c 6c 6f` که `hello` است. RFC می‌گوید کلاینت باید ماسک کند و سرور نباید.

### WebSocket در `axum`

فیچرِ `ws` به‌طورِ پیش‌فرض خاموش است. `Cargo.toml`ِ این درس روشنش می‌کند:

```toml
axum = { workspace = true, features = ["ws"] }
```

هندلر یک اکسترکتورِ `WebSocketUpgrade` می‌خواهد و `on_upgrade` سوکتِ آماده را به یک تابعِ `async` می‌دهد. این کدِ داده‌شده در `src/lib.rs` است، کوتاه‌شده:

```rust
async fn ws_handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(Ok(msg)) = socket.recv().await {
        if let Message::Text(text) = msg {
            let reply = ws_reply(text.as_str());
            if socket.send(Message::text(reply)).await.is_err() { break; }
        }
    }
}
```

هندلر پاسخِ `101` را بلافاصله برمی‌گرداند. تابعی که به `on_upgrade` می‌دهی *بعد از آن* اجرا می‌شود، به‌عنوانِ تسکِ خودش، وقتی اتصال عوض شد. `socket.recv().await` یک `Option<Result<Message, Error>>` می‌دهد: `None` وقتی اتصال بسته شد، `Some(Err(_))` وقتی شکست، و در غیرِ این صورت `Some(Ok(message))`.

نوعِ `Message` پنج حالت دارد، با سورسِ `axum` نسخه‌یِ ۰٫۸٫۹ چک شده:

| حالت | بار | معنا |
|---|---|---|
| `Text` | `Utf8Bytes` | یک پیامِ متنیِ UTF-8 |
| `Binary` | `Bytes` | بایت‌هایِ خام |
| `Ping` و `Pong` | `Bytes` | بررسیِ زنده‌بودن؛ `axum` به یک `Ping` خودش `Pong` جواب می‌دهد |
| `Close` | `Option<CloseFrame>` | طرفِ مقابل دارد می‌بندد؛ `axum` با close خودش جواب می‌دهد |

`Text` یک `Utf8Bytes` نگه می‌دارد، نه `String`. این در `axum` ۰٫۸ تازه است، و اگر از یک آموزشِ قدیمی یاد بگیری اولین چیزی است که اذیتت می‌کند. `Utf8Bytes` یک بافرِ شمارش‌مرجع است، ارزان برای کلون‌کردن، که همیشه UTF-8ِ معتبر نگه می‌دارد: `.as_str()` آن را به‌شکلِ `&str` قرض می‌دهد و `.into()` از یک `String` یا `&str` می‌سازدش. `Message::text("hi")` سازنده‌یِ میان‌بر است. خطایش را در «خطاهایی که خواهی دید» می‌بینی.

`handle_socket`ِ واقعی در `src/lib.rs` از این قطعه بلندتر است. هر ۳۰ ثانیه هم ping می‌فرستد، و با `tokio::select!` ([۲.۹.۲](../../../phase2-intermediate/09-async-in-practice/02-select-and-cancellation-safety/README.fa.md)) `socket.recv()` را با یک زمان‌سنج رقابت می‌دهد. این ضربان (heartbeat) است، و دومین چیزی که WebSocket مجبورت می‌کند بسازی. اتصالی که ساکت مانده شاید سالم باشد و شاید ده دقیقه پیش یک جعبه‌یِ NAT رهایش کرده باشد، و هیچ‌کدام از دو طرف تا وقتی چیزی ننویسند نمی‌فهمند. pingِ دوره‌ای شکست را دیدنی می‌کند، و مرورگر خودکار با `Pong` جواب می‌دهد.

### کدام را انتخاب کنم

| | SSE | WebSocket |
|---|---|---|
| جهت | سرور به کلاینت | دو طرفه |
| انتقال | پاسخِ HTTPِ ساده، chunked | `101`ِ HTTP و بعد فریم‌هایِ خودش |
| بار | متنِ UTF-8 | متن یا دودویی |
| اتصالِ دوباره | داخلِ مرورگر، با `Last-Event-ID` | خودت می‌سازی |
| ضربان | خط‌هایِ توضیحِ `KeepAlive` | خودت می‌سازی (ping و pong) |
| عبور از پروکسی | معمولاً مشکلی نیست؛ HTTP است | پروکسی باید `Upgrade` را اجازه بدهد |
| APIِ مرورگر | `EventSource` | `WebSocket` |

قاعده کوتاه است. اگر داده همیشه *از* سرور می‌آید (اعلان‌ها، پیشرفت، یک فیدِ زنده، یک داشبورد)، SSE را انتخاب کن، چون بخش‌هایِ آزاردهنده‌یِ WebSocket در آن از قبل انجام شده‌اند. اگر کلاینت جریانِ پیوسته‌ای از پیام‌هایِ خودش می‌فرستد (یک کادرِ چت، یک بازیِ چندنفره، ویرایشِ هم‌کارانه) یا داده‌یِ دودویی لازم داری، WebSocket را انتخاب کن. هر چیزی که کلاینت در یک اپِ SSE گاه‌به‌گاه می‌فرستد می‌تواند یک `POST`ِ معمولی باشد، همان‌طور که `POST /publish` در این درس است.

### پخش: یک پیام، همه‌یِ کلاینت‌ها

یک اتصالِ تنها چندان جالب نیست. کارِ رایج این است: در یک هندلر اتفاقی می‌افتد و *همه‌یِ* کلاینت‌هایِ وصل‌شده باید خبردار شوند. این یک کانال با گیرنده‌هایِ زیاد می‌خواهد، که هر گیرنده نسخه‌یِ خودش را بگیرد. [۲.۸.۳](../../../phase2-intermediate/08-concurrency/03-channels-message-passing/README.fa.md) از `std::sync::mpsc` استفاده کرد، که هر پیام دقیقاً به یک گیرنده می‌رود. `tokio::sync::broadcast` شکلِ دیگر است: فرستنده‌هایِ زیاد، گیرنده‌هایِ زیاد، و هر گیرنده هر پیام را می‌بیند.

`examples/04-broadcast-fanout.rs` از یک `broadcast::Sender` روتِ `/events` را سرو می‌کند، دو کلاینتِ خامِ SSE وصل می‌کند، و یک پیام می‌فرستد:

```rust
let stream = BroadcastStream::new(tx.subscribe())
    .filter_map(|item| item.ok())
    .map(|msg| Ok(Event::default().data(msg)));
Sse::new(stream)
```

```text
subscribers: 2
delivered to 2
client A got "data: konnichiwa\n\n"
client B got "data: konnichiwa\n\n"
```

`tx.subscribe()` یک گیرنده‌یِ تازه می‌سازد که فقط چیزهایی را می‌بیند که *بعد از* وجودش فرستاده شوند. `BroadcastStream` از `tokio-stream` (فیچرِ `sync`) آن گیرنده را به `Stream` تبدیل می‌کند، تا در `Sse::new` جا بیفتد. `tx.send(msg)` تعدادِ گیرنده‌هایی را که پیام به آن‌ها رسید برمی‌گرداند، و وقتی هیچ‌کدام نباشند `Err` می‌دهد، که شکستِ سامانه نیست، فقط یعنی کسی گوش نمی‌دهد.

دو نکته در عمل مهم‌اند. اول، `subscribe()` در بدنه‌یِ هندلر صدا زده می‌شود، *پیش از* برگرداندنِ پاسخ، نه تنبل داخلِ استریم. وقتی کلاینت هدرهایِ پاسخ را دید، از قبل مشترک است، پس پیامی که همان لحظه فرستاده شود گم نمی‌شود. دوم، کانال ظرفیتِ ثابتی دارد. کلاینتی که خیلی کند می‌خواند فرستنده‌ها را کند نمی‌کند (این می‌گذاشت یک گوشیِ بد همه را متوقف کند): به‌جایش، وقتی بیش از `capacity` پیام عقب افتاد قدیمی‌ترین‌ها را از دست می‌دهد و آیتمِ بعدی‌اش `Err(Lagged(n))` است، که `n` تعدادِ پیام‌هایِ ازدست‌رفته است. `.filter_map(|item| item.ok())`ِ بالا آن را بی‌صدا دور می‌اندازد. در «خطاهایی که خواهی دید» می‌بینی `unwrap()` با آن چه می‌کند، و در «بساز» آن را به‌شکلِ یک رویداد به کلاینت گزارش می‌کنی.

```senpai-visual
{"kind":"queue","labels":["POST /publish تابعِ hub.publish را صدا می‌زند","کانالِ broadcast پیام‌هایِ اخیر را نگه می‌دارد","استریمِ کلاینتِ A نسخه‌یِ خودش را می‌خواند","استریمِ کلاینتِ B نسخه‌یِ خودش را می‌خواند","کلاینتِ کند عقب می‌افتد: Lagged(n)","قدیمی‌ترین پیام‌ها دور ریخته می‌شوند، نه فرستنده‌ها"]}
```

### اینجا چه چیزی تست می‌شود و چه چیزی نه

تست‌هایِ این درس یک سرورِ واقعی را رویِ `127.0.0.1:0` (سیستم‌عامل یک پورتِ آزاد انتخاب می‌کند) بالا می‌آورند و با یک `TcpStream`ِ خام با آن حرف می‌زنند، پس بایت‌هایِ رویِ سیم را چک می‌کنند. تست‌هایِ SSE رمزگذاریِ chunked را با یک کمکیِ کوچک در `tests/common/mod.rs` رمزگشایی می‌کنند. برایِ WebSocket در این workspace کتابخانه‌یِ کلاینت نیست، پس همان فایل یک کلاینتِ دست‌نویس دارد: دست‌دهی با کلیدِ نمونه‌یِ RFC، یک فریمِ ماسک‌شده به بیرون، یک فریم به داخل. پیام‌هایِ متنیِ زیرِ ۱۲۶ بایت، فریم‌هایِ close و مقدارِ `Sec-WebSocket-Accept` را پوشش می‌دهد. پیام‌هایِ تکه‌تکه، فریم‌هایِ دودویی، ping و pong، و ضربانِ ۳۰ثانیه‌ای را پوشش *نمی‌دهد*، و اینجا هیچ‌چیز مرورگری را اجرا نمی‌کند. برایِ یک کتابخانه‌یِ واقعیِ کلاینت، `tokio-tungstenite` انتخابِ معمول است. جزوِ وابستگی‌هایِ این دوره نیست، پس برایِ تمرین‌ها لازمش نداری، ولی در یک پروژه‌یِ واقعی می‌خواهیش. هر سروری که یک تست بالا بیاورد آخرِ همان تست متوقف می‌شود.

---

## دست‌به‌کد

```sh
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 01-sse-raw-response
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 03-ws-handshake-by-hand
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 04-broadcast-fanout
```

مثالِ `02` یک سرور است. آن را در یک ترمینال بالا بیاور و دو دستورِ `curl`ِ «مفهوم» را در ترمینالِ دیگر بزن، بعد با Ctrl+C متوقفش کن:

```sh
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 02-sse-ticker-server
```

بعد سه‌تایِ خراب. `06` و `07` کامپایل نمی‌شوند، `08` پنیک می‌کند:

```sh
cargo build -p p3-08-05-websockets-and-sse-in-axum --example 06-sse-item-not-result-broken --features broken
cargo build -p p3-08-05-websockets-and-sse-in-axum --example 07-ws-text-string-broken --features broken
cargo run -p p3-08-05-websockets-and-sse-in-axum --example 08-lagged-unwrap-panic-broken --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-sse-raw-response` به‌جایِ داده‌یِ تک‌خطی `.data("a\nb")` بگذار. رویداد رویِ سیم چند خطِ `data:` دارد، و اندازه‌یِ chunk عوض می‌شود؟
۲. در `02-sse-ticker-server` بازه‌یِ keep-alive را ۱ ثانیه و بازه‌یِ تیک را ۳ ثانیه کن، دوباره اجرا کن و `curl -N` را تماشا کن. بینِ تیک‌ها چه چیزی پیدا می‌شود؟
۳. در `04-broadcast-fanout` یک کلاینتِ سوم *بعد از* `send` وصل کن. `konnichiwa` را می‌گیرد؟

---

## خطاهایی که خواهی دید

هر رونوشتِ پایین خروجیِ واقعیِ مثالِ نام‌برده است، بدونِ هشدارهایِ `todo!()`ِ خودِ این درس.

### `E0271` — استریمِ SSE که آیتم‌هایش `Result` نیستند

```text
error[E0271]: type mismatch resolving `<Iter<IntoIter<Event>> as Stream>::Item == Result<_, _>`
  --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\05-websockets-and-sse-in-axum\examples\06-sse-item-not-result-broken.rs:9:25
   |
 9 |     let _sse = Sse::new(tokio_stream::iter(events));
   |                -------- ^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<_, _>`, found `Event`
   |                |
   |                required by a bound introduced by this call
   |
   = note: expected enum `Result<_, _>`
            found struct `Event`
   = note: required for `tokio_stream::Iter<std::vec::IntoIter<Event>>` to implement `futures_core::stream::TryStream`
note: required by a bound in `Sse::<S>::new`
  --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\response\sse.rs:64:12
   |
62 |     pub fn new(stream: S) -> Self
   |            --- required by a bound in this associated function
63 |     where
64 |         S: TryStream<Ok = Event> + Send + 'static,
   |            ^^^^^^^^^^^^^^^^^^^^^ required by this bound in `Sse::<S>::new`

error[E0271]: type mismatch resolving `<Iter<IntoIter<Event>> as Stream>::Item == Result<_, _>`
 --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\05-websockets-and-sse-in-axum\examples\06-sse-item-not-result-broken.rs:9:16
  |
9 |     let _sse = Sse::new(tokio_stream::iter(events));
  |                ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected `Result<_, _>`, found `Event`
  |
  = note: expected enum `Result<_, _>`
           found struct `Event`
  = note: required for `tokio_stream::Iter<std::vec::IntoIter<Event>>` to implement `futures_core::stream::TryStream`

For more information about this error, try `rustc --explain E0271`.
error: could not compile `p3-08-05-websockets-and-sse-in-axum` (example "06-sse-item-not-result-broken") due to 2 previous errors
```

**کامپایلر به چه اعتراض دارد:** `Sse::new` شرطِ `S: TryStream<Ok = Event>` را می‌خواهد. `TryStream` یک `Stream` است که آیتم‌هایش `Result` هستند، و `note:` ساده می‌گوید: انتظارِ `Result<_, _>` بود، `Event` پیدا شد. خطای دوم همان اشتباه است که یک بار دیگر برایِ کلِ فراخوانی گزارش شده.

**اصلاح:** هر آیتم را در `Ok` بپیچ، و بگو نوعِ خطا چیست، وقتی چیزی شکست نمی‌خورد `Infallible`:

```rust
let events = events.into_iter().map(Ok::<Event, Infallible>);
let _sse = Sse::new(tokio_stream::iter(events));
```

**چرا این اصلاح است:** نوعِ خطایِ استریم همان خطایی است که *منبعِ* رویدادها می‌تواند بخورد. برایِ یک کورسورِ پایگاه‌داده یا خواندنِ فایل یک خطایِ واقعی است، و `axum` وقتی یکی ظاهر شود استریم را تمام می‌کند. برایِ یک ساعتِ تیک‌تیکِ درون‌حافظه هیچ‌چیز خراب نمی‌شود، و `Infallible` نوعِ «این نمی‌تواند اتفاق بیفتد» است (همان نوعی که یک `Router` استفاده می‌کند، از [۳.۲.۴](../../02-axum-and-rest-api-design/04-tower-service-and-layer-middleware/README.fa.md)).

### `E0308` — `Message::Text` دیگر `String` نیست

```text
error[E0308]: mismatched types
   --> phase3-backend-foundations\08-error-handling-and-testing-at-scale\05-websockets-and-sse-in-axum\examples\07-ws-text-string-broken.rs:9:31
    |
  9 |     socket.send(Message::Text(hello)).await.unwrap();
    |                 ------------- ^^^^^ expected `Utf8Bytes`, found `String`
    |                 |
    |                 arguments to this enum variant are incorrect
    |
note: tuple variant defined here
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\axum-0.8.9\src\extract\ws.rs:774:5
    |
774 |     Text(Utf8Bytes),
    |     ^^^^
help: call `Into::into` on this expression to convert `String` into `Utf8Bytes`
    |
  9 |     socket.send(Message::Text(hello.into())).await.unwrap();
    |                                    +++++++

For more information about this error, try `rustc --explain E0308`.
error: could not compile `p3-08-05-websockets-and-sse-in-axum` (example "07-ws-text-string-broken") due to 1 previous error
```

**کامپایلر به چه اعتراض دارد:** این حالت یک `Utf8Bytes` می‌گیرد (`note:` به تعریفش در `ws.rs`ِ `axum` نسخه‌یِ ۰٫۸٫۹ اشاره می‌کند) و تو یک `String` دادی. آموزش‌هایِ قدیمی، که برایِ `axum` ۰٫۶ و ۰٫۷ نوشته شده‌اند، `Message::Text(String)` دارند.

**اصلاح:** پیشنهادِ خودِ کامپایلر، `hello.into()`، یا سازنده‌یِ `Message::text(hello)`.

**چرا این اصلاح است:** `Utf8Bytes` را می‌شود از یک `String` یا `&str` ساخت، و این تبدیل شکست نمی‌خورد (یک `String` از قبل UTF-8ِ معتبر است)، پس `.into()` کافی است. نوع ثبت می‌کند که یک پیامِ متنی UTF-8ِ معتبر است، تا پایین‌دست‌ها مجبور نباشند دوباره بررسی کنند.

### یک پنیکِ زمانِ اجرا: `unwrap` رویِ مشترکِ عقب‌افتاده

```text
thread 'main' (41756) panicked at phase3-backend-foundations\08-error-handling-and-testing-at-scale\05-websockets-and-sse-in-axum\examples\08-lagged-unwrap-panic-broken.rs:19:33:
called `Result::unwrap()` on an `Err` value: Lagged(3)
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(عددِ داخلِ پرانتز شناسه‌یِ ریسمان است و هر اجرا عوض می‌شود.)

**واقعاً چه چیزی خراب است:** مثال یک کانال با جایِ ۲ پیام می‌سازد، پیش از آنکه گیرنده چیزی بخواند ۵ پیام می‌فرستد، و بعد هر آیتم را `unwrap` می‌کند. اولین آیتمی که گیرنده می‌گیرد پیام نیست، `Err(Lagged(3))` است: «۳ تا را از دست دادی». سیستمِ نوع این احتمال را نشان داده بود، چون `BroadcastStream` چیزهایِ `Result` می‌دهد، و `unwrap` آن را به یک کرش تبدیل کرد.

**اصلاح:** رویِ آیتم `match` بزن و عقب‌افتادن را مثلِ داده رفتار کن:

```rust
match item {
    Ok(n) => println!("got {n}"),
    Err(BroadcastStreamRecvError::Lagged(missed)) => println!("lagged: missed {missed}"),
}
```

با این، مثال `lagged: missed 3`، `got 4` و `got 5` را چاپ می‌کند.

**چرا این اصلاح است:** در یک سرور این `unwrap` هر بار که یک گوشی رویِ شبکه‌یِ بد عقب بیفتد تسکِ یک اتصال را پنیک می‌کرد. عقب‌افتادن باگ نیست؛ راهِ کانال است برایِ اینکه خواننده‌هایِ کند فرستنده‌هایِ سریع را متوقف نکنند، و یک تصمیم می‌خواهد: بی‌صدا ردش کن، به کلاینت بگو (رویدادِ `lagged`ِ تمرینِ «بساز»)، یا قطعش کن.

---

## تمرین

### گرم‌کردن

<details>
<summary>پاسخِ SSE <code>Content-Length</code> ندارد. کلاینت از کجا می‌فهمد یک رویداد کجا تمام می‌شود و بعدی کجا شروع، و از کجا می‌فهمد خودِ پاسخ تمام نشده؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

یک خطِ خالی هر رویداد را تمام می‌کند، پس کلاینت تا دیدنِ `\n\n` می‌خواند. پاسخ هرگز تمام نمی‌شود چون سرور chunkِ پایانیِ اندازه‌صفرِ رمزگذاریِ chunked را هیچ‌وقت نمی‌فرستد. اتصال را باز نگه می‌دارد و هر بار که اتفاقی بیفتد یک chunkِ دیگر می‌نویسد.

</details>

<details>
<summary>اتصالِ مرورگر به endpointِ SSEِ تو ده ثانیه قطع می‌شود. در کلاینت چه می‌شود، و برایِ اینکه ادامه درست باشد در سرور چه باید بنویسی؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

`EventSource` خودش دوباره وصل می‌شود و `id`ِ آخرین رویدادی را که دیده در هدرِ `Last-Event-ID` می‌فرستد. برایِ درست‌بودنِ ادامه، سرور باید رویدادهایش `id` داشته باشند و در اتصالِ دوباره آن هدر را بخواند و آنچه را از دست رفته بازپخش کند. هیچ‌چیز یک WebSocket را برایِ تو دوباره وصل نمی‌کند.

</details>

<details>
<summary>یک کانالِ <code>broadcast</code> ظرفیتِ ۳ دارد و یک مشترک ۱۰ پیام عقب است. <code>recv</code>ِ بعدی‌اش چه می‌دهد، و آیا فرستنده کند می‌شود؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

یک `Err(Lagged(n))` که `n` تعدادِ پیام‌هایی است که از دست داده، و بعد قدیمی‌ترین پیام‌هایی که هنوز در بافر مانده‌اند. فرستنده کند نمی‌شود: هرگز منتظرِ گیرنده‌ها نمی‌ماند.

</details>

<details>
<summary>یک جدولِ امتیازِ زنده می‌سازی که مرورگر فقط تماشا می‌کند، و یک دکمه‌یِ «ریست» که ادمین گاه‌به‌گاه می‌زند. SSE یا WebSocket، و ریست چطور به سرور می‌رسد؟</summary>

پیش از دیدنِ پاسخ خودت فکر کن.

</details>

<details>
<summary>پاسخ</summary>

SSE: داده فقط از سرور به کلاینت می‌رود و مرورگر بدونِ هیچ کدی دوباره وصل می‌شود و ادامه می‌دهد. دکمه‌یِ ریست یک درخواستِ `POST`ِ معمولی است، همان‌طور که `POST /publish` در این درس است.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/06-sse-item-not-result-broken.rs` کامپایل شود.
۲. `examples/07-ws-text-string-broken.rs` کامپایل شود.
۳. `examples/08-lagged-unwrap-panic-broken.rs` عبارت‌هایِ `lagged: missed 3`، `got 4` و `got 5` را چاپ کند و تمام شود.

### پیاده‌سازی

دو تابعِ خالص در `src/lib.rs`:

- `sse_frame`: یک رویدادِ SSE را به‌شکلِ همان متنِ دقیقی که رویِ سیم می‌رود تولید می‌کند.
- `ws_reply`: پاسخِ یک پیامِ متنی در روتِ `/ws`.

```sh
cargo test -p p3-08-05-websockets-and-sse-in-axum --test sse_frame_test
cargo test -p p3-08-05-websockets-and-sse-in-axum --test ws_reply_test
cargo test -p p3-08-05-websockets-and-sse-in-axum --test ws_wire_test
```

توضیحِ بالایِ هر تابع کلِ مشخصاتِ آن است، با رشته‌هایِ دقیق. هیچ‌وقت لازم نیست تست‌ها را باز کنی. فایلِ تستِ اول یک تست دارد، `matches_what_axum_sends`، که روتِ `/events`ِ تمرینِ «بساز» را هم لازم دارد، پس تا «بساز» تمام نشود می‌افتد. `ws_wire_test` یک سرور بالا می‌آورد و با کلاینتِ WebSocketِ دست‌نویس با آن حرف می‌زند؛ بعد از پیاده‌شدنِ `ws_reply` سبز می‌شود، چون `handle_socket`ِ داده‌شده آن را صدا می‌زند.

### بساز

`Hub::publish`، هندلرِ `events` و هندلرِ `publish` در همان فایل: اتاقِ broadcastِ پشتِ `GET /events` و `POST /publish`.

```sh
cargo test -p p3-08-05-websockets-and-sse-in-axum --test hub_test
```

توضیح‌هایِ بالایِ آن‌ها مشخصات‌اند. دو نکته که تست‌ها فراتر از «کار می‌کند» چک می‌کنند همان‌هایی‌اند که در «پخش» دیدی: هندلر پیش از برگشتن مشترک می‌شود، و به مشترکِ عقب‌افتاده با یک رویدادِ `lagged` خبر داده می‌شود نه اینکه قطع شود. تست‌ها دو کلاینتِ واقعیِ SSE وصل می‌کنند، از راهِ `POST /publish` منتشر می‌کنند و بایت‌هایی را که هرکدام می‌گیرد چک می‌کنند.

### چالش (اختیاری)

`/events` را ادامه‌پذیر کن. به هر پیامِ منتشرشده یک شماره بده، ۱۰۰ پیامِ آخر را در یک بافرِ حلقوی داخلِ `Hub` نگه دار، و هر رویداد را با `id: <number>` بفرست. وقتی کلاینت با هدرِ `Last-Event-ID` وصل شد، اول هر چیزِ جدیدتر از آن شناسه را از بافر بازپخش کن، بعد به پیام‌هایِ زنده برو. هیچ تستی این را چک نمی‌کند. با دست و `curl -N -H "Last-Event-ID: 3"` امتحانش کن: بخشِ سخت این است که در درزِ بینِ بازپخش و زنده پیامی گم یا تکرار نشود، و پیش از نوشتنِ کد باید به همین یکی فکر کنی.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کار می‌آید |
|---|---|---|
| رویدادهایِ ارسال‌شده از سرور (SSE) | یک پاسخِ chunkedِ تمام‌نشدنیِ HTTP از رویدادهایِ `text/event-stream` | فیدهایِ زنده، پیشرفت، اعلان‌ها |
| `Sse` / `Event` / `KeepAlive` | نوعِ پاسخ، سازنده‌یِ رویداد و فرستنده‌یِ توضیحِ بیکاریِ `axum` | هر endpointِ SSE |
| `Last-Event-ID` | هدری که مرورگرِ دوباره‌وصل‌شونده با آخرین `id`ِ دیده‌شده می‌فرستد | ادامه‌یِ یک استریمِ SSE |
| WebSocket | اتصالی که با `101` از HTTP ارتقا یافته و بعد پیام‌هایِ فریم‌شده در دو جهت | چت، بازی، هم‌کاری |
| `WebSocketUpgrade` / `WebSocket` | اکسترکتوری که دست‌دهی را می‌کند و سوکتی که می‌خوانی و می‌نویسی | هر endpointِ WebSocket |
| `Message` | متن (`Utf8Bytes`)، دودویی، ping، pong یا close | خواندن و نوشتنِ فریم‌ها |
| ضربان (heartbeat) | یک pingِ دوره‌ای تا اتصالِ مرده دیده شود | WebSocketهایِ بلندمدت |
| کانالِ `broadcast` | فرستنده‌هایِ زیاد، گیرنده‌هایِ زیاد، هرکدام هر پیام را می‌گیرد | پخش به کلاینت‌هایِ وصل |
| `Lagged(n)` | مشترک `n` پیام را از دست داد چون عقب افتاد | برخورد با کلاینتِ کند |

### الان می‌دانی

- پاسخِ SSE همان HTTP/1.1 است که کاری را می‌کند که همیشه می‌توانست: یک بدنه‌یِ chunked بدونِ `Content-Length` که سرور هیچ‌وقت تمامش نمی‌کند.
- رویداد خطوطِ `field: value` است که با یک خطِ خالی تمام می‌شود، و `Sse`ِ `axum` یک `Stream` از `Result<Event, E>` می‌گیرد.
- WebSocket یک درخواستِ HTTP است که با `101` جواب داده می‌شود، و بعدش اتصال فریم حرف می‌زند: کلاینت‌ها ماسک می‌کنند، سرورها نه.
- `Message::Text` در `axum` ۰٫۸ یک `Utf8Bytes` نگه می‌دارد، نه `String`.
- SSE خودش دوباره وصل می‌شود و ادامه می‌دهد و مثلِ HTTPِ ساده از پروکسی رد می‌شود. WebSocket اتصالِ دوباره و ضربانِ خودش را می‌خواهد، و در عوض دو طرفه است و داده‌یِ دودویی هم می‌برد.
- `tokio::sync::broadcast` به هر مشترک نسخه‌یِ خودش از هر پیام را می‌دهد، هرگز فرستنده را متوقف نمی‌کند، و مشترکِ کند را با `Lagged(n)` گزارش می‌کند.

### بعداً کامل‌تر می‌بینی

- **احرازِ هویتِ درخواست‌ها با میان‌افزار، که درخواستِ ارتقایِ WebSocket هم مثلِ هر درخواستِ دیگر از آن رد می‌شود** — [۳.۷.۳ — JWT و میان‌افزار در `tower`](../../07-auth-and-security/03-jwt-and-tower-middleware/README.fa.md)
- **وقتی سرور می‌ایستد اتصال‌هایِ باز چه می‌شوند** — [۳.۴.۳ — خاموشیِ آرام، health و readiness](../../04-configuration-and-app-structure/03-graceful-shutdown-health-readiness/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا یک endpointِ SSE می‌تواند `Content-Length` نداشته باشد، و در پاسخِ خام چه چیزی به کلاینت می‌گوید هر تکه چقدر است؟
- کلاینت برایِ ادامه‌یِ یک استریمِ SSE بعد از اتصالِ دوباره چه می‌فرستد، و سرور با آن چه باید بکند؟
- سه هدری که یک WebSocket را شروع می‌کنند چه هستند، سرور چه جواب می‌دهد، و چرا آن جواب یک پاسخِ `1xx` حساب می‌شود؟
- چرا WebSocket ضربان می‌خواهد، و چه کسی آن را می‌فرستد؟
- چرا `subscribe()` باید پیش از برگشتنِ هندلرِ SSE اجرا شود، و `Lagged(n)` به تو چه می‌گوید؟
- برایِ یک نشانِ اعلان و برایِ یک بازیِ چندنفره، SSE یا WebSocket را انتخاب می‌کنی، و چرا؟

---

## بیشتر

- [`axum::response::sse`](https://docs.rs/axum/0.8.9/axum/response/sse/index.html): `Sse`، `Event` و `KeepAlive` با مثال.
- [`axum::extract::ws`](https://docs.rs/axum/0.8.9/axum/extract/ws/index.html): `WebSocketUpgrade`، `WebSocket` و `Message`، از جمله جداکردنِ سوکت به یک خواننده و یک نویسنده.
- [WHATWG: Server-sent events](https://html.spec.whatwg.org/multipage/server-sent-events.html): قالبِ استریمِ رویداد، زمانِ اتصالِ دوباره و `Last-Event-ID`.
- [RFC 6455 — The WebSocket Protocol](https://www.rfc-editor.org/rfc/rfc6455.html): دست‌دهی (§1.3، منبعِ کلیدِ نمونه‌یِ اینجا)، فریم‌بندی و ماسک (§5) و ping و pong (§5.5).
- [`tokio::sync::broadcast`](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html): ظرفیت، `Lagged` و `RecvError`.
