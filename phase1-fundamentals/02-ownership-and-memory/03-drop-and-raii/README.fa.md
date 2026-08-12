# ۰۲.۳ — `Drop` و RAII

## چیزی که قرار است یاد بگیری

```rust
{
    let s = String::from("hello");
} // Rust drops s here
```

با پایان محدوده مالک، مقدار به‌شکل قطعی drop می‌شود. الگوی «گرفتن منبع هنگام ساخت مقدار و آزادسازی خودکار هنگام پایان محدوده» را RAII می‌نامیم. این الگو فقط برای حافظه نیست: file handle، socket، database connection و mutex guard نیز از آن استفاده می‌کنند.

در Python، context manager مثل `with open(...) as f:` پاک‌سازی وابسته به محدوده می‌دهد. در Rust چرخه‌ی عمر منابع معمولاً در نوع و implementation مربوط به `Drop` کپسوله می‌شود.

```rust
struct LoudResource {
    name: String,
}

impl Drop for LoudResource {
    fn drop(&mut self) {
        println!("Cleaning up: {}", self.name);
    }
}
```

## قانون دقیق Rust

- `.drop()` را مستقیم صدا نمی‌زنی؛ برای drop زودهنگام `std::mem::drop(value)` را به‌کار ببر.
- local variableها در یک محدوده معمولاً برخلاف ترتیب تعریف drop می‌شوند.
- fieldهای struct طبق ترتیب تعریف خود drop می‌شوند؛ جزئیات drop order را با حدس و تشبیه جایگزین نکن.
- انتقال، محل پاک‌سازی را به پایان محدوده مالک نهایی می‌برد.

## مثال بک‌اند

وقتی connection guard از محدوده پردازشگر خارج می‌شود، connection به pool برمی‌گردد؛ mutex guard نیز lock را آزاد می‌کند، حتی در خروج زودهنگام با `?`. با این حال `Drop` نمی‌تواند `async fn` باشد، پس پاک‌سازی شبکه‌ای که به `.await` نیاز دارد باید صریح طراحی شود.

## این تشبیه کجا دیگر دقیق نیست؟

RAII تضمین نمی‌کند فرایند در crash سخت یا `std::process::abort` فرصت پاک‌سازی داشته باشد. همچنین می‌توان با `std::mem::forget` عمداً destructor را اجرا نکرد؛ safety حافظه حفظ می‌شود ولی منبع leak ممکن است.

## تمرین

توابع مربوط به `Tracker` در `src/lib.rs` را پیاده کن و ترتیب log را پیش‌بینی کن.

```senpai-visual
{"kind":"ownership","labels":["scope","Drop","آزادسازی"]}
```
