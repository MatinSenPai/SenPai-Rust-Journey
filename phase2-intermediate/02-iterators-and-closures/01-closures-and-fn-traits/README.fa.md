# ۲.۲.۱ — کلوژرها، `Fn`/`FnMut`/`FnOnce` و `move`

## در یک نگاه

بعد از این درس می‌توانی:

- با خواندنِ بدنه‌ی یک کلوژر بگویی چه چیزی را و چطور (با ارجاعِ اشتراکی، ارجاعِ تغییرپذیر، یا با `move` و گرفتنِ کاملِ مالکیت) از اطرافش می‌گیرد — بدونِ اجرا کردنِ کامپایلر.
- بگویی یک کلوژرِ مشخص کدام‌یک از `Fn`، `FnMut` و `FnOnce` را پیاده‌سازی می‌کند و چرا، و توضیح بدهی چرا این سه یک سلسله‌مراتب‌اند نه سه انتخابِ جدا از هم.
- بگویی `move` دقیقاً کِی لازم است، خودت `E0596`، `E0597` و `E0382` را رفع کنی، و بینِ یک پارامترِ `Fn`، `FnMut` یا `FnOnce` برای یک امضای تابعِ واقعی یکی را انتخاب کنی.

**زمان:** حدود ۶۰ دقیقه · **پیش‌نیاز:**
[۲.۱.۴ — انتخابِ مجموعه](../../01-collections/04-choosing-a-collection/README.fa.md)، و به‌طورِ خاص
[۱.۲.۲ — معناشناسیِ حرکت](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.fa.md) و
[۱.۳.۱ — ارجاع‌های اشتراکی و تغییرپذیر](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md)

---

## چرا اهمیت دارد

تویِ کل ماژولِ ۲.۱ همین کلمه بارها زیرِ دستت آمد، بدونِ توضیح. تویِ
[۲.۱.۱](../../01-collections/01-vec-depth/README.fa.md) نوشتیم
`list.retain(|entry| !entry.watched)` و درست زیرش گفتیم: «آن `|entry| ...` یک
**کلوژر (closure)** است — یک تابعِ کوچکِ بی‌نام که همین‌جا، سرِ محل، نوشته‌ای.
کلوژرها را ۲.۲.۱ کامل یاد می‌دهد؛ همین الان فقط بخوانش». همان درس، همان‌جا،
`.sort_by(|a, b| a.rating.total_cmp(&b.rating))` و
`.dedup_by_key(|entry| entry.0.clone())` را هم نوشت — سه‌تا کلوژر، سه شکلِ
متفاوت از استفاده، و هر سه بدونِ توضیح.

این درس همان قرض را صاف می‌کند. اگر تا حالا پایتون نوشته باشی، خودِ ایده‌ی
کلوژر برایت غریبه نیست:

```python
threshold = 10
is_long = lambda title: len(title) > threshold
```

`is_long` نیازی نداشت `threshold` را به‌عنوانِ پارامتر بگیرد — دست دراز کرد و
از اسکوپِ اطرافش برش داشت. Rust هم دقیقاً همین اجازه را می‌دهد. فرق از جایی
شروع می‌شود که پایتون تمام می‌کند: در پایتون هر چیزی یک ارجاع به یک شیءِ
شمارش‌مرجع‌دار است، پس یک کلوژرِ پایتونی هیچ‌وقت مجبور نیست بینِ «فقط نگاه
کن» و «مالکیتش را بگیر» یکی را انتخاب کند — کاری برایش نمانده که انتخاب کند.
تویِ Rust، borrow checker همان قاعده‌ای را که در
[۱.۲](../../../phase1-fundamentals/02-ownership-and-memory/README.fa.md) و
[۱.۳](../../../phase1-fundamentals/03-borrowing-and-references/README.fa.md)
یاد گرفتی — یک مالکِ یکتا، یا هر تعداد قرضِ اشتراکی یا دقیقاً یک قرضِ
تغییرپذیر — رویِ هر کلوژری که می‌نویسی هم پیاده می‌کند. این درس دقیقاً همان‌جا
شروع می‌شود: کلوژر، از نگاهِ Rust، یک مسئله‌ی مالکیت است، نه یک تکه‌نحوِ
راحت.

---

## مفهوم

### یک کلوژر، یک مقدارِ واقعی با یک نوعِ واقعی و بی‌نام

نحوِ کلوژر همیشه همین شکل است: یک لیستِ پارامتر بینِ دو خطِ عمودی، بعدش یک
بدنه — یک عبارتِ تنها، یا یک بلوکِ `{ ... }` اگر بیشتر از یک خط لازم داری:

```rust
let add_one = |x: i32| x + 1;
let add_two = |x: i32| x + 2;
println!("add_one(5) = {}", add_one(5));
println!("add_two(5) = {}", add_two(5));
```

```text
add_one(5) = 6
add_two(5) = 7
```

`add_one` یک متغیرِ معمولی است، دقیقاً مثلِ هر `let` دیگری — فقط این‌بار
مقداری که نگه می‌دارد قابلِ‌فراخوانی است. و نوعش؟ نه `i32` است، نه `String`،
نه چیزی که بشود تویِ کد نوشت. کامپایلر برایِ هر کلوژری که می‌نویسی یک
ساختارِ ناشناس و مخصوصِ خودش می‌سازد؛ `std::any::type_name` این را نشانت
می‌دهد:

```rust
fn type_name_of<V>(_value: &V) -> String {
    std::any::type_name::<V>().to_string()
}

println!("{}", type_name_of(&add_one));
```

```text
01_closure_is_a_value::main::{{closure}}
```

