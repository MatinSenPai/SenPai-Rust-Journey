# پاسخ تشریحی

```rust
pub fn increment(&self) {
    *self.inner.borrow_mut() += 1;
}

pub fn get(&self) -> i32 {
    *self.inner.borrow()
}
```

`self.inner` از نوع `Rc<RefCell<i32>>` است. `.borrow_mut()` از راه ارجاع مشترک `Rc` به `RefCell` کار می‌کند و `RefMut<i32>` می‌دهد؛ `*` به `i32` واقعی برای خواندن یا نوشتن می‌رسد. guardهای `RefMut` و `Ref` در پایان هر statement drop می‌شوند، چون temporary هستند. بنابراین دو `increment()` پشت سر هم panic نمی‌کنند؛ borrow اول پیش از شروع بعدی آزاد شده است.

بدون `RefCell`، نوع `Rc<i32>` به هر clone فقط `&i32` می‌دهد و API آن راهی برای تغییر مقصد از پشت ارجاع مشترک ندارد. پس متد دارای `&self` فقط می‌تواند بخواند. `RefCell` همان ابزاری است که تغییرپذیری را برمی‌گرداند و invariant «فقط یک دسترسی تغییرپذیر» را از compile time به runtime منتقل می‌کند.
