# ۰۷.۲ — channel و پیام‌رسانی

## `mpsc`: چند تولیدکننده، یک مصرف‌کننده

`std::sync::mpsc::channel()` یک `Sender<T>` و `Receiver<T>` می‌دهد. `Sender` قابل clone است تا چند thread پیام بفرستند، اما receiver یکتا است. ارسال معمولاً مالکیت `T` را move می‌کند؛ پس پس از `tx.send(value)` دیگر فرستنده به همان value دسترسی ندارد.

## `rx` به‌عنوان iterator و اهمیت dropشدن senderها

`for value in rx` تا رسیدن پیام‌ها صبر می‌کند و فقط وقتی پایان می‌یابد که **همه‌ی** senderها drop شده باشند. اگر sender اصلی parent را پس از ساخت cloneها نگه داری، receiver تصور می‌کند پیام دیگری ممکن است برسد و حلقه برای همیشه منتظر می‌ماند. `drop(tx)` بخشی از protocol پایان است.

```senpai-visual
{"kind":"queue","labels":["producer ۱","producer ۲","channel<T>","consumer","drop senderها"]}
```

## channel یا `Mutex`؟

channel وقتی طبیعی است که workerها نتیجه یا eventهای مستقل تولید و مالکیتشان را به consumer واگذار می‌کنند. `Mutex` وقتی طبیعی است که همه باید یک state واحد را درجا تغییر دهند. برای job queue و progress update معمولاً message passing روشن‌تر است؛ برای counter مشترک یک lock کوچک ساده‌تر است.

صف نانوایی تشبیه مناسبی است: چند باجه شماره تحویل می‌دهند و یک نمایشگر مصرف می‌کند. مرز تشبیه: channel ظرفیت، blocking و قطع اتصال دقیق دارد و پایانش با dropشدن senderها تعیین می‌شود.

## تمرین تو

`compute_async_sum` و `collect_from_producers` را پیاده کن.