`{{closure}}` نوشته‌ی خودِ کامپایلر است، نه چیزی که تو تایپ کرده باشی — و
اصلاً هم نحوِ معتبرِ Rust نیست. حتی اگر دو کلوژر کاملاً یک شکل باشند، مثلِ
`add_one` و `add_two` بالا، باز هم نامشان تویِ این چاپ یکی به‌نظر می‌رسد ولی
دو نوعِ کاملاً متفاوت‌اند — کافی است سعی کنی یکی را جایِ دیگری تویِ همان
متغیر بگذاری: در «خطاهایی که خواهی دید» می‌بینی دقیقاً همین اتفاق چه خطایی
می‌دهد. همین یک نکته — هر کلوژر، نوعِ یکتایِ خودش — دلیلِ اصلیِ چیزی است که
بخشِ «کلوژر به‌عنوانِ پارامتر» پایینِ همین صفحه توضیح می‌دهد.

### گیراندازیِ پیش‌فرض: با ارجاع، هر شکلی که بدنه لازم دارد

وقتی یک کلوژر از متغیرِ اطرافش استفاده می‌کند، به‌طورِ پیش‌فرض آن را با
ارجاع می‌گیرد — دقیقاً همان `&T` یا `&mut T` که در
[۱.۳.۱](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md)
دیدی، فقط این‌بار کامپایلر خودش تصمیم می‌گیرد کدام‌یک، بر اساسِ اینکه بدنه‌ی
کلوژر چه کاری با آن متغیر می‌کند:

```rust
let name = String::from("Rin");
let greet = || println!("hello, {name}");
greet();
println!("still have `name` here: {name}");
```

```text
hello, Rin
still have `name` here: Rin
```

`greet` فقط `name` را می‌خواند، پس با یک ارجاعِ اشتراکی گرفتش — و `name`
بعدش هم کاملاً مالِ خودت می‌ماند، همان‌طور که هر قرضِ اشتراکیِ دیگری هم
همین قول را می‌دهد.

حالا یک کلوژر که تغییرش می‌دهد:

```rust
let mut hits = 0;
let mut record_hit = || {
    hits += 1;
};
record_hit();
record_hit();
record_hit();
println!("hits: {hits}");
```

```text
hits: 3
```

این‌بار بدنه `hits` را تغییر می‌دهد، پس کلوژر آن را با یک ارجاعِ تغییرپذیر
گرفت. و چون خودِ *بایندِ* `record_hit` حالا یک قرضِ تغییرپذیر تویِ دستش
دارد، خودِ بایند هم باید `mut` باشد — همان قاعده‌ی هم‌نامی‌ای که در ۱.۳.۱
یاد گرفتی، این‌بار رویِ متغیری که یک کلوژر نگه می‌دارد. یادت برود؟ کامپایلر
جلویت را می‌گیرد؛ خطای کاملش را در «خطاهایی که خواهی دید» می‌بینی.

### گیراندازی با `move`: گرفتنِ کاملِ مالکیت

بعضی وقت‌ها ارجاع کافی نیست. کلوژر باید از اسکوپی که تویش نوشته شده،
بیشتر عمر کند — از یک تابع برگردد، به یک نخِ اجرایِ دیگر سپرده شود، یا
جایی نگه داشته شود که دیرتر از دوامِ طبیعیِ آن قرض صدا زده می‌شود. کلمه‌ی
کلیدیِ `move` دقیقاً برایِ همین موردها هست: کلوژر را مجبور می‌کند به‌جایِ
قرض‌گرفتن، مالکیتِ هر چیزی را که گرفته، کامل بگیرد.

```rust
let printer;
{
    let message = String::from("hi from the inner scope");
    printer = move || println!("{message}");
}
printer();
```

```text
hi from the inner scope
```

`message` متغیرِ محلیِ همان بلوکِ داخلی است — با پایانِ آن بلوک از بین
می‌رود. اگر `printer` فقط یک ارجاع به `message` قرض گرفته بود، وقتی
`printer()` بیرونِ بلوک صدا زده می‌شد، آن ارجاع دیگر آویزان (dangling) بود.
`move` این مشکل را از ریشه حل می‌کند: `message` دیگر مالِ آن بلوکِ داخلی
نیست، مالِ خودِ کلوژر است، و کلوژر هر جا برود، آن را با خودش می‌برد. بدونِ
`move`، همین کد `E0597` می‌گیرد — نسخه‌ی خرابش تویِ «خطاهایی که خواهی دید»
است.

```senpai-visual
{"kind":"ownership","labels":["کلوژر message را قرض می‌گیرد","اسکوپِ درونی تمام می‌شود","message از بین می‌رود","قرض حالا آویزان است","move مالکیتش را می‌گیرد"]}
```

نکته‌ای که نباید قاطی شود: `move` فقط تصمیم می‌گیرد *چطور* یک متغیر گرفته
شود، نه اینکه کلوژر چند بار قابلِ‌صدا‌زدن است. آن دوتا سؤالِ کاملاً جدا از
هم‌اند:

```rust
let name = String::from("Rin");
let greet = move || println!("hello, {name}");

greet();
greet();
greet();
```

```text
hello, Rin
hello, Rin
hello, Rin
```

`greet` مالکیتِ `name` را کامل گرفت — ولی بدنه‌اش فقط آن را می‌خواند، هیچ‌جا
از دستش درنمی‌آورد. برایِ همین، بازهم هر چند بار که بخواهی صدا زدنی است. با
`move` گرفتن یعنی «کامل بگیرش»، نه «فقط یک‌بار اجازه‌ی استفاده داری» —
اینکه چند بار قابلِ‌صدا‌زدن است را چیزِ دیگری تعیین می‌کند، همان چیزی که
بخشِ بعدی اسمش را می‌گذارد.

### `Fn`، `FnMut`، `FnOnce`: یک سلسله‌مراتب، نه سه انتخابِ جدا از هم

