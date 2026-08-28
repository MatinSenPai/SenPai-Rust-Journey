# ۲.۶.۴ — `Weak` و چرخه‌های ارجاع

## در یک نگاه

بعد از این درس می‌توانی:

- دو ساختارِ Rc-دار بسازی که هرکدام دیگری را با یک ارجاعِ قوی نگه داشته، و دقیقاً توضیح بدهی چرا شمارنده‌ی هیچ‌کدام هرگز به صفر نمی‌رسد.
- بگویی safe Rust چه چیزی را واقعاً تضمین می‌کند و چه چیزی را نه — و این تفاوت را با یک نشتِ حافظه‌ی واقعی که خودت با دست ساخته‌ای نشان بدهی.
- از `Weak<T>` و `.upgrade()` استفاده کنی، و `Option<Rc<T>>`ی که برمی‌گرداند را به‌جایِ یک مانع، جوابِ صادقانه به یک سؤالِ واقعی ببینی.
- یک درختِ پدر/فرزند بسازی که پدر فرزندانش را قوی نگه می‌دارد و هر فرزند فقط یک ارجاعِ ضعیف به پدرش دارد، و بگویی چرا این جهت — نه برعکسش — چرخه را می‌شکند.

**زمان:** حدود ۵۰ دقیقه · **پیش‌نیاز:** [۲.۶.۳ — `Rc` و `Arc`](../03-rc-and-arc/README.fa.md)

---

## چرا اهمیت دارد

درسِ قبل یک قولِ قشنگ به تو داد: با `Rc`، یک مقدار تا وقتی زنده می‌ماند که دست‌کم یک مالکِ قوی داشته باشد، و همان لحظه‌ای که آخرین مالک هم رهایش کند، خودش را تمیز پاک می‌کند. `Rc::clone` ارزان است، شمارنده بالا می‌رود؛ یک `Rc` که drop می‌شود، شمارنده پایین می‌آید؛ به صفر که برسد، مقدار آزاد می‌شود. این قول درست است — ولی یک پیش‌فرضِ ناگفته دارد که این درس رویش انگشت می‌گذارد: فرض می‌کند شمارنده *بالاخره* به صفر می‌رسد.

خبرِ بد این‌جاست: Rust هیچ تضمینی نمی‌دهد که این‌طور بشود. اگر دو مقدار، هرکدام یک `Rc` قوی به آن یکی نگه دارند، شمارنده‌ی هیچ‌کدام هرگز به صفر نمی‌رسد — نه چون کدت باگ دارد، نه چون جایی پنیک گرفته‌ای، بلکه چون خودِ شکلِ دیتا این را غیرممکن کرده. این یک نشتِ حافظه‌ی واقعی است، همان‌جور مقدارها برای همیشه رویِ هیپ می‌مانند و `Drop::drop`شان هیچ‌وقت اجرا نمی‌شود، تا وقتی برنامه دارد اجرا می‌شود.

این را باید صریح و صادقانه گفت، چون نصفه‌گفتنش بدتر از نگفتنش است: Rust جلوی مسابقه‌ی داده و دسترسیِ بعد از آزادسازی (use-after-free) را با تمامِ قدرتِ سیستمِ نوعش می‌گیرد — این دو دسته باگ در Rustِ ایمن واقعاً نوشتنی نیستند. ولی نشتِ حافظه هیچ‌وقت جزوِ آن قول نبوده. این درس دقیقاً نشانت می‌دهد کجا این مرز رد می‌شود، و — چون این مشکل واقعی است — ابزاری که دقیقاً برایش ساخته شده: `Weak<T>`.

---

## مفهوم

### چرخه‌ای که خودش را هرگز آزاد نمی‌کند

فرض کن دو تا `Friend` داری و هرکدام می‌خواهد بهترین دوستش را نگه دارد. ساده‌ترین چیزی که به ذهن می‌رسد این است: هرکدام یک `Rc` قوی به آن یکی نگه دارد.

