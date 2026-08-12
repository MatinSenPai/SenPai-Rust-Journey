# پاسخ تشریحی

```rust
let (tx, rx) = std::sync::mpsc::channel();
for producer in producers {
    let tx = tx.clone();
    std::thread::spawn(move || {
        tx.send(producer()).unwrap();
    });
}
drop(tx);
let values: Vec<_> = rx.into_iter().collect();
```

هر clone یک producer مستقل است. receiver تا بسته‌شدن channel iterator را تمام نمی‌کند؛ بسته‌شدن یعنی هیچ `Sender` زنده‌ای نمانده باشد. sender اصلی parent خودش پیامی نمی‌فرستد، اما اگر drop نشود همچنان امکان نظری پیام را باز نگه می‌دارد و collect منتظر می‌ماند.

برای یک خروجی، `join()` کافی است. channel زمانی ارزش بیشتری دارد که هر worker چند پیام، progress یا stream نتیجه بدهد و consumer هنگام ادامه‌ی کار آن‌ها را دریافت کند. مالکیت پیام هنگام `send` منتقل می‌شود و نیاز به lockکردن یک collection مشترک کمتر می‌شود.