هر کلوژری، بسته به اینکه بدنه‌اش با چیزی که گرفته چه‌کار می‌کند، یکی (یا
بیشتر) از این سه trait را پیاده‌سازی می‌کند:

```rust
let greeting = String::from("hi");
let read_only = || println!("read-only: {greeting}");
read_only();
read_only();

let mut hits = 0;
let mut count_calls = || {
    hits += 1;
    println!("mutate: called {hits} time(s)");
};
count_calls();
count_calls();

let payload = String::from("payload");
let consume = move || payload;
let taken = consume();
println!("consume: took ownership of {taken}");
```

```text
read-only: hi
read-only: hi
mutate: called 1 time(s)
mutate: called 2 time(s)
consume: took ownership of payload
```

- **`Fn`** — بدنه فقط چیزی که گرفته را می‌خواند. هر چند بار بخواهی
  صدازدنی است.
- **`FnMut`** — بدنه چیزی که گرفته را تغییر می‌دهد. بازهم هر چند بار
  صدازدنی است، ولی هر صدا ممکن است چیزِ گرفته‌شده را عوض کند.
- **`FnOnce`** — بدنه چیزی که گرفته را از داخلِ خودِ کلوژر بیرون می‌برد
  (move می‌کند). فقط **یک‌بار** قابلِ‌صدا‌زدن است، چون بعدِ اولین صدا دیگر
  چیزی برایِ گرفتن نمانده.

این سه یک سلسله‌مراتب می‌سازند: هر کلوژرِ `Fn` یک `FnMut`ِ معتبر هم هست
(«فقط خواندن» حالتِ خاصی از «نیازی به دسترسیِ انحصاری نداشتن» است)، و هر
کلوژرِ `FnMut` یک `FnOnce`ِ معتبر هم هست («تغییر دادنِ مکرر» حالتِ خاصی از
«دستِ‌کم یک‌بار قابلِ‌صدا‌زدن بودن» است). یعنی جهت فقط یک‌طرفه است: هر `Fn`
یک `FnOnce` هم هست، ولی هر `FnOnce` یک `Fn` نیست — دقیقاً همان چیزی که
مثالِ `consume` بالا نشان داد.

```senpai-visual
{"kind":"concept","labels":["بدونِ گیراندازی: `fn`","فقط می‌خواند: `Fn`","تغییر می‌دهد: `FnMut`","مصرف می‌کند: `FnOnce`"]}
```

و نکته‌ی مهم: تو هیچ‌وقت خودت نمی‌نویسی این کلوژر `Fn` است یا `FnMut`.
کامپایلر بدنه را می‌خواند و محدودترین traitای که واقعاً صدق می‌کند را
خودش تشخیص می‌دهد — همان‌طور که در مثالِ بالا `read_only`، `count_calls` و
`consume` سه سرنوشتِ متفاوت گرفتند، بدونِ اینکه جایی به کامپایلر گفته
باشیم کدام‌یک کدام است.

### کلوژر به‌عنوانِ پارامتر و مقدارِ بازگشتی

از بخشِ اول یادت هست: هر کلوژر یک نوعِ یکتا و بی‌نام دارد. این یعنی یک
تابع که می‌خواهد «یک کلوژر» بگیرد، هیچ‌وقت نمی‌تواند آن نوع را تویِ امضایش
بنویسد — باید جنریک باشد، رویِ traitای که واقعاً لازم دارد:

```rust
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

fn apply_twice_v2(f: impl Fn(i32) -> i32, x: i32) -> i32 {
    f(f(x))
}

println!("{}", apply_twice(|x| x + 10, 1));
println!("{}", apply_twice_v2(|x| x + 10, 1));
```

```text
21
21
```

`apply_twice` و `apply_twice_v2` یک امضایِ کاملاً یکسان دارند، فقط با دو
نوشتار: یک پارامترِ جنریک با کراندِ `Fn(i32) -> i32`، یا `impl Trait` تویِ
جایگاهِ پارامتر — کوته‌نویسیِ همان چیز. جنریک‌ها و `impl Trait` را ماژولِ
۲.۳ کامل یاد می‌دهد؛ همین‌قدر امروز کافی است: چون آن `F` هر بار با نوعِ
کلوژرِ واقعی که پاس داده‌ای جایگزین می‌شود، کامپایلر برایِ هر فراخوانیِ
متفاوت، یک نسخه‌ی جداگانه از تابع می‌سازد — دیگر لازم نیست کسی نامِ آن نوعِ
`{{closure}}` را بداند.

مسیرِ برگشت هم همین مشکل را دارد، فقط برعکس: تابعی که می‌خواهد «یک کلوژر»
برگرداند، باز هم نمی‌تواند نوعش را بنویسد. اینجا جنریک کمکی نمی‌کند (نوعِ
برگشتی را صداکننده انتخاب نمی‌کند، خودِ تابع باید بگوید)، پس نوشتار
`impl Trait` تویِ جایگاهِ خروجی می‌آید:

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

let add_five = make_adder(5);
println!("{}", add_five(1));
println!("{}", add_five(100));
```

```text
6
105
```

`impl Fn(i32) -> i32` یعنی «یک نوعِ ملموس که این را پیاده‌سازی می‌کند، به
تو نمی‌گویم کدام». `move` هم اینجا لازم است: بدونِ آن، کلوژر می‌خواست `n`
را قرض بگیرد — پارامتری که با برگشتنِ `make_adder` از بین می‌رفت. (یک
نکته برایِ بعد: `impl Trait` تویِ خروجی فقط یک نوعِ ملموس را قول می‌دهد؛
وقتی نوعِ واقعی بسته به شرط فرق می‌کند، به `Box<dyn Fn(i32) -> i32>` نیاز
داری — چیزی که تویِ درسِ اشیایِ trait کاملش را می‌بینی.)

`FnMut` و `FnOnce` هم دقیقاً به همین دلیل تویِ امضاها ظاهر می‌شوند — نه
سلیقه، بلکه اینکه تابع چند بار قرار است کلوژر را صدا بزند:

```rust
fn call_n_times<F: FnMut()>(mut f: F, n: u32) {
    for _ in 0..n {
        f();
    }
}

fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}

let mut log = Vec::new();
call_n_times(|| log.push("tick"), 3);
println!("{log:?}");

let owned = String::from("payload");
println!("{}", run_once(move || owned));
```

```text
["tick", "tick", "tick"]
payload
```

`call_n_times` ممکن است `f` را بارها صدا بزند — و کلوژرِ واقعی که یک
کاربر پاس می‌دهد معمولاً می‌خواهد چیزی را هر بار تغییر دهد (یک شمارنده،
یک لاگ) — پس `FnMut` می‌خواهد، نه `Fn`ِ سخت‌گیرتر. `run_once` دقیقاً
برعکس: فقط یک‌بار صدا می‌زند، پس ضعیف‌ترین trait برایش کافی است —
`FnOnce` — و همین ضعیف‌بودن است که به کلوژری که چیزِ گرفته‌شده‌اش را
مصرف می‌کند هم اجازه‌ی ورود می‌دهد.

جهتِ سلسله‌مراتب یک‌طرفه است، و کامپایلر این را جدی می‌گیرد: کلوژری که
فقط `FnOnce` است، جایی که `Fn` خواسته شده رد می‌شود — دقیقاً برعکسش (یک
کلوژرِ `Fn` دادن جایی که `FnOnce` کافی است) همیشه قبول است، چون `Fn` هم
یک `FnOnce`ِ معتبر است. `run_once` بالا را با یک کلوژرِ ساده و `Fn` هم
امتحان کن، مشکلی نیست؛ ولی `apply_twice` را با یک کلوژرِ فقط-`FnOnce`
صدا بزن، `E0525` می‌گیری — خطای کاملش تویِ بخشِ بعدی است.

### اشاره‌گرِ تابع: حالتِ بی‌گیراندازی و پیش‌پاافتاده

یک تابعِ معمولی، وقتی به‌عنوانِ مقدار به‌کار می‌رود، هیچ متغیری از هیچ
اسکوپی نمی‌گیرد — برایِ همین به‌طورِ رایگان هر سه‌یِ `Fn`، `FnMut` و
`FnOnce` را پیاده‌سازی می‌کند، بدونِ هیچ شرطی:

```rust
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

fn add_ten(x: i32) -> i32 {
    x + 10
}

println!("{}", apply_twice(add_ten, 1));
let as_pointer: fn(i32) -> i32 = add_ten;
println!("{}", apply_twice(as_pointer, 1));
```

```text
21
21
```

نوعِ `add_ten` وقتی به‌عنوانِ مقدار به‌کار می‌رود، `fn(i32) -> i32` است —
یک **اشاره‌گرِ تابع (function pointer)**: حروفِ کوچکِ `fn`، یک نوعِ ملموس،
نه traitِ `Fn`. چون هیچی برایِ گرفتن ندارد، همان‌جایی که یک کلوژرِ `Fn`
انتظار می‌رود، بدونِ هیچ تبدیلِ دستی جا می‌شود — این حالتِ ساده‌ترینِ
همه‌ی چیزهایی است که تویِ این درس دیدی.

---

## دست‌به‌کد

```sh
cargo run -p p2-02-01-closures-and-fn-traits --example 01-closure-is-a-value
cargo run -p p2-02-01-closures-and-fn-traits --example 02-capture-by-reference
cargo run -p p2-02-01-closures-and-fn-traits --example 03-move-lets-a-closure-outlive-its-scope
cargo run -p p2-02-01-closures-and-fn-traits --example 04-move-does-not-mean-fnonce
cargo run -p p2-02-01-closures-and-fn-traits --example 05-fn-fnmut-fnonce-hierarchy
cargo run -p p2-02-01-closures-and-fn-traits --example 06-closures-as-parameters-and-return-values
cargo run -p p2-02-01-closures-and-fn-traits --example 07-fnmut-and-fnonce-parameters
cargo run -p p2-02-01-closures-and-fn-traits --example 08-function-pointers
```

بعد پنج‌تای خراب:

```sh
cargo run -p p2-02-01-closures-and-fn-traits --example 09-closures-have-distinct-types --features broken
cargo run -p p2-02-01-closures-and-fn-traits --example 10-forgot-mut-on-capturing-closure --features broken
cargo run -p p2-02-01-closures-and-fn-traits --example 11-missing-move-dangling-borrow --features broken
cargo run -p p2-02-01-closures-and-fn-traits --example 12-fnonce-closure-called-twice --features broken
cargo run -p p2-02-01-closures-and-fn-traits --example 13-fn-bound-rejects-fnonce-only --features broken
```

بعد این‌ها را امتحان کن:

۱. تویِ `02-capture-by-reference`، به‌جایِ سه بار، `record_hit` را ده بار
صدا بزن. بعد `mut` را از جلویِ `record_hit` بردار و ببین کامپایلر دقیقاً
چه پیشنهادی می‌دهد.
۲. تویِ `05-fn-fnmut-fnonce-hierarchy`، یک خطِ `consume();` دیگر بعدِ
`let taken = consume();` اضافه کن. کدام خطا می‌گیری، و کدام خطِ داخلِ
پیامِ خطا می‌گوید *چرا*؟
۳. تویِ `06-closures-as-parameters-and-return-values`، به‌جایِ
`make_adder(5)`، `make_adder(-3)` بساز و رویِ چند عددِ مختلف صدایش بزن.

---

## خطاهایی که خواهی دید

### `E0308` — دو کلوژر، حتی با امضایِ یکسان، دو نوعِ متفاوت‌اند

```text
error[E0308]: mismatched types
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\09-closures-have-distinct-types.rs:15:13
   |
