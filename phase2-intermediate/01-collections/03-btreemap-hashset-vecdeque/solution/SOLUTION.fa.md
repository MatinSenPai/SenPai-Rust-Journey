# راه‌حل — ۲.۱.۳ `BTreeMap`، `HashSet`، `VecDeque`، `BinaryHeap`

```rust
pub fn sorted_by_title(counts: HashMap<String, u32>) -> BTreeMap<String, u32> {
    let mut sorted = BTreeMap::new();
    for (title, count) in counts {
        sorted.insert(title, count);
    }
    sorted
}
```

هیچ `.sort()`ای اینجا نیست، و نیازی هم نبود. `counts` را با یک `for` معمولی پیمایش می‌کنیم — به هر ترتیبی که `HashMap` تصادفاً بدهد — و هر جفت را با `.insert()` داخلِ یک `BTreeMap` تازه می‌گذاریم. مرتب‌بودن، ویژگیِ *جایی است که داده تهش می‌رود*، نه ترتیبی که رسیده: `BTreeMap` با هر `insert()` جایگاهِ درستِ کلید را در درختش دوباره پیدا می‌کند، مهم نیست کلیدها به چه ترتیبی رسیده باشند.

```rust
pub fn shows_in_year_range(
    releases: &BTreeMap<u32, String>,
    start: u32,
    end: u32,
) -> Vec<String> {
    if start > end {
        return Vec::new();
    }
    let mut titles = Vec::new();
    for (_year, title) in releases.range(start..=end) {
        titles.push(title.clone());
    }
    titles
}
```

نگهبانِ اول دقیقاً همان چیزی است که در «خطاهایی که خواهی دید» دیدی: اگر `.range(start..=end)` را بدونِ این چک صدا بزنی و `start` از `end` بزرگ‌تر باشد، پنیک می‌گیری. اینجا، به‌جایِ پنیک، مشخصاتِ تابع می‌گوید یک `Vec` خالی برگردان — پس همان‌جا، قبل از رسیدن به `.range()`، برمی‌گردیم. بعدِ نگهبان، `.range(start..=end)` مستقیم می‌رود سراغِ همان بخشِ درخت؛ `title.clone()` لازم است چون `releases` فقط قرض گرفته شده و `titles` باید مالکِ رشته‌های خودش باشد.

```rust
pub fn shared_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    let mut shared = HashSet::new();
    for genre in a.intersection(b) {
        shared.insert(genre.clone());
    }
    shared
}

pub fn exclusive_genres(a: &HashSet<String>, b: &HashSet<String>) -> HashSet<String> {
    let mut exclusive = HashSet::new();
    for genre in a.symmetric_difference(b) {
        exclusive.insert(genre.clone());
    }
    exclusive
}
```

هر دو تابع یک شکل دارند: متدِ جبرِ مجموعه (`.intersection()` یا `.symmetric_difference()`) یک پیمایشگر از `&String`های قرض‌گرفته‌شده از `a` می‌دهد؛ چون امضای تابع یک `HashSet<String>`ِ *مالک* قول داده، هر عضو را کلون می‌کنیم و داخلِ یک `HashSet` تازه می‌گذاریم. هیچ‌جا `.collect()` لازم نبود — یک `for` و یک `.insert()` همانِ کاری را می‌کنند که `.cloned().collect()` می‌کرد، فقط صریح‌تر.

```rust
pub struct WatchQueue {
    queue: VecDeque<String>,
}

impl WatchQueue {
    pub fn new() -> Self {
        Self { queue: VecDeque::new() }
    }

    pub fn enqueue(&mut self, title: String) {
        self.queue.push_back(title);
    }

    pub fn watch_next(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    pub fn watch_next_priority(&mut self, title: String) {
        self.queue.push_front(title);
    }
    // len و is_empty فقط self.queue.len() و self.queue.is_empty() را پاس می‌دهند.
}
```

`WatchQueue` فقط یک پوششِ نازک دورِ `VecDeque` است — هر متد مستقیم متدِ هم‌نامِ `VecDeque` را صدا می‌زند. نکته‌ی اصلی در انتخابِ نوع است، نه در کد: `enqueue` به `push_back` می‌رود (ته صف، `O(1)`) و `watch_next_priority` به `push_front` (اولِ صف، هم `O(1)`) — اگر این پوشش دورِ `Vec` بود، `watch_next_priority` مجبور بود از `.insert(0, title)` استفاده کند، که `O(n)` است.

```rust
pub struct WatchPriorityQueue {
    queue: BinaryHeap<(u32, String)>,
}

impl WatchPriorityQueue {
    pub fn new() -> Self {
        Self { queue: BinaryHeap::new() }
    }

    pub fn add(&mut self, priority: u32, title: String) {
        self.queue.push((priority, title));
    }

    pub fn watch_highest_priority(&mut self) -> Option<String> {
        self.queue.pop().map(|(_priority, title)| title)
    }
}
```

`add` هر جفت را بدونِ هیچ منطقِ اضافه‌ای `push` می‌کند — تاپل‌ها خودشان از قبل به‌ترتیبِ واژه‌نامه‌ای مقایسه می‌شوند، اول `priority`، بعد `title` فقط برایِ شکستنِ تساوی. `watch_highest_priority` بیشینه را `pop` می‌کند و با `.map()` روی `Option`ِ نتیجه (نه روی یک Iterator — این همان ترکیب‌گرِ `Option` از [۱.۶.۲](../../../../phase1-fundamentals/06-absence-and-failure/02-option-combinators/README.fa.md) است) فقط `title` را نگه می‌دارد و `priority` را دور می‌ریزد.

## این درس واقعاً درباره‌ی چه بود

- **نوعِ برگشتیِ `BTreeMap` به‌جایِ `HashMap` یعنی پیمایشِ مرتب، رایگان از هر سمتی که insert کرده باشی** — `sorted_by_title` این را بدونِ هیچ `.sort()`ای ثابت کرد.
- **نگهبان قبل از `.range()`، نه بعدش** — دقیقاً همان الگویِ «اول موردِ استثنایی، بعد بقیه‌ی تابع» که از [۱.۱.۵](../../../../phase1-fundamentals/01-foundations/05-control-flow/README.fa.md) می‌شناسی.
- **متدهایِ جبرِ مجموعه پیمایشگر می‌دهند، نه یک `HashSet` تازه** — یک `for` و یک `.insert()`، بدونِ `.collect()`، دقیقاً همان نتیجه را می‌سازد.
- **انتخابِ `VecDeque` به‌جایِ `Vec` فقط در انتخابِ نوع بود، نه در پیچیدگیِ کد** — همان چهار خط، ولی حالا `O(1)` در هر دو سر.
- **تاپل‌ها یک `BinaryHeap` را با صفرخط منطقِ اضافه به یک صفِ اولویت‌دار تبدیل می‌کنند** — چون Rust خودش می‌داند چطور دو تاپل را مقایسه کند.
