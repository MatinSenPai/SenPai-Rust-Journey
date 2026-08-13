# راه‌حل

```rust
pub fn compute_async_sum(nums: Vec<i32>) -> i32 {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let sum: i32 = nums.iter().sum();
        tx.send(sum).unwrap();
    });
    rx.recv().unwrap()
}
```

یه دونه تولیدکننده، یه دونه پیام، و تمام — دستورِ `()rx.recv` تو همونجا توقف (block) می‌کنه تا زمانی که اون یه دونه مقدار بالاخره برسه (یا اگه فرستنده بدون ارسال هیچ مقداری از بین بره و drop بشه، مقدار `Err` رو برمی‌گردونه، که اینجا این اتفاق اصلاً ممکن نیست بیفته چون ریسمانِ تولیدشده همیشه قبل از تموم شدنش حتماً پیام رو ارسال می‌کنه).

```rust
pub fn collect_from_producers(producer_count: usize, values_per_producer: usize) -> Vec<i32> {
    let (tx, rx) = mpsc::channel();
    let mut handles = Vec::new();
    for i in 0..producer_count {
        let tx = tx.clone();
        handles.push(thread::spawn(move || {
            let start = (i * values_per_producer) as i32;
            for v in start..start + values_per_producer as i32 {
                tx.send(v).unwrap();
            }
        }));
    }
    drop(tx);
    let mut collected: Vec<i32> = rx.iter().collect();
    for handle in handles {
        handle.join().unwrap();
    }
    collected.sort();
    collected
}
```

دستور `(drop(tx` بلافاصله بعد از حلقه‌ی spawn، دقیقاً همون خطیه که اگه کدِ تو هَنگ کرد و گیر افتاد، به احتمال خیلی زیاد یادت رفته بنویسیش: دستور `()rx.iter` (که کاملاً معادلِ اینه که بنویسی `for v in rx`) فقط زمانی از حرکت متوقف می‌شه که *تک‌تکِ* `Sender`ها، از جمله تمام `()clone.`هایی که ازش گرفته شده، کاملاً پاک شده (drop) باشن. کپیِ (clone) مربوط به `tx` تو ریسمانِ هر کدوم از تولیدکننده‌ها، وقتی که کلوژرِ مربوط به اون ریسمان به پایان می‌رسه، به‌طور خودکار پاک و drop می‌شه — اما اون `tx`ِ *اصلی* که تو از روش کپی گرفتی، هنوز تو اسکوپِ ریسمانِ والد (parent thread) زنده‌ست تا زمانی که تو خودت به طور صریح اونو با دستور `drop` از بین ببری (یا اینکه به طور طبیعی اسکوپش تموم بشه، که تو این مثال اون موقع واسه drop شدنش خیلی دیر می‌شه — چون بعد از دستورِ `()rx.iter().collect` که خودش داره منتظرِ همون `tx` می‌مونه قرار می‌گرفت و برنامه قفل می‌شد).

در مورد سؤال ۴ چک‌پوینت: تابعِ `count_matching_in_threads` (تو درسِ قبلی) نیاز داشت که هر ریسمان بیاد و **همون یک عددِ واحد رو** هم‌زمان با بقیه و به صورت زنده آپدیت کنه — این یعنی یه وضعیت (state) مشترک و در حال تغییر (mutating)، که وظیفه و تخصصِ یه `Mutex`ه. اما تابعِ `collect_from_producers` به این نیاز داره که هر کدوم از ریسمان‌ها یه **جریان از مقادیر مستقل و جداگانه (stream of independent values)** رو تحویل بدن بدون اینکه هیچ وضعیت مشترکِ مداومی بینشون وجود داشته باشه — یعنی هیچ‌چیزی درجا و در لحظه قرار نیست تغییر (mutate) کنه، فقط مقادیر دارن از یه سری جاهای مختلف منتقل (move) می‌شن به یه جای واحد — و این دقیقاً همون چیزیه که یه کانال (channel) مستقیماً برای مدل‌سازیش طراحی شده.