12 |     let add_one = |x: i32| x + 1;
   |                   -------- the expected closure
13 |     let add_two = |x: i32| x + 2;
   |                   -------- the found closure
14 |     let mut which = add_one;
15 |     which = add_two;
   |             ^^^^^^^ expected closure, found a different closure
   |
   = note: expected closure `{closure@phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\09-closures-have-distinct-types.rs:12:19: 12:27}`
              found closure `{closure@phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\09-closures-have-distinct-types.rs:13:19: 13:27}`
   = note: no two closures, even if identical, have the same type
   = help: consider boxing your closure and/or using it as a trait object

For more information about this error, try `rustc --explain E0308`.
```

**کامپایلر به چه اعتراض دارد:** `add_one` و `add_two` هر دو امضایِ
`Fn(i32) -> i32` را دارند، ولی این دو کلوژرِ *متفاوت* یک نوعِ مشترک ندارند.
`which` وقتی با `add_one` مقداردهیِ اولیه شد، نوعش دقیقاً همان نوعِ
`add_one` شد؛ گذاشتنِ `add_two` داخلش، مثلِ این است که بخواهی یک `i32` را
جایِ یک `bool` بگذاری.

**راه‌حل:** دو کلوژرِ متفاوت را تویِ دو متغیرِ جدا نگه دار:

```rust
let add_one = |x: i32| x + 1;
let add_two = |x: i32| x + 2;
println!("{}", add_one(5));
println!("{}", add_two(5));
```

**چرا این راه‌حل است:** هیچ‌جا لازم نبود این دو یک متغیرِ مشترک را پر
کنند. اگر واقعاً به یک متغیر نیاز داری که بسته به شرط، هرکدام از این دو
کلوژر را نگه دارد، آن‌وقت باید نوعِ ملموس را پاک کنی — دقیقاً همان
`Box<dyn Fn(i32) -> i32>`ی که در بخشِ مفهوم به‌اش اشاره شد. و همین تفاوتِ
نوع است که بخشِ «کلوژر به‌عنوانِ پارامتر» را توجیه می‌کند: تابعی که
می‌خواهد هرکدام از این دو کلوژر را قبول کند، باید جنریک باشد.

### `E0596` — کلوژری که تغییر می‌دهد، باید در بایندی `mut` بنشیند

```text
error[E0596]: cannot borrow `increment` as mutable, as it is not declared as mutable
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\10-forgot-mut-on-capturing-closure.rs:15:5
   |
13 |         count += 1;
   |         ----- calling `increment` requires mutable binding due to mutable borrow of `count`
14 |     };
15 |     increment();
   |     ^^^^^^^^^ cannot borrow as mutable
   |
help: consider changing this to be mutable
   |
12 |     let mut increment = || {
   |         +++

For more information about this error, try `rustc --explain E0596`.
```

**کامپایلر به چه اعتراض دارد:** بدنه‌ی `increment` می‌خواهد `count` را
تغییر دهد، پس خودِ کلوژر یک ارجاعِ تغییرپذیر تویِ دستش نگه می‌دارد. صدا
زدنِ `increment()` یعنی از آن ارجاعِ تغییرپذیر استفاده کن — و این استفاده،
مثلِ هر استفاده‌ی دیگری از یک قرضِ تغییرپذیر، به یک بایندِ `mut` نیاز دارد.

**راه‌حل:** `mut` را جلویِ بایند بگذار:

```rust
let mut count = 0;
let mut increment = || {
    count += 1;
};
increment();
println!("{count}");
```

```text
1
```

**چرا این راه‌حل است:** این دقیقاً همان قاعده‌ای است که در ۱.۳.۱ دیدی، فقط
رویِ متغیری که یک کلوژر نگه می‌دارد به‌جایِ یک `&mut` ساده: چیزی که یک
دسترسیِ انحصاری تویِ دست دارد، خودش هم باید `mut` باشد.

### `E0597` — بدونِ `move`، کلوژر از اسکوپِ نوشته‌شدنش بیشتر عمر نمی‌کند

```text
error[E0597]: `message` does not live long enough
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\11-missing-move-dangling-borrow.rs:14:33
   |
13 |         let message = String::from("hi from the inner scope");
   |             ------- binding `message` declared here
14 |         printer = || println!("{message}");
   |                   --            ^^^^^^^ borrowed value does not live long enough
   |                   |
   |                   value captured here
15 |     }
   |     - `message` dropped here while still borrowed
16 |     printer();
   |     ------- borrow later used here

For more information about this error, try `rustc --explain E0597`.
```

**کامپایلر به چه اعتراض دارد:** بدونِ `move`، `printer` فقط `message` را
قرض گرفت. `message` با پایانِ آن بلوکِ داخلی از بین رفت — ولی `printer`،
و آن قرضی که تویِ دستش دارد، بیرونِ همان بلوک هنوز زنده است. این دقیقاً
همان چیزی است که در [۱.۲.۲](../../../phase1-fundamentals/02-ownership-and-memory/02-move-semantics/README.fa.md)
یاد گرفتی: یک قرض هیچ‌وقت نباید از چیزی که به آن اشاره می‌کند، بیشتر عمر
کند.

**راه‌حل:** `move` اضافه کن:

```rust
let printer;
{
    let message = String::from("hi from the inner scope");
    printer = move || println!("{message}");
}
printer();
```

```text
hi from the inner scope
```

**چرا این راه‌حل است:** با `move`، کلوژر دیگر قرض نمی‌گیرد — مالکیتِ
`message` را کامل می‌گیرد. حالا دیگر مهم نیست آن بلوکِ داخلی کِی تمام
می‌شود؛ `message` دیگر مالِ آن بلوک نیست، مالِ خودِ `printer` است.

### `E0382` — یک کلوژرِ `FnOnce`، با صدا زدنش خودش هم مصرف می‌شود

```text
error[E0382]: use of moved value: `consume`
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\12-fnonce-closure-called-twice.rs:17:5
   |
