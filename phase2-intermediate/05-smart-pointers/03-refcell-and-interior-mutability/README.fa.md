# ۰۵.۳ — `RefCell` و تغییرپذیری درونی

## انتقال بررسی قانون از compile time به runtime

قانون «aliasing XOR mutability» می‌گوید در هر لحظه یا چند `&T` خواندنی داریم، یا دقیقاً یک `&mut T` انحصاری؛ نه هر دو. معمولاً کامپایلر این قانون را پیش از اجرا بررسی می‌کند. `RefCell<T>` دقیقاً **همان قانون** را در runtime اجرا می‌کند.

```rust
use std::cell::RefCell;
let cell = RefCell::new(5);
let a = cell.borrow();
println!("{}", *a);
drop(a);
let mut b = cell.borrow_mut();
*b += 1;
```

`.borrow()` و `.borrow_mut()` از طریق `&RefCell<T>` کار می‌کنند و برای دومی به `&mut` بیرونی نیاز نیست. `RefCell` تعداد borrowهای زنده را نگه می‌دارد. اگر در حضور borrow دیگری `borrow_mut()` بخواهی، کد compile می‌شود اما در runtime با `already borrowed: BorrowMutError` panic می‌کند.

```senpai-visual
{"kind":"borrowing","labels":["Rc مشترک","borrow خواندنی","borrow_mut انحصاری","پایان guard"]}
```

این ابزار وقتی مفید است که borrow checker به‌علت abstractionهای چندلایه نتواند ایمنی واقعی را ثابت کند؛ مثلاً callback، trait method یا tree با childهای مشترک. `RefCell` راه فرار امن است: تضمین زودهنگام را با check دیرتر عوض می‌کنی و در صورت نقض، panic می‌گیری نه memory corruption. این `unsafe` نیست.

## ترکیب مهم `Rc<RefCell<T>>`

`Rc<T>` چند مالک می‌دهد ولی هر مالک فقط `&T` دارد. با قراردادن `T` در `RefCell`، هر clone می‌تواند `.borrow_mut()` کند و همان داده‌ی مشترک را تغییر دهد:

```rust
let counter = Rc::new(RefCell::new(0));
let handle = Rc::clone(&counter);
*handle.borrow_mut() += 1;
*counter.borrow_mut() += 1;
assert_eq!(*counter.borrow(), 2);
```

این ترکیب برای state مشترک و تغییرپذیر تک‌thread است. نسخه‌ی thread-safe آن `Arc<Mutex<T>>` است. اما ابزار باید آگاهانه و گاه‌به‌گاه استفاده شود. اگر یک مالک روشن و انتقال `&mut` ممکن است، همان ساده‌تر و بدون هزینه‌ی runtime است. کاربردهای مناسب: graph با node مشترک، observerها یا cache میان زیرسیستم‌های مستقل.

مثل دفتر ثبت مشترکی است که چند باجه به آن دسترسی دارند، اما فقط یک باجه در لحظه مجوز ویرایش می‌گیرد. مرز تشبیه: `RefCell` منتظر آزادشدن مجوز نمی‌ماند؛ اگر قانون را بشکنی panic می‌کند.

## تمرین تو

`SharedCounter` را در `src/lib.rs` پیاده کن، سپس `CHECKPOINT.fa.md` و `solution/SOLUTION.fa.md` را بخوان.
