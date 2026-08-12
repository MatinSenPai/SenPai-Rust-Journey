# پاسخ تشریحی

```rust
pub fn from_vec(items: &[i32]) -> List {
    let mut list = List::Nil;
    for &item in items.iter().rev() {
        list = List::Cons(item, Box::new(list));
    }
    list
}
```

جهت حلقه مهم است. cons list فقط pointer رو‌به‌جلو دارد و appendکردن به انتهای فهرست موجود مستلزم پیمایش تا `Nil` و بازسازی nodeهاست. پس `items` را با `.iter().rev()` از آخر به اول می‌خوانیم و فهرست را از درون می‌سازیم: ابتدا آخرین عضو `Cons(last, Nil)` می‌شود و هر عضو قبلی یک لایه‌ی تازه دور نتیجه می‌گذارد. در پایان نخستین عضو ورودی بیرونی‌ترین `Cons` است و ترتیب حفظ می‌شود.

```rust
pub fn sum(&self) -> i32 {
    match self {
        List::Cons(val, rest) => val + rest.sum(),
        List::Nil => 0,
    }
}
```

چون روی `&self` match می‌کنیم، نوع `rest` برابر `&Box<List>` است. `rest.sum()` به‌کمک `Deref<Target = List>` در `Box<List>` و auto-deref متدها کار می‌کند؛ Rust هر تعداد dereference لازم را برای یافتن متد مناسب وارد می‌کند.

اگر `sum` به‌جای `&self`، `self` می‌گرفت، `list.sum()` کل فهرست را مصرف می‌کرد و nodeها هنگام recursion move و drop می‌شدند. برای استفاده‌ی دوباره باید فهرست را بازسازی می‌کردی. جمع‌زدن نه تغییر می‌خواهد و نه مالکیت، پس `&self` کمترین دسترسی درست است.