16 |     consume();
   |     --------- `consume` moved due to this call
17 |     consume();
   |     ^^^^^^^ value used here after move
   |
note: closure cannot be invoked more than once because it moves the variable `name` out of its environment
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\12-fnonce-closure-called-twice.rs:13:21
   |
13 |         let owned = name;
   |                     ^^^^
note: this value implements `FnOnce`, which causes it to be moved when called
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\12-fnonce-closure-called-twice.rs:16:5
   |
16 |     consume();
   |     ^^^^^^^

For more information about this error, try `rustc --explain E0382`.
```

**کامپایلر به چه اعتراض دارد:** بدنه‌ی `consume` با `let owned = name;`
مقدارِ `name` را از داخلِ گیراندازیِ خودش بیرون می‌برد — یعنی `consume`
فقط `FnOnce` است. صدا زدنِ یک کلوژرِ `FnOnce` خودِ کلوژر را هم move
می‌کند؛ صدایِ اول `consume` را مصرف کرد، صدایِ دوم دیگر چیزی برایِ صدا
زدن پیدا نکرد.

**راه‌حل:** اگر واقعاً به چند بار صدا زدن نیاز داری، چیزی که گرفته را از
داخلِ کلوژر بیرون نبر — کلونش کن:

```rust
let name = String::from("Rin");
let consume = move || {
    let owned = name.clone();
    println!("consumed: {owned}");
};
consume();
consume();
```

```text
consumed: Rin
consumed: Rin
```

**چرا این راه‌حل است:** `.clone()` فقط از `name` یک کپیِ تازه می‌سازد و
خودِ `name` را دستِ‌نخورده تویِ گیراندازیِ کلوژر نگه می‌دارد — بدنه دیگر
چیزی را از داخلِ خودش بیرون نمی‌برد، پس کلوژر دیگر `FnOnce` نیست، `Fn`
است، و هر چند بار که بخواهی صدازدنی می‌ماند.

### `E0525` — یک پارامترِ `Fn`، یک کلوژرِ فقط-`FnOnce` را قبول نمی‌کند

```text
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\13-fn-bound-rejects-fnonce-only.rs:18:27
   |