```rust
struct Friend {
    name: String,
    best_friend: RefCell<Option<Rc<Friend>>>,
}

let alice = Rc::new(Friend { name: "Alice".to_string(), best_friend: RefCell::new(None) });
let bob = Rc::new(Friend { name: "Bob".to_string(), best_friend: RefCell::new(None) });

*alice.best_friend.borrow_mut() = Some(Rc::clone(&bob));
*bob.best_friend.borrow_mut() = Some(Rc::clone(&alice));

println!("alice strong_count: {}", Rc::strong_count(&alice));
println!("bob strong_count:   {}", Rc::strong_count(&bob));
```

```text
alice strong_count: 2
bob strong_count:   2
```

فیلدِ `best_friend` را داخلِ یک `RefCell` پیچیده‌ایم تا بشود بعد از این‌که `alice` و `bob` هر دو از قبل پشتِ یک `Rc` هستند، مقدارش را عوض کرد — بدونش هیچ راهی نیست که همزمان `alice` به `bob` اشاره کند و `bob` هم به `alice`. `RefCell` موضوعِ [۲.۶.۵](../05-refcell-and-interior-mutability/README.fa.md) است؛ فعلاً همین‌قدر کافی است: `.borrow_mut()` یعنی «این فیلد را موقتاً برایِ نوشتن باز کن»، حتی وقتی فقط یک ارجاعِ اشتراکی داری.

شمارنده‌ی هرکدام از ۱ رفته رویِ ۲: یک واحد برایِ بایندِ محلیِ خودش (`alice` یا `bob`)، یک واحد برایِ کپی‌ای که آن یکی نگه داشته. حالا این دو بایندِ محلی را drop کن:

```rust
drop(alice);
drop(bob);
```

```text
both local bindings dropped — no "dropping: ..." line printed above
```

هیچ‌کدام drop نشدند. هر `Friend` قبل از این هم یک `impl Drop` داشت که اسمش را چاپ می‌کرد — و آن پیام هیچ‌وقت چاپ نشد، نه برایِ آلیس و نه برایِ باب. `drop(alice)` فقط بایندِ محلیِ `alice` را از بین می‌برد؛ شمارنده از ۲ می‌آید رویِ ۱ — چون کپیِ داخلِ `bob.best_friend` هنوز زنده است. `drop(bob)` دقیقاً همین کار را با شمارنده‌ی `bob` می‌کند. هر دو مقدار الان رویِ هیپ نشسته‌اند، هرکدام فقط با یک ارجاعِ قوی — از دلِ آن یکی — زنده نگه داشته شده، و هیچ متغیرِ نام‌داری در هیچ‌کجایِ برنامه به هیچ‌کدامشان اشاره نمی‌کند. یک چرخه‌ی دونفره‌ی کاملاً جدا از بقیه‌ی برنامه، که تا وقتی برنامه اجرا می‌شود، هیچ‌وقت پاک نمی‌شود.

```senpai-visual
{"kind":"ownership","labels":["آلیس، باب را قوی نگه می‌دارد","باب، آلیس را قوی نگه می‌دارد","هر دو شمارنده: ۲","هر دو بایند drop می‌شوند","هر دو شمارنده: ۱ — هرگز صفر"]}
```

### `Weak<T>` — دستگیره‌ای که مالکیت نمی‌گیرد

اگر یکی از آن دو ارجاع اصلاً در شمارنده‌ی قوی حساب نمی‌شد چه؟ دقیقاً همین کارِ `Weak<T>` است: یک دستگیره به همان مقدار، که نگه‌داشتنش مقدار را زنده نگه نمی‌دارد و در `Rc::strong_count` هیچ اثری ندارد. با `Rc::downgrade(&rc)` می‌سازیش:

```rust
let strong = Rc::new(String::from("shared"));
let weak: Weak<String> = Rc::downgrade(&strong);

println!("strong_count: {}", Rc::strong_count(&strong));
println!("weak_count:   {}", Rc::weak_count(&strong));
```

