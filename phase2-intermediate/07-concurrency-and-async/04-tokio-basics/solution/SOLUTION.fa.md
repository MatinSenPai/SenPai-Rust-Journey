# پاسخ تشریحی

الگوی concurrency چنین است:

```rust
let mut handles = Vec::new();
for item in items {
    handles.push(tokio::spawn(fetch_simulated(item)));
}
let mut results = Vec::new();
for handle in handles {
    results.push(handle.await.unwrap());
}
```

ساخت همه‌ی taskها پیش از await نتیجه‌ها باعث می‌شود timerها و انتظارهای I/O overlap کنند. اگر همان حلقه‌ی اول مستقیماً `fetch_simulated(item).await` کند، هر درخواست پس از قبلی آغاز می‌شود؛ asyncبودن syntax تضمین concurrency نیست.

`#[tokio::test]` یک test همگام سازگار با harness تولید می‌کند، runtime می‌سازد و future بدنه را `block_on` می‌کند. `JoinHandle` خروجی `Result<T, JoinError>` دارد؛ panic یا لغو task شاخه‌ی `Err` را می‌سازد. برای کار CPU-heavy انبوه، task عادی Tokio مناسب نیست چون executor را block می‌کند؛ برای تعداد زیاد انتظار شبکه‌ای مناسب است.