18 |     let consume_and_add = move |x: i32| {
   |                           ^^^^^^^^^^^^^ this closure implements `FnOnce`, not `Fn`
19 |         drop(bonus);
   |              ----- closure is `FnOnce` because it moves the variable `bonus` out of its environment
...
22 |     println!("{}", apply_twice(consume_and_add, 5));
   |                    ----------- --------------- the requirement to implement `Fn` derives from here
   |                    |
   |                    required by a bound introduced by this call
   |
note: required by a bound in `apply_twice`
  --> phase2-intermediate\02-iterators-and-closures\01-closures-and-fn-traits\examples\13-fn-bound-rejects-fnonce-only.rs:12:19
   |
12 | fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
   |                   ^^^^^^^^^^^^^^ required by this bound in `apply_twice`

For more information about this error, try `rustc --explain E0525`.
```

**کامپایلر به چه اعتراض دارد:** `apply_twice` قرار است `f` را دو بار صدا
بزند، پس کراندش `Fn` است. `consume_and_add` با `drop(bonus)` مقدارِ
`bonus` را از داخلِ گیراندازیِ خودش بیرون می‌برد — یعنی فقط `FnOnce` است.
سلسله‌مراتب یک‌طرفه است: هر `Fn` یک `FnOnce` هم هست، ولی این کلوژر برعکسش
است، و کامپایلر این را قبول نمی‌کند.

**راه‌حل:** به‌جایِ بیرون‌بردنِ `bonus`، فقط بخوانش:

```rust
fn apply_twice<F: Fn(i32) -> i32>(f: F, x: i32) -> i32 {
    f(f(x))
}

let bonus = String::from("bonus");
let add_with_log = move |x: i32| {
    println!("adding for {bonus}");
    x + 1
};
println!("{}", apply_twice(add_with_log, 5));
```

```text
adding for bonus
adding for bonus
7
```

**چرا این راه‌حل است:** `println!("{bonus}")` فقط `bonus` را می‌خواند —
چیزی از داخلِ گیراندازیِ کلوژر بیرون نمی‌رود. حالا `add_with_log` هم `Fn`
است، و `apply_twice` می‌تواند بدونِ مشکل دو بار صدایش بزند.

---

## تمرین

### گرم‌کردن

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let text = String::from("hi");
let show = || println!("{text}");
show();
println!("{text}");
```

</details>

<details>
<summary>پاسخ</summary>

بله. `show` فقط `text` را می‌خواند، پس با ارجاعِ اشتراکی گرفتش — `text`
بعدش هم کاملاً قابلِ‌استفاده می‌ماند.

</details>

<details>
<summary>این چه چیزی چاپ می‌کند؟</summary>

```rust
let mut total = 0;
let mut add = |n: i32| total += n;
add(3);
add(4);
println!("{total}");
```

</details>

<details>
<summary>پاسخ</summary>

```text
7
```

هر صدا `total` را تغییر می‌دهد؛ `add` یک `FnMut` است، و صداهایِ پیاپی
اثرشان جمع می‌شود: ۰ + ۳ + ۴ = ۷.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
let data = vec![1, 2, 3];
let consume = move || data;
let first = consume();
let second = consume();
```

</details>

<details>
<summary>پاسخ</summary>

نه. بدنه‌ی `consume` مقدارِ `data` را از داخلِ گیراندازیِ خودش بیرون
می‌برد و برمی‌گرداند — یعنی `consume` فقط `FnOnce` است. صدایِ اول خودِ
`consume` را move می‌کند؛ صدایِ دوم یک `E0382` می‌دهد، دقیقاً مثلِ مثالِ
`consume`/`name` در «خطاهایی که خواهی دید».

</details>

<details>
<summary>این کلوژر کدام‌یک از <code>Fn</code>، <code>FnMut</code> و <code>FnOnce</code> را پیاده‌سازی می‌کند؟</summary>

```rust
let word = String::from("hi");
let show_len = move || word.len();
```

</details>

<details>
<summary>پاسخ</summary>

هر سه‌تا — `Fn` (و در نتیجه‌اش `FnMut` و `FnOnce` هم). `word` با `move`
کامل گرفته شد، ولی بدنه فقط `.len()` رویش صدا می‌زند — یعنی فقط
می‌خواندش، هیچ‌جا تغییرش نمی‌دهد و از داخلِ کلوژر هم بیرونش نمی‌برد.

</details>

<details>
<summary>این کامپایل می‌شود؟</summary>

```rust
fn needs_fn<F: Fn()>(f: F) {
    f();
    f();
}

let msg = String::from("bye");
let closure = move || {
    drop(msg);
};
needs_fn(closure);
```

</details>

<details>
<summary>پاسخ</summary>

نه، `E0525`. کلوژرِ داده‌شده `msg` را با `drop(msg)` از داخلِ خودش بیرون
می‌برد، پس فقط `FnOnce` است — ولی `needs_fn` کراندش `Fn` است، چون قرار
است `f` را دو بار صدا بزند.

</details>

### تعمیر

هر پنج مثالِ خراب را درست کن:

۱. `examples/09-closures-have-distinct-types.rs` را طوری درست کن که
کامپایل شود — بدونِ اینکه یک متغیرِ مشترک را بینِ دو کلوژرِ متفاوت جابه‌جا
کنی.
۲. `examples/10-forgot-mut-on-capturing-closure.rs` را با اضافه‌کردنِ
یک `mut` درست کن.
۳. `examples/11-missing-move-dangling-borrow.rs` را با اضافه‌کردنِ
`move` درست کن.
۴. `examples/12-fnonce-closure-called-twice.rs` را طوری درست کن که
`consume` دو بار قابلِ‌صدا‌زدن باشد، بدونِ اینکه امضایِ چیزی که چاپ
می‌کند عوض شود.
۵. `examples/13-fn-bound-rejects-fnonce-only.rs` را طوری درست کن که
کلوژرِ داده‌شده به `apply_twice`، `Fn` باشد نه فقط `FnOnce`.

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p2-02-01-closures-and-fn-traits
```

هر تابع دقیقاً یکی از شکل‌هایی است که در «مفهوم» دیدی — یک پارامترِ `Fn`،
یک تابعی که `impl Fn` برمی‌گرداند، یک پارامترِ `FnMut`، و یک پارامترِ
`FnOnce`. کامنتِ مستنداتِ هر تابع دقیقاً می‌گوید چه چیزی برمی‌گرداند و در
چه حالت‌هایی؛ چیزی را حدس نزن.

### بساز

یک تابع بنویس که یک کلوژرِ `FnMut() -> bool` بگیرد و اسمش را
`retry_until_success` بگذار: تا سقفِ `max_tries` بار صدایش بزند و همین که
یک بار `true` برگرداند، متوقف شود؛ در پایان بگوید که آیا موفق شده یا نه
(`bool`).

بعد یک نسخه‌ی دوم بنویس — امضایش را خودت انتخاب کن — که علاوه بر موفقیت،
بگوید *چند بار* طول کشید تا موفق شود (`Option<u32>`، یا هر شکلی که خودت
تصمیم می‌گیری). در کامنتِ مستنداتش بنویس چرا آن شکلِ برگشتی را انتخاب
کردی.

### چالش (اختیاری)

با `std::mem::size_of_val(&closure)` اندازه‌یِ سه کلوژرِ زیر را با هم
مقایسه کن: یکی که هیچی نمی‌گیرد، یکی که یک `i32` را با `move` می‌گیرد، و
یکی که دو `String` را با `move` می‌گیرد. چه الگویی می‌بینی؟ (سرنخ: به یک
کلوژر مثلِ یک ساختارِ تولیدشده‌توسطِ‌کامپایلر فکر کن — فیلدهایش چه
می‌بودند؟)

(این بخش جلوتر را نگاه می‌کند.) در مستنداتِ استاندارد `Box<dyn Fn(i32) ->
i32>` را جست‌وجو کن و با `impl Fn(i32) -> i32` مقایسه‌اش کن. چرا
`make_adder` می‌توانست `impl Fn` برگرداند ولی تابعی که بسته به یک `if`،
گاهی یک کلوژر و گاهی کلوژرِ دیگری برمی‌گرداند، نمی‌تواند؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| کلوژر (closure) | تابعِ ناشناسِ درون‌خطی، `\|x\| ...` | فراخوانی‌هایی که یک تابعِ کوچک و یک‌بارمصرف لازم دارند |
| گیراندازی (capture) | نحوه‌ی دسترسیِ کلوژر به متغیرِ اطرافش | تصمیمِ خودکارِ کامپایلر: ارجاعِ اشتراکی، تغییرپذیر، یا `move` |
| `move` | مجبورکردنِ کلوژر به گرفتنِ کاملِ مالکیت | وقتی کلوژر باید از اسکوپِ نوشته‌شدنش بیشتر عمر کند |
| `Fn` | فقط می‌خواند؛ هر چند بار صدازدنی | پارامترهایی که فقط باید بخوانند |
| `FnMut` | تغییر می‌دهد؛ هر چند بار صدازدنی | پارامترهایی که بارها صدا زده می‌شوند و باید چیزی را عوض کنند |
| `FnOnce` | چیزِ گرفته‌شده را بیرون می‌برد؛ فقط یک‌بار | پارامترهایی که فقط یک‌بار صدا زده می‌شوند و مالکیت می‌خواهند |
| اشاره‌گرِ تابع (`fn`) | نوعِ یک تابعِ ساده و بدونِ گرفتن | جایی که یک کلوژرِ `Fn`/`FnMut`/`FnOnce` انتظار می‌رود |

### الان می‌دانی

- کلوژر یک مقدارِ واقعی است با یک نوعِ واقعی، یکتا و بی‌نام — نه یک
  میان‌بُرِ نحوی برایِ چیزِ دیگری.
- گرفتنِ پیش‌فرض با ارجاع است — اشتراکی اگر بدنه فقط بخواند، تغییرپذیر
  اگر بدنه تغییر بدهد — و کامپایلر خودش این را از رویِ بدنه تشخیص می‌دهد.
- `move` فقط تصمیم می‌گیرد *چطور* گرفته شود، نه چند بار قابلِ‌صدا‌زدن
  است؛ لازمش داری هر جا کلوژر باید از اسکوپِ نوشته‌شدنش بیشتر عمر کند.
- `Fn`، `FnMut` و `FnOnce` یک سلسله‌مراتب‌اند: هر `Fn` یک `FnMut` هم
  هست، هر `FnMut` یک `FnOnce` هم هست — و این جهت هیچ‌وقت برعکس نمی‌شود.
- تابعی که «یک کلوژر» می‌گیرد باید جنریک باشد یا `impl Trait` تویِ
  پارامتر بنویسد؛ تابعی که «یک کلوژر» برمی‌گرداند باید `impl Trait` تویِ
  خروجی بنویسد — چون نوعِ واقعیِ کلوژر هیچ‌وقت قابلِ‌نوشتن نیست.
- یک تابعِ معمولی، وقتی به‌عنوانِ مقدار به‌کار می‌رود، هیچی نمی‌گیرد و
  رایگان هر سه‌یِ `Fn`/`FnMut`/`FnOnce` را پیاده‌سازی می‌کند.

### بعداً کامل‌تر می‌بینی

- **آداپتورهایِ Iterator، که مدام رویِ `Fn`/`FnMut` تکیه می‌کنند بدونِ
  توضیحِ دوباره** — [۰۲.۲ — آداپتورهای Iterator](../02-iterator-adapters/README.fa.md)
- **جنریک‌ها، به‌طورِ کامل** — [۰۳.۱ — توابع و ساختارهای Generic (عمومی)](../../03-traits-and-generics/02-generic-functions-and-structs/README.fa.md)
- **`impl Trait` و `dyn Trait`، و کِی هرکدام** — [۰۳.۳ — اشیای Trait در برابر تخصیص استاتیک (static dispatch)](../../03-traits-and-generics/07-static-vs-dynamic-dispatch/README.fa.md)
- **سپردنِ یک کلوژر به یک نخِ اجرایِ دیگر، جایی که `move` تقریباً همیشه
  اجباری است** — [۰۷.۱ — ریسمان‌ها (Threads)، قفل‌ها (`Mutex`) و `Arc`](../../08-concurrency/01-threads-mutex-arc/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا `std::any::type_name` برایِ یک کلوژر چیزی مثلِ `{{closure}}`
  چاپ می‌کند، و این یعنی چه برایِ نوشتنِ نوعِ آن با دست؟
- بینِ گرفتنِ پیش‌فرض (با ارجاع) و گرفتن با `move` چه فرقی هست، و کِی
  کدام‌یک را انتخاب می‌کنی؟
- چرا `move |x| x + n` هنوز `Fn` است، با اینکه `n` را با `move` گرفته؟
- سلسله‌مراتبِ `Fn`/`FnMut`/`FnOnce` را با یک مثالِ خودت (نه از این درس)
  توضیح بده — کدام‌یک از کدام‌یک نتیجه می‌شود؟
- چرا یک تابع که «یک کلوژر» می‌گیرد باید جنریک باشد؟ چرا تابعی که «یک
  کلوژر» برمی‌گرداند به `impl Trait` نیاز دارد؟
- چرا یک اشاره‌گرِ تابع، هر سه‌یِ `Fn`/`FnMut`/`FnOnce` را رایگان
  پیاده‌سازی می‌کند؟

---

## بیشتر

- [کتابِ Rust — کلوژرها](https://doc.rust-lang.org/book/ch13-01-closures.html) — همین زمین، رسمی و کامل.
- [مرجعِ Rust — انواعِ کلوژر](https://doc.rust-lang.org/reference/types/closure.html) — جزئیاتِ دقیقِ اینکه کامپایلر چطور یک کلوژر را به یک ساختار تبدیل می‌کند، و چطور تصمیمِ گرفتن را می‌گیرد.
- [`std::ops::Fn`](https://doc.rust-lang.org/std/ops/trait.Fn.html)، [`FnMut`](https://doc.rust-lang.org/std/ops/trait.FnMut.html)، [`FnOnce`](https://doc.rust-lang.org/std/ops/trait.FnOnce.html) — تعریفِ رسمیِ هر سه trait تویِ کتابخانه‌ی استاندارد.
- [Rust by Example — کلوژرها](https://doc.rust-lang.org/rust-by-example/fn/closures.html) — مثال‌های بیشتر، از جمله همین سه‌گانه‌ی `Fn`/`FnMut`/`FnOnce` با کدهایِ کوتاه‌تر.