```text
strong_count: 1
weak_count:   1
```

شمارنده‌ی قوی همچنان ۱ است — فقط یک شمارنده‌ی جداگانه، شمارنده‌ی ضعیف، بالا رفته. ولی حالا یک مشکل داری: چطور از رویِ یک `Weak<String>` به خودِ `String` برسی؟ نمی‌شود مستقیم به‌اش دسترسی داشت — و این عمدی است. یک `Weak` نمی‌تواند قول بدهد مقدارش هنوز هست؛ صاحبِ آخرش شاید همین الان drop شده باشد. تنها راهِ رسیدن به مقدار، متدِ `.upgrade()` است، که یک `Option<Rc<T>>` برمی‌گرداند — `Some` اگر دست‌کم یک مالکِ قوی هنوز زنده باشد، `None` اگر همه‌شان رفته باشند:

```rust
match weak.upgrade() {
    Some(value) => println!("upgrade while alive: got {value:?}"),
    None => println!("upgrade while alive: got nothing"),
}
```

```text
upgrade while alive: got "shared"
```

حالا مالکِ قوی را drop کن و همان سؤال را دوباره بپرس:

```rust
drop(strong);

match weak.upgrade() {
    Some(value) => println!("upgrade after drop:  got {value:?}"),
    None => println!("upgrade after drop:  got nothing"),
}
```

```text
upgrade after drop:  got nothing
```

هیچ چیزِ اسرارآمیزی این‌جا نیست — همان `Option<T>`ی است که از [۱.۶.۱](../../../phase1-fundamentals/06-absence-and-failure/01-option-and-null-safety/README.fa.md) می‌شناسی، فقط این‌بار `T`اش `Rc<T>` است. و دقیقاً به همان دلیلی که آن‌جا `Option` تصادفی نبود — یک جست‌وجو که شاید چیزی پیدا نکند نباید تظاهر کند همیشه پیدا می‌کند — همین‌جا هم `Option<Rc<T>>`، تظاهر به قطعیتی است که `Weak` از اساس نمی‌تواند بکند. `weak_count` هم مثلِ `strong_count` کم می‌شود، فقط با یک قاعده‌ی متفاوت: یک `Weak` که drop می‌شود شمارنده‌ی ضعیف را کم می‌کند، ولی مقدارِ زیرینش را — تا وقتی شمارنده‌ی قوی از قبل به صفر نرسیده باشد — دست‌نخورده نگه می‌دارد.

### راه‌حلِ استاندارد: قوی به پایین، ضعیف به بالا

بازگرد به آن چرخه. مشکلِ آلیس و باب این نبود که هرکدام دیگری را نگه داشتند — مشکل این بود که هر دو طرفِ رابطه *قوی* بود. یک رابطه‌ای که واقعاً یک جهتِ طبیعی دارد — یک درختِ پدر/فرزند — همین را عیان می‌کند: پدر *صاحبِ* فرزندهایش است (اگر پدر برود، فرزندها هم باید بروند)؛ فرزند صاحبِ پدرش نیست (فرزند فقط باید بتواند پدرش را *پیدا* کند، نه او را زنده نگه دارد). پس ارجاعِ رو‌به‌پایین (پدر به فرزند) قوی می‌ماند؛ ارجاعِ برگشتی (فرزند به پدر) `Weak` می‌شود.

```rust
struct Folder {
    name: String,
    parent: Weak<Folder>,
    children: Vec<Rc<Folder>>,
}

let root = Rc::new_cyclic(|weak_root| Folder {
    name: "root".to_string(),
    parent: Weak::new(),
    children: vec![
        Rc::new(Folder { name: "docs".to_string(), parent: weak_root.clone(), children: vec![] }),
        Rc::new(Folder { name: "src".to_string(), parent: weak_root.clone(), children: vec![] }),
    ],
});
```

