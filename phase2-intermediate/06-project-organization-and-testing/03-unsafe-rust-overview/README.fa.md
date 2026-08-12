# ۰۶.۳ — آشنایی با Rust ناامن (`unsafe`)

## `unsafe` borrow checker را خاموش نمی‌کند

بلوک `unsafe` فقط پنج توانایی اضافه می‌دهد: dereferenceکردن raw pointer، فراخوانی تابع unsafe، دسترسی یا تغییر static mutable، پیاده‌سازی trait unsafe و دسترسی به fieldهای `union`. باقی بررسی‌های نوع، ownership و lifetime همچنان فعال‌اند.

## raw pointerهای `*const T` و `*mut T`

ساخت raw pointer اغلب safe است؛ **dereference** آن unsafe است، چون compiler نمی‌تواند معتبر، aligned و زنده‌بودن مقصد یا نبود alias ناسازگار را ثابت کند. `unsafe` یعنی برنامه‌نویس این proof را می‌پذیرد، نه اینکه عملیات خودکار امن شود.

## تمرین `split_at_mut`

borrow checker نمی‌تواند دو indexing پویا را همیشه non-overlapping تشخیص دهد. implementation استاندارد با `as_mut_ptr`، `ptr.add(mid)` و `from_raw_parts_mut` دو slice می‌سازد. شرط حیاتی `mid <= len` است و کامنت `// SAFETY:` باید توضیح دهد چرا pointerها داخل همان allocation و بازه‌ها جدا هستند.

```senpai-visual
{"kind":"borrowing","labels":["slice کامل","0..mid","mid..len","دو &mut جدا"]}
```

مثال بریدن سینی به دو نیمه مفید است، اما مرزش مهم است: کامپایلر خط برش raw pointer را نمی‌بیند؛ صحت assertion و محاسبه کاملاً مسئولیت توست. اشتباه می‌تواند undefined behavior باشد، نه خطای معمول.

## تمرین تو

`split_at_mut_demo` را با کوچک‌ترین بلوک unsafe و کامنت SAFETY دقیق کامل کن.
