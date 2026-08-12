# ۰۲.۲ — adapterهای iterator

این درس یکی از مهم‌ترین تغییر عادت‌ها برای برنامه‌نویس پایتون است. جایی که در پایتون سراغ list comprehension می‌روی، در Rust اصطلاحی زنجیره‌ای از **adapterهای iterator** می‌سازیم.

```senpai-visual
{"kind":"concept","labels":["داده‌ی ورودی","filter","map","collect"]}
```

## سه روش پیمایش، وابسته به مالکیت

```rust
let titles = vec!["Frieren".to_string(), "Mob Psycho".to_string()];

for t in titles.iter() {}      // t: &String — هر عضو قرض گرفته می‌شود و بعداً هنوز `titles` را داریم
for t in titles.iter_mut() {}  // t: &mut String — قرض تغییرپذیر و نیازمند `let mut titles`
for t in titles.into_iter() {} // t: String — `titles` مصرف می‌شود و پس از حلقه دیگر وجود ندارد
```

عبارت `for t in &titles` شکل کوتاه `.iter()` و `for t in titles` بدون `&` شکل کوتاه `.into_iter()` است. این همان تفاوت borrow و move در فاز یک است. حالت پیش‌فرض ذهنی‌ات `.iter()` باشد، چون بیشتر وقت‌ها فقط می‌خواهی داده را **ببینی**. فقط وقتی `.into_iter()` را انتخاب کن که واقعاً می‌خواهی مالکیت همه‌ی عضوها را واگذار کنی و دیگر به مجموعه‌ی اصلی نیازی نداری.

## adapterها تنبل‌اند؛ تا مصرف نکنی کاری انجام نمی‌شود

```rust
let doubled = vec![1, 2, 3].iter().map(|n| n * 2); // هنوز هیچ ضربی انجام نشده است
let doubled: Vec<i32> = doubled.collect();          // حالا map اجرا می‌شود
```

متدهای `.map()`، `.filter()`، `.enumerate()`، `.zip()`، `.take()`، `.skip()` و `.take_while()` iterator تازه‌ای می‌سازند که به‌تنهایی کاری نمی‌کند؛ فقط pipeline را توصیف می‌کنند. تنها یک مصرف‌کننده مانند `.collect()`، `.sum()`، `.fold()`، `.count()` یا `.for_each()` مقدارها را از pipeline عبور می‌دهد و کار را اجرا می‌کند.

این تفاوت با پایتون واقعی و مهم است:

```python
doubled = [n * 2 for n in [1, 2, 3]]  # همین‌جا و فوری اجرا می‌شود
```

list comprehension پایتون همان لحظه اجرا می‌شود. زنجیره‌ی iterator در Rust بیشتر شبیه برنامه‌ای است که ابتدا می‌سازی. می‌توانی `.enumerate()`، `.map()` و `.filter()` را پشت سر هم قرار دهی، بی‌آنکه در هر گام یک `Vec` میانی ساخته شود. وقتی نتیجه را مصرف کنی، کل pipeline عضو‌به‌عضو و در یک گذر اجرا می‌شود.

## `.fold()` — ابزار عمومی برای کاهش چند مقدار به یک نتیجه

```rust
let total = vec![1, 2, 3].iter().fold(0, |acc, n| acc + n); // 6
```

عبارت `.fold(initial, |accumulator, item| ...)` دقیقاً همتای این حلقه است:

```rust
let mut total = 0;
for n in vec![1, 2, 3].iter() {
    total = total + n;
}
```

می‌توانی `.sum()` را نامی ساده‌تر برای حالت بسیار رایج `.fold(0, |acc, n| acc + n)` بدانی. هر وقت می‌خواهی همه‌ی عضوها را در یک نتیجه ترکیب کنی اما منطق کار جمع، ضرب یا شمارش ساده نیست، سراغ `.fold()` برو.

## `.collect()` باید بداند چه چیزی بسازد

متد `.collect()` نسبت به `FromIterator` جنریک است؛ از یک iterator مشابه می‌تواند `Vec`، `HashMap`، `String` و نوع‌های دیگر بسازد. پس Rust برای انتخاب مقصد به annotation نوع نیاز دارد:

```rust
let v: Vec<i32> = (1..5).collect();          // نوع binding را بنویس، یا...
let v = (1..5).collect::<Vec<i32>>();        // ...از turbofish یعنی `::<>` استفاده کن
```

اگر هر دو را فراموش کنی، خطای `type annotations needed` می‌گیری. این خطا نشانه‌ی خرابی نیست؛ فقط باید به Rust بگویی چه مجموعه‌ای می‌سازی.

## مقایسه‌ی حلقه و زنجیره

تابع `total_episodes` را می‌توانیم هر دو شکل بنویسیم تا روشن شود iterator جادو نیست؛ همان حلقه با بیانی توصیفی‌تر است:

```rust
// حلقه‌ی دستی با accumulator
fn total_episodes_loop(shows: &[(String, u32)]) -> u32 {
    let mut total = 0;
    for (_, episodes) in shows {
        total += episodes;
    }
    total
}

// زنجیره‌ی iterator
fn total_episodes_iter(shows: &[(String, u32)]) -> u32 {
    shows.iter().map(|(_, episodes)| episodes).sum()
}
```

هر دو با کار یکسان نتیجه‌ی یکسان می‌دهند. نسخه‌ی زنجیره‌ای دو گام هدف را نام می‌برد: «هر عنوان را به تعداد قسمت‌هایش تبدیل کن» و «آن‌ها را جمع کن». دیگر لازم نیست جزئیات دفترداری مانند تعریف و تغییر `total` را بنویسی. وقتی خواندن adapterها برایت عادی شود، این شکل نیت کد را سریع‌تر منتقل می‌کند، بی‌آنکه اتفاق واقعی را پنهان کند.

در backend هم همین الگو را زیاد می‌بینی: فهرست سفارش‌ها را filter می‌کنی، به DTO تبدیل می‌کنی و collect می‌کنی. فقط یادت باشد تشبیه «خط تولید» تا جایی دقیق است که iterator تنبل است؛ اگر مصرف‌کننده‌ای نداشته باشد، هیچ سفارشی وارد خط نمی‌شود.

## تمرین تو

پنج تابع `src/lib.rs` را پیاده‌سازی کن.

## ایست بازرسی

بعد از تمرین، `CHECKPOINT.fa.md` و سپس `solution/SOLUTION.fa.md` را بخوان.