این‌بار به هیچ `RefCell`ای نیاز نبود. `Rc::new_cyclic` دقیقاً برایِ همین شکل ساخته شده: پیش از این‌که `root` واقعاً به‌عنوانِ یک `Rc` وجود داشته باشد، یک `Weak<Folder>` به کلوژرش می‌دهد که *بعداً*، وقتی ساختش تمام شد، به همان `root` اشاره می‌کند. هر فرزند همان لحظه‌ی ساخته‌شدنش یک `weak_root.clone()` می‌گیرد — یک `Weak` کارآمد به پدری که، درست همین لحظه، هنوز کاملاً ساخته نشده.

```rust
for child in &root.children {
    match child.parent.upgrade() {
        Some(parent) => println!("{}'s parent is {}", child.name, parent.name),
        None => println!("{}'s parent is gone", child.name),
    }
}
println!("root strong_count: {}", Rc::strong_count(&root));
```

```text
docs's parent is root
src's parent is root
root strong_count: 1
```

با این‌که دو فرزند هر دو به `root` اشاره می‌کنند، شمارنده‌ی قوی‌اش همچنان ۱ است — چون هیچ‌کدام از آن اشاره‌ها قوی نیست. حالا `root` را drop کن:

```rust
drop(root);
```

```text
dropping: root
dropping: docs
dropping: src
```

این‌بار همه چیز پاک شد، به همان ترتیبِ طبیعی: تنها مالکِ قوی‌یِ `root`، همان بایندِ محلیِ `root` بود؛ وقتی رفت، شمارنده‌ی `root` به صفر رسید و `Folder::drop`ش اجرا شد. آن `drop` خودش فیلدِ `children`اش را هم آزاد می‌کند — که یعنی آن دو کپیِ قویِ `docs` و `src` هم می‌روند، شمارنده‌ی هرکدام به صفر می‌رسد، و `Folder::drop`شان هم اجرا می‌شود. یک اثرِ زنجیره‌ای، دقیقاً همان‌طور که با یک درختِ معمولی و یک مالکیتِ ساده انتظار داری — نه چون تلاش کردیم مراقب باشیم، بلکه چون جهتِ ارجاع‌ها دیگر چرخه‌ای نمی‌سازد.

```senpai-visual
{"kind":"ownership","labels":["root فرزندهایش را قوی نگه می‌دارد","فرزندها root را ضعیف نگه می‌دارند","شمارنده‌ی root: ۱","root حذف می‌شود","فرزندها هم حذف می‌شوند — شمارنده ۰"]}
```

---

## دست‌به‌کد

```sh
cargo run -p p2-06-04-weak-and-reference-cycles --example 01-the-cycle-problem
cargo run -p p2-06-04-weak-and-reference-cycles --example 02-weak-and-upgrade
cargo run -p p2-06-04-weak-and-reference-cycles --example 03-the-fix-parent-child-tree
```

بعد سه‌تای خراب:

```sh
cargo run -p p2-06-04-weak-and-reference-cycles --example 04-upgrade-is-not-the-value --features broken
cargo run -p p2-06-04-weak-and-reference-cycles --example 05-weak-has-no-direct-methods --features broken
cargo run -p p2-06-04-weak-and-reference-cycles --example 06-cannot-mutate-through-shared-rc --features broken
```

بعد این‌ها را امتحان کن:

۱. در `01-the-cycle-problem`، `Rc::strong_count(&alice)` را همان قبل از دو خطِ `borrow_mut` چاپ کن. چند است، و چرا با بعدش فرق دارد؟
۲. در `02-weak-and-upgrade`، قبل از `drop(strong)` یک `let _second = Rc::clone(&strong);` اضافه کن. حالا بعد از `drop(strong)`، `weak.upgrade()` چه می‌دهد؟ چرا؟
۳. در `03-the-fix-parent-child-tree`، یک فرزندِ سوم به نامِ `"tests"` به `root` اضافه کن و دوباره اجرایش کن. ترتیبِ خط‌های `dropping: ...` چطور عوض می‌شود؟

---

