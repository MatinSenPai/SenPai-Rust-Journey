# ۰۱.۲ — `BTreeMap`، `HashSet` و `VecDeque`

در درس قبل `Vec` و `HashMap` را دیدی. اینجا جعبه‌ابزار مجموعه‌ها را با سه شکل دیگر کامل می‌کنیم: نگاشت مرتب، مجموعه‌ی بدون تکرار و صفی که کار در هر دو سر آن سریع است. مثال‌ها همچنان از داده‌های فهرست تماشا می‌آیند.

```senpai-visual
{"kind":"queue","labels":["BTreeMap: مرتب","HashSet: یکتا","VecDeque: دو سر صف"]}
```

## `BTreeMap<K, V>` — شبیه `HashMap`، اما مرتب

رابط `BTreeMap` تقریباً همان رابط `HashMap` است: `.insert`، `.get` و رابط entry در هر دو به یک شکل کار می‌کنند. تفاوت این است که `BTreeMap` کلیدهایش را در ساختار داخلی به‌ترتیب نگه می‌دارد؛ پس پیمایش آن همیشه عضوها را بر اساس کلید و به‌شکل صعودی تحویل می‌دهد. برخلاف تابع `top_n` درس قبل، دیگر لازم نیست خودت صریحاً sort کنی.

البته این قطعیت رایگان نیست. جست‌وجو و درج در `BTreeMap` برابر `O(log n)` است، چون باید درخت پیموده شود؛ در حالی که میانگین این عملیات در `HashMap` برابر `O(1)` است. قاعده‌ی عملی: معمولاً از `HashMap` شروع کن و وقتی واقعاً به پیمایش مرتب نیاز داری سراغ `BTreeMap` برو؛ مثلاً برای چاپ گزارش یا پرس‌وجوی بازه‌ای مانند «همه‌ی فیلم‌های منتشرشده از ۲۰۲۰ تا ۲۰۲۳». متد `BTreeMap::range` از چنین کاری پشتیبانی می‌کند، اما `HashMap` نه.

```rust
use std::collections::{BTreeMap, HashMap};

let counts: HashMap<String, u32> = HashMap::from([
    ("naruto".to_string(), 4),
    ("bleach".to_string(), 2),
]);
let sorted: BTreeMap<String, u32> = counts.into_iter().collect();
// پیمایش `sorted` حالا همیشه "bleach" را پیش از "naruto" می‌دهد.
```

## `HashSet<T>` — همان عملیات مجموعه‌ای آشنا از پایتون

اگر با ORM جنگو یا `set` پایتون کار کرده باشی، این نوع برایت آشناست. `HashSet<T>` مقدارهای یکتا را بدون داده‌ی همراه نگه می‌دارد—از نظر ذهنی می‌توانی آن را شبیه `HashMap<T, ()>` ببینی—و همان جبر مجموعه‌های پایتون را در اختیارت می‌گذارد:

| Python | Rust |
|---|---|
| `a & b` | `a.intersection(&b)` |
| `a \| b` | `a.union(&b)` |
| `a - b` | `a.difference(&b)` |
| `a ^ b` | `a.symmetric_difference(&b)` |

یک تفاوت مهم وجود دارد: متدهای مجموعه در Rust مستقیماً یک `HashSet` تازه نمی‌سازند؛ آن‌ها iteratorای از ارجاع‌های قرض‌گرفته‌شده برمی‌گردانند. اگر مجموعه‌ای با مالکیت مستقل می‌خواهی، معمولاً نتیجه را با `.collect()` جمع می‌کنی:

```rust
use std::collections::HashSet;

let a: HashSet<&str> = HashSet::from(["action", "comedy", "isekai"]);
let b: HashSet<&str> = HashSet::from(["comedy", "drama"]);

let shared: HashSet<&str> = a.intersection(&b).copied().collect();
// shared == {"comedy"}
```

## `VecDeque<T>` — سریع در هر دو سر

افزودن یا برداشتن عضو از **انتهای** `Vec` سریع و `O(1)` است، اما همین کار در **ابتدا** `O(n)` هزینه دارد. `insert(0, x)` یا `remove(0)` باید همه‌ی عضوهای دیگر را یک خانه جابه‌جا کند.

اگر لازم است از هر دو سر ساختار با سرعت عضو اضافه یا حذف کنی—مثلاً صف تماشا که عنوان‌ها به انتهایش می‌روند و گاهی یک عنوان فوری باید ابتدای صف بیاید—از `VecDeque<T>` یا صف دوسر (double-ended queue) استفاده کن. عملیات هر دو سر آن `O(1)` است:

```rust
use std::collections::VecDeque;

let mut queue: VecDeque<&str> = VecDeque::new();
queue.push_back("Frieren");   // افزودن به انتهای صف — O(1)
queue.push_front("Bocchi");   // رفتن به ابتدای صف — O(1)
let next = queue.pop_front(); // Some("Bocchi") — O(1)
```

برای یک مثال ایرانی، صف نانوایی را تصور کن: مشتری عادی به انتهای صف می‌رود و سفارش از ابتدای صف تحویل می‌شود. `VecDeque` برای این الگو مناسب است. بااین‌حال، این تشبیه تا جایی دقیق است که فقط دو سر صف مهم باشند؛ پیدا کردن یا حذف یک عضو در میانه‌ی `VecDeque` همچنان کار ثابتی نیست.

## تمرین تو

تابع‌ها و struct به نام `WatchQueue` را در `src/lib.rs` پیاده‌سازی کن:

- `sorted_by_key`: یک `HashMap` را به `BTreeMap` تبدیل کن تا پیمایش مرتب را «رایگان» بگیری.
- `shared_genres`: اشتراک دو مجموعه‌ی ژانر را بساز.
- `exclusive_genres`: تفاضل متقارن؛ ژانرهایی که فقط در یکی از دو مجموعه‌اند، نه هر دو.
- `WatchQueue`: پوششی کوچک دور `VecDeque<String>` برای مدل‌کردن صف «بعدی‌ها». `enqueue` عنوان را به انتها می‌برد، `watch_next` از ابتدا برمی‌دارد و `watch_next_priority` یک عنوان را فوری به ابتدای صف می‌آورد.

## ایست بازرسی

اول `CHECKPOINT.fa.md` را پاسخ بده، بعد `solution/SOLUTION.fa.md` را بخوان.
