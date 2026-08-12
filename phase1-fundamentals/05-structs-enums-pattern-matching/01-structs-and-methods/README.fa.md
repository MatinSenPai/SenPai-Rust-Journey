# ۰۵.۱ — struct و متد

```rust
struct Anime {
    title: String,
    episodes: u32,
    is_completed: bool,
}
```

struct fieldهای نام‌دار و typed را کنار هم می‌گذارد؛ نزدیک به `@dataclass` در Python، اما نوع‌ها در کامپایل بررسی می‌شوند و نمونه پیش‌فرض immutable است.

```rust
impl Anime {
    fn new(title: &str, episodes: u32) -> Self {
        Anime { title: title.to_string(), episodes, is_completed: false }
    }

    fn describe(&self) -> String {
        format!("{} ({} episodes)", self.title, self.episodes)
    }

    fn mark_completed(&mut self) {
        self.is_completed = true;
    }
}
```

تابع بدون `self` یک associated تابع است و طبق قرارداد `new` نامیده می‌شود؛ constructor ویژه‌ی زبان نیست. `&self` فقط می‌خواند، `&mut self` تغییر انحصاری می‌دهد و `self` مالکیت نمونه را مصرف می‌کند. `Self` همان نوع بلوک `impl` است. در field init shorthand، نام variable و field یکسان فقط یک‌بار نوشته می‌شود.

مثال بک‌اند: `Order::new` ناوردایی اولیه را می‌سازد، `order.describe()` فقط می‌خواند و `order.mark_paid()` وضعیت را تغییر می‌دهد. public کردن همه‌ی fieldها ساده است ولی ناوردایی‌ها را ضعیف می‌کند؛ متد‌های کنترل‌شده encapsulation بهتری می‌دهند.

```senpai-visual
{"kind":"concept","labels":["Book::new","&self","&mut self"]}
```

متد‌های `Book` در `src/lib.rs` را پیاده کن.