## خطاهایی که خواهی دید

### `E0308` — `.upgrade()` خودِ مقدار را نمی‌دهد

```text
error[E0308]: mismatched types
  --> phase2-intermediate\06-smart-pointers\04-weak-and-reference-cycles\examples\04-upgrade-is-not-the-value.rs:26:16
   |
26 |     print_name(&maybe_folder);
   |     ---------- ^^^^^^^^^^^^^ expected `&Rc<Folder>`, found `&Option<Rc<Folder>>`
   |     |
   |     arguments to this function are incorrect
   |
   = note: expected reference `&Rc<_>`
              found reference `&Option<Rc<_>>`
note: function defined here
  --> phase2-intermediate\06-smart-pointers\04-weak-and-reference-cycles\examples\04-upgrade-is-not-the-value.rs:15:4
   |
15 | fn print_name(folder: &Rc<Folder>) {
   |    ^^^^^^^^^^ -------------------
```

**کامپایلر به چه اعتراض دارد:** `weak.upgrade()` یک `Option<Rc<Folder>>` برگرداند، نه یک `Rc<Folder>`. کدِ بالا آن `Option` را مستقیم به تابعی داده که یک `&Rc<Folder>` می‌خواهد — دو نوعِ متفاوت، و کامپایلر هیچ‌وقت این دو را خودش قاطی نمی‌کند.

**راه‌حل:** اول با هر دو حالتِ `Option` روبه‌رو شو، بعد جواب را بده:

```rust
match weak.upgrade() {
    Some(folder) => print_name(&folder),
    None => println!("(gone)"),
}
```

**چرا این راه‌حل است:** `.upgrade()` نمی‌تواند قول بدهد مقدار هنوز هست — این دقیقاً همان قولی است که یک `Weak` از اساس نمی‌تواند بدهد — پس نوعِ برگشتی‌اش این را می‌گوید، نه چیزِ دیگری. تنها راهِ رسیدن به `Rc<Folder>`یِ داخلش، جواب‌دادن به هر دو حالت است.

### `E0599` — `Weak<T>` خودش هیچ متدی از `T` ندارد

```text
error[E0599]: no method named `shout` found for struct `std::rc::Weak<T, A>` in the current scope
  --> phase2-intermediate\06-smart-pointers\04-weak-and-reference-cycles\examples\05-weak-has-no-direct-methods.rs:27:25
   |
27 |     println!("{}", weak.shout());
   |                         ^^^^^ method not found in `std::rc::Weak<Folder>`
```

**کامپایلر به چه اعتراض دارد:** `Folder` یک متدِ `shout` دارد، ولی `weak` از نوعِ `Weak<Folder>` است، نه `Folder` و نه حتی `Rc<Folder>`. برخلافِ `Rc<T>`، نوعِ `Weak<T>` خصیصه‌ی `Deref` را پیاده نمی‌کند — یعنی هیچ بازکردنِ خودکاری به سمتِ `T` وجود ندارد، چون کامپایلر نمی‌تواند از قبل ثابت کند `T` هنوز آن‌جاست.

**راه‌حل:** اول `.upgrade()` کن، بعد متد را رویِ نتیجه صدا بزن:

```rust
if let Some(folder) = weak.upgrade() {
    println!("{}", folder.shout());
}
```

**چرا این راه‌حل است:** `.upgrade()` دقیقاً همان قدمی است که از یک «شاید هست» به یک `Rc<Folder>`یِ واقعی می‌رسد — تنها چیزی که `Deref` رویش تعریف شده. تا آن قدم را برنداری، هیچ متدی از `Folder` در دسترس نیست.

### `E0594` — نمی‌شود از پشتِ یک `Rc` اشتراکی چیزی را نوشت

```text
error[E0594]: cannot assign to data in an `Rc`
  --> phase2-intermediate\06-smart-pointers\04-weak-and-reference-cycles\examples\06-cannot-mutate-through-shared-rc.rs:26:5
   |
26 |     docs.parent = Rc::downgrade(&root);
   |     ^^^^^^^^^^^ cannot assign
   |
   = help: trait `DerefMut` is required to modify through a dereference, but it is not implemented for `Rc<Folder>`
```

