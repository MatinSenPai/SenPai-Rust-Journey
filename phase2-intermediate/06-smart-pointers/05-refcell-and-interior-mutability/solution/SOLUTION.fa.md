# راه‌حل — ۲.۶.۵ `RefCell`، `Cell` و معامله‌ی پنیکِ زمانِ اجرا

```rust
impl ClubStats {
    pub fn new() -> Self {
        ClubStats {
            members_online: Cell::new(0),
            watch_log: RefCell::new(Vec::new()),
        }
    }

    pub fn member_joined(&self) {
        self.members_online.set(self.members_online.get() + 1);
    }

    pub fn members_online(&self) -> u32 {
        self.members_online.get()
    }

    pub fn log_episode(&self, title: &str) {
        self.watch_log.borrow_mut().push(title.to_string());
    }

    pub fn watch_log(&self) -> Vec<String> {
        self.watch_log.borrow().clone()
    }
}

impl SharedHypeMeter {
    pub fn new() -> Self {
        SharedHypeMeter {
            inner: Rc::new(RefCell::new(0)),
        }
    }

    pub fn hype_up(&self, amount: i32) {
        *self.inner.borrow_mut() += amount;
    }

    pub fn hype(&self) -> i32 {
        *self.inner.borrow()
    }
}
```

هیچ‌کدامِ این‌ها چیزی فراتر از آنچه در «مفهوم» دیدی نمی‌خواست — همان دو ابزار، این‌بار رویِ یک دامنه‌ی کمی متفاوت.

## `ClubStats` — `Cell` برایِ شمارنده، `RefCell` برایِ لاگ

`member_joined`/`members_online` دقیقاً همان شکلِ `Cell` است که رویِ `PageViews` دیدی: `.set(.get() + 1)` و `.get()`. `log_episode`/`watch_log` هم دقیقاً همان شکلِ `RefCell` است که رویِ `WatchLog` دیدی: `.borrow_mut().push()` و `.borrow().clone()`. هر دو فیلد داخلِ یک ساختار زندگی می‌کنند، و هر متدی رویِ آن فقط `&self` می‌گیرد — چون کلِ نکته همین است: یک `ClubStats` که از پشتِ یک `&ClubStats` ساده (یا بعداً از پشتِ یک `Rc<ClubStats>`) به‌دستت رسیده، همچنان قابلِ به‌روزرسانی است.

`watch_log()` به‌جایِ برگرداندنِ خودِ گارد، `Vec<String>` را از داخلِ `Ref` بیرون کلون می‌کند — گارد از `self` قرض گرفته و نمی‌تواند از پایانِ متد جان به‌در ببرد، پس فراخواننده به‌جایش یک کپیِ مالکانه می‌گیرد. این یعنی یک `Ref<Vec<String>>` که با یک `.clone()` تبدیل شده به `Vec<String>` — دقیقاً همان خواندنِ «فقط نگاه کن، دست نزن» که `.replace()`/`.take()` رویِ یک `Cell` نمی‌توانست بدهد.

## `SharedHypeMeter` — همان ترکیبِ کلاسیکِ `Rc<RefCell<T>>`

متغیرِ `self.inner` از نوعِ `Rc<RefCell<i32>>` است — `.borrow_mut()` از طریقِ همان ارجاعِ اشتراکیِ `Rc` رویِ `RefCell` کار می‌کند (هیچ‌جا نیازی به `&mut self` نیست، چون `RefCell` وظیفه‌ی چکِ تغییرپذیری را موکول کرده به زمانِ اجرا)، و در نهایت یک `RefMut<i32>` برمی‌گرداند که `*` بازش می‌کند تا خودِ `i32` را بخوانی یا بنویسی. گاردهایِ `RefMut`/`Ref` این‌جا موقتی‌اند — با پایانِ هر دستور drop می‌شوند — و دقیقاً همین است که چرا صدازدنِ پشتِ‌سرِهمِ `hype_up()` هرگز پنیک نمی‌گیرد: قرضِ هر فراخوانی، پیش از شروعِ فراخوانیِ بعدی، کاملاً آزاد می‌شود.

`.clone()` رویِ یک `SharedHypeMeter` همان `Clone`ِ مشتق‌شده است، که فیلدِ `Rc` را کلون می‌کند — ارزان، و به همان تخصیصِ رویِ هیپ اشاره می‌کند (نگاه کن به ۲.۶.۳). دقیقاً همین است که تستِ `clones_share_the_same_underlying_total` را سبز می‌کند: `original` و `handle` دو اشاره‌گرِ `Rc` به یک `RefCell<i32>`اند، نه دو سنجاقِ هیجانِ مستقل.

## چیزی که این درس واقعاً درباره‌اش بود

- **قاعده خودش جابه‌جا نمی‌شود، مگر بهش بگویی.** `Cell`/`RefCell` فقط رویِ همان فیلدهایی ظاهر شدند که واقعاً باید از پشتِ `&self` تغییر کنند؛ هیچ‌چیزِ دیگری در هیچ‌کدام از دو ساختار نیازی به دست‌خوردن نداشت.
- **یک گارد یک مقدارِ موقتی است، و موقتی‌ها زود drop می‌شوند.** خطِ `*self.inner.borrow_mut() += amount;` هیچ‌وقت رویِ فراخوانی‌هایِ پیاپی پنیک نمی‌گیرد، دقیقاً چون `RefMut`ی که می‌سازد از عمرِ همان دستور فراتر نمی‌رود.
- **`Rc<RefCell<T>>` دو وظیفه‌ی جدا رویِ هم چیده‌شده است.** `Rc` تصمیم می‌گیرد چند مالک هست؛ `RefCell` تصمیم می‌گیرد آیا هرکدام می‌تواند بنویسد یا نه. از دست‌دادنِ هرکدام نصفِ الگو را از بین می‌برد — `Rc<i32>` تنها فقط‌خواندنی می‌ماند، و یک `RefCell<i32>`ِ تنها (بدونِ `Rc`) هم باز فقط یک مالک دارد.
