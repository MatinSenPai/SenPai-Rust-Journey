# ۰۱.۲ — نوع‌های scalar و compound

Rust statically typed است: نوع هر مقدار در زمان کامپایل معلوم و برای همان اتصال ثابت است.

| نوع | کاربرد |
|---|---|
| `i8`…`i128`, `isize` | integer علامت‌دار |
| `u8`…`u128`, `usize` | integer بدون علامت؛ `usize` برای index/طول |
| `f32`, `f64` | اعشاری؛ پیش‌فرض `f64` |
| `bool` | `true` یا `false` |
| `char` | یک Unicode scalar مقدار، همیشه ۴ byte در حافظه |

integer overflow را آگاهانه مدیریت کن. رفتار plain arithmetic به profile و تنظیم overflow وابسته است؛ برای domain حساس از `checked_*`، `saturating_*` یا `wrapping_*` استفاده کن.

- tuple اندازه‌ی ثابت و نوع‌های متفاوت دارد: `(f64, f64)`.
- آرایه اندازه‌ی ثابت و عضو هم‌نوع دارد: `[i32; 3]`. اندازه بخشی از نوع است.

مثال ایرانی: کد وضعیت HTTP را می‌توان `u16` نگه داشت، اما مبلغ ریالی شاید به نوع بزرگ‌تر و سیاست overflow روشن نیاز داشته باشد. `char` را با «حرف دیداری» یکی نگیر؛ یک grapheme مثل emoji ترکیبی می‌تواند چند `char` باشد.

```senpai-visual
{"kind":"concept","labels":["scalar","tuple","array"]}
```