**کامپایلر به چه اعتراض دارد:** `docs` از نوعِ `Rc<Folder>` است. `Rc<T>` خصیصه‌ی `Deref` را پیاده می‌کند (برایِ همین `docs.parent` رویِ خواندن کار می‌کند) ولی `DerefMut` را نه — یعنی از پشتِ یک `Rc` هیچ‌وقت `&mut T` نمی‌گیری، مهم نیست چند مالک الان زنده‌اند.

**راه‌حل:** یا یک فیلدِ نیازمندِ نوشتنِ بعدی را از اول داخلِ `RefCell` بگذار — دقیقاً همان کاری که `01-the-cycle-problem.rs` کرد — یا، اگر می‌شود، مثلِ `03-the-fix-parent-child-tree.rs` کل چیزی که به هم وابسته است را یک‌جا و با `Rc::new_cyclic` بساز، طوری که دیگر لازم نباشد بعداً چیزی را عوض کنی.

**چرا این راه‌حل است:** این همان دیواری است که `Weak` و `Rc::new_cyclic` دورش می‌زنند، هرکدام به روشِ خودشان — یکی با اجازه دادنِ نوشتن از پشتِ یک ارجاعِ اشتراکی (`RefCell`)، دیگری با ساختنِ همه‌چیز پیش از آن‌که اصلاً به این دیوار بخوری. کامپایلر این‌جا حق دارد: هیچ راهی نیست که دو مالکِ همزمان، بدونِ هماهنگی، هر دو `&mut` بگیرند — این دقیقاً همان قاعده‌ی هم‌نامی‌ای است که از [۱.۳.۱](../../../phase1-fundamentals/03-borrowing-and-references/01-shared-and-mutable-refs/README.fa.md) می‌شناسی.

---

## تمرین

### گرم‌کردن

<details>
<summary>این چه چاپ می‌کند؟</summary>

```rust
let a = Rc::new(5);
let b = Rc::clone(&a);
let weak = Rc::downgrade(&a);
drop(a);
drop(b);
println!("{:?}", weak.upgrade());
```

</details>

<details>
<summary>پاسخ</summary>

```text
None
```

`weak` هیچ‌وقت در شمارنده‌ی قوی حساب نمی‌شد. `a` و `b` هر دو مالکِ قوی بودند؛ وقتی هر دو drop شدند، شمارنده‌ی قوی به صفر رسید و مقدار آزاد شد — پس دیگر چیزی برایِ `.upgrade()` نمانده.

</details>

<details>
<summary>و این یکی؟</summary>

```rust
let rc = Rc::new(10);
let weak = Rc::downgrade(&rc);
println!("{:?}", weak.upgrade());
```

</details>

<details>
<summary>پاسخ</summary>

```text
Some(10)
```

`rc` هنوز زنده است — هیچ‌کس drop نشده. `.upgrade()` یک `Rc<i32>`ِ تازه (و ارزان) به همان مقدار برمی‌گرداند؛ شمارنده‌ی قوی برایِ یک لحظه ۲ می‌شود، تا همان `Rc`ِ تازه هم drop شود.

</details>

<details>
<summary>چرا ساختنِ یک <code>Weak&lt;T&gt;</code> هیچ‌وقت شمارنده‌ی قوی را عوض نمی‌کند؟</summary>

چون یک `Weak` مالکیت نمی‌گیرد — قول نمی‌دهد مقدار را زنده نگه دارد. شمارنده‌ی قوی دقیقاً همان چیزی است که تصمیم می‌گیرد مقدار کِی آزاد شود؛ اگر یک `Weak` رویش حساب می‌شد، دیگر نمی‌شد با آن چرخه را شکست، چون خودش هم به همان اندازه‌ی یک `Rc` مقدار را زنده نگه می‌داشت.

