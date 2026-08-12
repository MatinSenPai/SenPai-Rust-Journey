# پاسخ تشریحی

```rust
pub fn word_frequency(text: &str) -> HashMap<String, usize> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    for word in text.split_whitespace() {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}
```

خط مهم این تابع `*counts.entry(word.to_lowercase()).or_insert(0) += 1;` است. متد `.entry(key)` یک enum از نوع `Entry` می‌دهد؛ دسته‌ای برای «جایگاه این کلید»، چه آن جایگاه اکنون پر باشد چه نباشد. متد `.or_insert(0)` این وضعیت را تعیین تکلیف می‌کند: اگر کلید وجود نداشته باشد، `0` را درج می‌کند و یک `&mut usize` به همان صفر تازه می‌دهد؛ اگر کلید موجود باشد، یک `&mut usize` به مقدار فعلی برمی‌گرداند و آن را دست‌نخورده می‌گذارد.

پس در هر دو حالت یک ارجاع تغییرپذیر به شمارنده داریم و `*... += 1` از پشت آن ارجاع مقدار را زیاد می‌کند. همه‌ی این مراحل—بررسی، درج احتمالی و تحویل دسترسی تغییرپذیر—با **یک** مراجعه به سازوکار داخلی `HashMap` انجام می‌شود، نه دو مراجعه.

این همان پاسخ پرسش دوم است: نسخه‌ی ساده‌ی `if map.contains_key(&word) { *map.get_mut(&word).unwrap() += 1 } else { map.insert(word, 1) }` در مسیر «کلید موجود است» دو بار کلید را می‌جوید؛ یک بار در `contains_key` و بار دیگر در `get_mut`. رابط entry در هر دو حالت فقط یک بار جست‌وجو می‌کند.

```rust
pub fn top_n(freqs: &HashMap<String, usize>, n: usize) -> Vec<(String, usize)> {
    let mut pairs: Vec<(String, usize)> =
        freqs.iter().map(|(word, count)| (word.clone(), *count)).collect();
    pairs.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    pairs.truncate(n);
    pairs
}
```

درباره‌ی پرسش چهارم: اگر `.then_with(|| a.0.cmp(&b.0))` را حذف کنیم و فقط شمارش را مرتب کنیم، ترتیب نسبی عضوهای هم‌امتیاز به ترتیبی وابسته می‌شود که `.iter()` همان بار تحویل داده است. چون `HashMap` ترتیب پیمایش تضمین‌شده ندارد، این ترتیب ممکن است میان اجراهای برنامه فرق کند.

برای نمونه، تست `top_n_sorts_by_count_descending_then_alphabetically` انتظار دارد `"b"` پیش از `"c"` بیاید، در حالی که شمارش هر دو `5` است. بدون قاعده‌ی شکستن تساوی، این assertion بسته به hash seed فرایند گاهی قبول و گاهی رد می‌شود؛ همان تست ناپایداری که نباید وارد پروژه شود.
