# ۰۵.۳ — `if let` و `while let`

وقتی فقط یک pattern مهم است، match کامل سنگین می‌شود:

```rust
if let Some(r) = maybe_rating {
    println!("rated {r}/10");
} else {
    println!("not rated yet");
}
```

`if let PATTERN = value` بلوک را فقط هنگام match اجرا و داده را bind می‌کند. وقتی همه‌ی حالت‌ها اهمیت دارند `match` بهتر است؛ وقتی فقط یک شکل مهم است `if let` قصد را مستقیم‌تر می‌گوید.

```rust
let mut stack = vec![1, 2, 3];
while let Some(top) = stack.pop() {
    println!("popped {top}");
}
```

`pop()` آخرین عضو را در `Some(T)` می‌دهد و برای بردار خالی `None`. حلقه تا زمانی که pattern `Some(top)` برقرار باشد ادامه می‌یابد و ترتیب ۳، ۲، ۱ دارد.

در بک‌اند می‌توان پیام‌های اختیاری صف را تا `None` مصرف کرد، اما صف async معمولاً API انتظار و cancellation جدا دارد؛ `Vec::pop` مدل کامل broker نیست.

```senpai-visual
{"kind":"queue","labels":["Some(job)","worker","None"]}
```