</details>

<details>
<summary>در چرخه‌ی آلیس و باب، بعد از این‌که هر دو بایندِ محلی drop شوند، <code>Drop::drop</code> برایِ کدام‌شان اجرا می‌شود؟</summary>

هیچ‌کدام. هرکدام هنوز یک ارجاعِ قوی — از دلِ آن یکی — دارد؛ شمارنده‌ی هرکدام از ۲ می‌رسد به ۱، نه به صفر. تا وقتی برنامه در حالِ اجراست، هر دو رویِ هیپ می‌مانند، بدونِ هیچ متغیرِ نام‌داری که به آن‌ها برسد.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/04-upgrade-is-not-the-value.rs` را طوری درست کن که کامپایل شود — با یک `match` یا `if let` که واقعاً هر دو حالتِ `Option`یِ `.upgrade()` را جواب بدهد، نه با `.unwrap()` بی‌دلیل.
۲. `examples/05-weak-has-no-direct-methods.rs` را طوری درست کن که `shout()` واقعاً صدا زده شود، وقتی `Folder` هنوز زنده است.
۳. `examples/06-cannot-mutate-through-shared-rc.rs` را **دو** جور درست کن: یک بار با گذاشتنِ `parent` داخلِ `RefCell` (مثلِ `01-the-cycle-problem.rs`)، یک بار با ساختنِ کلِ درخت یک‌جا با `Rc::new_cyclic` (مثلِ `03-the-fix-parent-child-tree.rs`) به‌جایِ این‌که بعداً بخواهی چیزی را عوض کنی.

### پیاده‌سازی

چهار تابع در `src/lib.rs`:

```sh
cargo test -p p2-06-04-weak-and-reference-cycles
```

هیچ‌کدام به `RefCell` یا ساختنِ یک درختِ تازه نیاز ندارد — درختِ نمونه از قبل ساخته شده؛ کارِ تو فقط خواندنش است.

### بساز

یک `pub fn full_path(folder: &Rc<Folder>) -> String` بنویس که مسیر را از ریشه تا `folder` برگرداند، با فرمتی که خودت انتخاب می‌کنی (مثلاً چیزی شبیهِ `"root/docs"`) و در کامنتِ مستنداتِ تابع دقیقاً همان فرمت را می‌نویسی.

### چالش (اختیاری)

**بخشِ یک.** یک درختِ سه‌سطحی بساز — `root` که یک `docs` دارد، و `docs` خودش یک `guide.md` دارد — با تو در تو کردنِ `Rc::new_cyclic` (هر سطحی که فرزند دارد، خودش هم باید با `Rc::new_cyclic` ساخته شود تا بتواند یک `Weak` کارآمد به فرزندهایش بدهد). بعد `depth()` را رویِ `guide.md` صدا بزن — چند برمی‌گرداند؟

**بخشِ دو.** (این یکی جلوتر را نگاه می‌کند.) درختِ این درس، بعد از ساخته‌شدن، دیگر رشد نمی‌کند — نمی‌شود بعداً یک فرزندِ تازه به `root` اضافه کرد، چون `children` یک `Vec<Rc<Folder>>` معمولی است، نه چیزی که از پشتِ یک ارجاعِ اشتراکی بشود نوشت. حدس بزن: در [۲.۶.۵](../05-refcell-and-interior-mutability/README.fa.md)، نوعِ کدام فیلد یا فیلدهایِ `Folder` باید عوض شود تا بشود بعد از ساختنِ درخت هم به آن فرزند اضافه کرد؟

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به‌کارت می‌آید |
|---|---|---|
| `Weak<T>` | دستگیره‌ای غیرمالک به مقدارِ مدیریت‌شده با `Rc`/`Arc` | ارجاعِ برگشتی در هر ساختارِ دوطرفه |
| `Rc::downgrade(&rc)` | ساختنِ یک `Weak<T>` از رویِ یک `Rc<T>` | جایی که فقط باید بتوانی *پیدا* کنی، نه زنده نگه‌داری |
| `.upgrade()` | تنها راهِ رسیدن به مقدار از رویِ یک `Weak`؛ `Option<Rc<T>>` می‌دهد | هر بار که واقعاً به مقدار نیاز داری |
| چرخه‌ی ارجاع | چند مقدار که با ارجاعِ *قوی* حلقه‌وار به هم اشاره می‌کنند | دقیقاً چیزی که این درس نشانت داد چطور بسازی و چطور نسازی |
| `Rc::new_cyclic` | سازنده‌ای که پیش از تمام‌شدنِ ساخت، یک `Weak<T>` به همان مقدار به کلوژرش می‌دهد | ساختنِ یک‌باره‌ی درخت/گراف، بدونِ نیاز به `RefCell` |

### الان می‌دانی

- safe Rust جلوی مسابقه‌ی داده و دسترسیِ بعد از آزادسازی را می‌گیرد، ولی جلوی همه‌ی نشتِ حافظه را نه — دو `Rc` که به هم قوی اشاره می‌کنند، شمارنده‌شان هرگز به صفر نمی‌رسد.
- `Weak<T>` مالکیت نمی‌گیرد و در `Rc::strong_count` حساب نمی‌شود؛ فقط راهِ رسیدن به مقدار، `.upgrade()`، یک `Option<Rc<T>>` می‌دهد، چون نمی‌تواند قول بدهد مقدار هنوز هست.
- راه‌حلِ استاندارد برایِ یک درختِ پدر/فرزند: ارجاعِ رو‌به‌پایین (پدر به فرزند) قوی می‌ماند؛ ارجاعِ برگشتی (فرزند به پدر) `Weak` می‌شود — چون مالکیتِ طبیعیِ یک درخت هم همین جهت را دارد.
- `Rc::new_cyclic` می‌گذارد یک درخت را یک‌جا بسازی، طوری که هر فرزند از همان لحظه‌ی ساخته‌شدن یک `Weak` کارآمد به پدرش دارد — بدونِ نیاز به `RefCell`.
- `Rc<T>` فقط `Deref` را پیاده می‌کند، نه `DerefMut`؛ نوشتن از پشتِ یک `Rc` اشتراکی، مهم نیست چند مالک الان زنده‌اند، همیشه رد می‌شود.

### بعداً کامل‌تر می‌بینی

- **`RefCell` و تغییرپذیریِ درونی — چطور اصلاً می‌شود از پشتِ یک ارجاعِ اشتراکی چیزی را نوشت** — [۲.۶.۵](../05-refcell-and-interior-mutability/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا دو `Rc` که به‌طورِ متقابل همدیگر را قوی نگه می‌دارند هرگز drop نمی‌شوند؟
- چرا نگه‌داشتنِ یک `Weak<T>` شمارنده‌ی قوی را عوض نمی‌کند؟
- `.upgrade()` چرا `Option<Rc<T>>` برمی‌گرداند، نه خودِ `Rc<T>` را؟
- در یک درختِ پدر/فرزند، چرا ارجاعِ قوی رو‌به‌پایین می‌رود و ارجاعِ ضعیف رو‌به‌بالا، نه برعکس؟
- `Rc::new_cyclic` چه مشکلی را حل می‌کند که یک `Rc::new` معمولی نمی‌تواند؟

---

## بیشتر

- [کتابِ Rust — چرخه‌های ارجاع می‌توانند حافظه نشت بدهند](https://doc.rust-lang.org/book/ch15-06-reference-cycles.html) — همین درس، از زبانِ رسمی.
- [`std::rc::Weak`](https://doc.rust-lang.org/std/rc/struct.Weak.html) — فهرستِ کاملِ متدهایش.
- [`Rc::new_cyclic`](https://doc.rust-lang.org/std/rc/struct.Rc.html#method.new_cyclic) — مستنداتِ رسمی، با یک مثالِ دیگر از همین الگو.
