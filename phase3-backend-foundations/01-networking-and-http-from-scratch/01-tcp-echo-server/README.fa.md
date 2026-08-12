# ۰۱.۱ — TCP echo server

## چرا پایین‌تر از framework شروع می‌کنیم؟

هر view در Django بر چند لایه تکیه دارد: server یک اتصال TCP خام را می‌پذیرد، byteها را می‌خواند، آن‌ها را HTTP می‌فهمد، URL را match می‌کند و تازه یک `request` مرتب تحویلت می‌دهد. فاز سه این تصویر را از socket بازسازی می‌کند تا وقتی کمی بعد `axum::Router::new().route(...)` می‌نویسی، دقیقاً بدانی زیر آن چه می‌گذرد. این درس هیچ dependency و frameworkای ندارد؛ فقط ابزار شبکه‌ی کتابخانه‌ی استاندارد.

## socket واقعاً چیست؟

**اتصال TCP** یک جریان byte دوسویه، مرتب و قابل‌اعتماد میان دو process است؛ ممکن است روی دو ماشین باشد. اگر یک سر `b"hello"` و بعد `b"world"` بنویسد، سر دیگر ترتیب را حفظ‌شده می‌خواند و TCP/سیستم‌عامل retransmission را انجام می‌دهد. اما این به‌معنای رسیدن داده در بسته‌های تمیز نیست: یک `write` ممکن است چند `read` شود و چند write کوچک ممکن است در یک read برسند.

`TcpListener` به آدرس `host:port` bind و برای اتصال گوش می‌دهد. هر اتصال پذیرفته‌شده یک `TcpStream` است؛ لوله‌ی byte دوسویه‌ای که مانند file handle از آن می‌خوانی و در آن می‌نویسی. این رابطه همتای `socket.socket()`، `.bind()`، `.listen()` و `.accept()` در پایتون است.

```rust
use std::net::TcpListener;

let listener = TcpListener::bind("127.0.0.1:7878")?;
for stream in listener.incoming() {
    let stream = stream?;
    // این اتصال را handle کن
}
```

## چرا line-based و چرا تابع جنریک؟

echo server «سلام دنیا»ی networking است: هرچه client بفرستد همان را برمی‌گرداند. اینجا داده را line-by-line و با `\n` می‌خوانیم، چون در درس بعد request HTTP را نیز خط‌به‌خط تا یک نشانه‌ی معنی‌دار می‌خوانی.

منطق اصلی یعنی `run_echo` به‌جای `TcpStream` concrete، پارامترهای جنریک `R: BufRead` و `W: Write` می‌گیرد:

```rust
pub fn run_echo<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<usize> {
    // ...
}
```

این همان الگوی «منطق اصلی قابل‌تست، I/O نازک» است. `TcpStream` واقعی `Read` و `Write` را پیاده می‌کند، اما `std::io::Cursor<&[u8]>` و `Vec<u8>` در حافظه هم چنین‌اند. پس تست‌های `tests/echo_test.rs` socket واقعی باز نمی‌کنند و فقط buffer می‌دهند. تنها `serve_once` و `main.rs` به `TcpListener` واقعی دست می‌زنند.

```senpai-visual
{"kind":"network","labels":["client","TcpStream","run_echo","writer","echo پاسخ"]}
```

## هم‌زمانی: یک thread برای هر اتصال

`TcpListener::incoming()` روی thread پذیرنده هر بار یک اتصال می‌دهد. اگر همان‌جا `run_echo` را صدا بزنی، client کند یا ساکت تمام clientهای دیگر را متوقف می‌کند. راه کلاسیک و مناسب آموزش، ساختن **یک OS thread برای هر اتصال** است:

```rust
for stream in listener.incoming() {
    let stream = stream?;
    std::thread::spawn(move || {
        // این اتصال را در thread خودش handle کن
    });
}
```

thread رایگان نیست؛ stack و سهم scheduler واقعی می‌گیرد. به همین دلیل runtimeهایی مانند `tokio` وجود دارند: هزاران اتصال concurrent را روی چند OS thread مدیریت می‌کنند، نه با یک thread برای هر اتصال.

مثل چند اپراتور پشتیبانی است که هرکدام تماس یک مشتری را می‌گیرند. مرز تشبیه: TCP پیام‌محور نیست؛ stream است و مرز `write`های فرستنده را نگه نمی‌دارد.

## تمرین تو

دو `todo!()` در `src/lib.rs` را کامل کن:

۱. `run_echo`: تا EOF، یعنی `read_line` با `Ok(0)`، از `reader` خط بخوان، همان را به `writer` بنویس و مجموع byteهای خوانده‌شده را برگردان.
۲. `serve_once`: دقیقاً یک اتصال را از `TcpListener` بپذیر، با `TcpStream::try_clone` reader و writer بساز و `run_echo` را اجرا کن.

`src/main.rs` از قبل آماده است و روی `127.0.0.1:7878` برای هر اتصال یک thread می‌سازد.

## اجرای واقعی

```sh
cargo run -p p3-01-01-tcp-echo-server &
printf 'hello\nworld\n' | nc 127.0.0.1 7878
# یا بدون netcat:
python3 -c "
import socket
s = socket.create_connection(('127.0.0.1', 7878))
s.sendall(b'hello\n')
print(s.recv(1024))
"
```

## ایست بازرسی

اول `CHECKPOINT.fa.md` و بعد `solution/SOLUTION.fa.md` را بخوان.
