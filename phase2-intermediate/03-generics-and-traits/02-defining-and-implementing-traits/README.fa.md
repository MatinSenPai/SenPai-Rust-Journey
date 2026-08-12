# ۰۳.۲ — تعریف و پیاده‌سازی trait

در درس قبل از یک **bound** مانند `T: PartialOrd` استفاده کردی؛ قراردادی که کتابخانه‌ی استاندارد از قبل تعریف کرده بود. این درس درباره‌ی نوشتن **trait خودت** است: قراردادی که تو طراحی می‌کنی و structهای خودت یا دیگران می‌توانند متعهد به اجرای آن شوند.

## تعریف trait

```rust
pub trait Summarize {
    fn title(&self) -> String;

    fn summary(&self) -> String {
        format!("{} (no summary available)", self.title())
    }
}
```

trait مجموعه‌ای از امضاهای متد است. `title` بدنه ندارد؛ هر نوعی که `Summarize` را پیاده کند **باید** `title` خودش را ارائه دهد. اما `summary` بدنه دارد و یک **پیاده‌سازی پیش‌فرض (Default implementation)** است. هر نوع پیاده‌کننده همین `summary` را رایگان می‌گیرد، مگر اینکه نسخه‌ی خودش را جایگزین کند.

دقت کن بدنه‌ی پیش‌فرض `summary`، متد `self.title()` را صدا می‌زند؛ در لحظه‌ی تعریف trait هنوز هیچ نوع concreteای وجود ندارد. این کد مجاز است چون خود trait تضمین می‌کند همه‌ی پیاده‌کننده‌ها `title` دارند. کامپایلر برای امن‌بودن فراخوانی لازم نیست نوع concrete را بداند؛ کافی است بداند نوع نهایی قرارداد trait را برآورده می‌کند.

```senpai-visual
{"kind":"concept","labels":["قرارداد Summarize","AnimeSeries","MangaVolume","summary پیش‌فرض"]}
```

## پیاده‌سازی trait

```rust
pub struct AnimeSeries {
    pub title: String,
    pub episodes: u32,
}

impl Summarize for AnimeSeries {
    fn title(&self) -> String {
        self.title.clone()
    }

    fn summary(&self) -> String {
        format!("{} — {} episodes", self.title, self.episodes)
    }
}
```

`AnimeSeries` هم `title` اجباری و هم `summary` اختصاصی را می‌دهد و بدنه‌ی پیش‌فرض را override می‌کند. struct دیگری می‌تواند فقط `title` را بنویسد و `summary` پیش‌فرض را بی‌هیچ تغییری به ارث ببرد؛ در تمرین همین کار را برای `MangaVolume` انجام می‌دهی.

## مقایسه با پایتون

نزدیک‌ترین ابزار پایتون احتمالاً **ABC**، یعنی `abc.ABC` همراه `@abstractmethod`، یا `Protocol` است که گاهی با **mixin** ترکیب می‌شود:

```python
class Summarize(ABC):
    @abstractmethod
    def title(self) -> str: ...

    def summary(self) -> str:
        return f"{self.title()} (no summary available)"
```

ایده یکی است: `title` abstract و اجباری است و `summary` بدنه‌ی پیش‌فرض mixin را دارد. تفاوت در زمان بررسی است. پایتون نبودن override برای `title` را وقتی می‌فهمد که نخستین بار بخواهی class را instantiate کنی. با یک `Protocol` ساده حتی ممکن است تا رسیدن اجرای برنامه به همان فراخوانی ناموجود چیزی متوقف نشود.

کامپایلر Rust هر بلوک `impl Summarize for X` را هنگام compile بررسی می‌کند. اگر `X` متد `title` نداشته باشد، `cargo build` پیش از اجرای هر کدی شکست می‌خورد و نام متد گمشده را دقیق می‌گوید.

در backend می‌توانی `Summarize` را مثل قرارداد یک formatter گزارش ببینی: سفارش، فاکتور و رخداد log هرکدام شکل متفاوتی دارند، اما تا وقتی قرارداد را پیاده کنند کد جنریک می‌تواند خلاصه‌شان را بگیرد. مرز تشبیه اینجاست که trait فقط «فهرست توصیه‌ها» نیست؛ کامپایلر اجرای کامل قرارداد را الزام می‌کند.

## تمرین تو

در `src/lib.rs`، trait به نام `Summarize` را کامل کن؛ آن را برای `AnimeSeries` با `summary` اختصاصی و برای `MangaVolume` فقط با `summary` پیش‌فرض پیاده کن؛ سپس `print_all_summaries` را بنویس.

## ایست بازرسی

ابتدا `CHECKPOINT.fa.md` و سپس `solution/SOLUTION.fa.md` را بخوان.
