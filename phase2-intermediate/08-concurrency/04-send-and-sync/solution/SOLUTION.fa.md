# راه‌حل

```rust
pub fn label_from_thread(label: String) -> String {
    let handle = thread::spawn(move || format!("{label} (from thread)"));
    handle.join().unwrap()
}

#[derive(Clone)]
pub struct SharedCounter {
    value: Arc<Mutex<i32>>,
}

impl SharedCounter {
    pub fn new(start: i32) -> Self {
        SharedCounter {
            value: Arc::new(Mutex::new(start)),
        }
    }

    pub fn increment(&self) {
        *self.value.lock().unwrap() += 1;
    }

    pub fn value(&self) -> i32 {
        *self.value.lock().unwrap()
    }
}

pub fn fan_out_increments(
    counter: &SharedCounter,
    thread_count: usize,
    increments_each: usize,
) -> i32 {
    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let counter = counter.clone();
            thread::spawn(move || {
                for _ in 0..increments_each {
                    counter.increment();
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    counter.value()
}
```

## `label_from_thread` — همان الگویِ زیربخشِ اولِ درس

هیچ‌چیزِ تازه‌ای این‌جا نیست: `String` از اول `Send` بود، پس `move` آن را کامل به داخلِ کلوژر می‌برد و `thread::spawn` بدونِ هیچ اعتراضی قبولش می‌کند. `.join().unwrap()` نتیجه‌ای را که کلوژر برمی‌گرداند — همان `String`ِ ساخته‌شده با `format!` — بیرون می‌کشد.

## `SharedCounter` — چرا فیلدش دقیقاً `Arc<Mutex<i32>>` است

هر دو نیمه لازم بودند، هرکدام به یک دلیلِ جدا:

- **`Arc`**، چون `SharedCounter` باید `Clone`‌پذیر باشد و هر کلون باید همان شمارنده‌ی زیرین را ببیند، نه یک کپیِ مستقل. `#[derive(Clone)]` رویِ خودِ `SharedCounter` این را خودکار می‌گیرد، دقیقاً چون `Arc<T>: Clone` همین معنا را دارد — کلون‌کردن یعنی دستگیره‌ی تازه، نه تخصیصِ تازه.
- **`Mutex`**، چون `increment` باید از پشتِ `&self` بنویسد — یک ارجاعِ اشتراکی. بدونِ یک قفلِ واقعی، دو ریسمانی که هم‌زمان `.increment()` را صدا بزنند دقیقاً همان مسابقه‌ی داده‌ای را می‌سازند که کلِ این درس درباره‌اش بود؛ با `Mutex`، کامپایلر حتی اجازه نمی‌دهد نوعی که این قفل را ندارد از پشتِ `&self` نوشته شود.

تستِ `a_clone_shares_the_same_underlying_count` دقیقاً همین را ثابت می‌کند: رویِ یک کلون افزایش می‌دهد، بعد از رویِ نسخه‌ی اصلی می‌خواند — اگر فیلد `Rc<RefCell<i32>>` بود هم همین تست سبز می‌شد (چون هنوز تک‌ریسمانی است)، ولی همان لحظه که `fan_out_increments` بخواهد این `counter` را بینِ چند ریسمان واقعی پخش کند، دیگر کامپایل نمی‌شد — دقیقاً همان خطایِ `E0277`ای که تویِ «خطاهایی که خواهی دید» دیدی.

## `fan_out_increments` — چرا عددِ نهایی هرگز کم نمی‌آید

هر ریسمان یک **کلون** از `counter` می‌گیرد — نه ارجاع، خودِ کلون — چون کلوژرِ `spawn` باید `'static` باشد و نمی‌تواند به `counter`ی که تویِ تابعِ فراخواننده زندگی می‌کند قرض‌گیر بماند. کلون‌کردن این‌جا ارزان است: فقط شمارنده‌ی داخلیِ `Arc` بالا می‌رود، نه دیتایِ زیرِ `Mutex`.

حلقه‌ی دوم — `for handle in handles { handle.join().unwrap(); }` — قبل از خواندنِ `counter.value()` تضمین می‌کند هر `increments_each` تایِ هر ریسمان واقعاً تمام شده. برایِ همین تستِ `fan_out_increments_loses_nothing_across_many_threads` هر بار که اجرا شود دقیقاً همان عدد را می‌گیرد (`thread_count * increments_each`)، هرچقدر هم ترتیبِ واقعیِ قفل‌گرفتن‌ها بینِ ریسمان‌ها، از اجرایی به اجرایِ دیگر، فرق کند — قفلِ `Mutex` هر بار دقیقاً یک `.increment()` را کامل می‌کند، بدونِ اینکه هیچ‌کدام گم شود.
