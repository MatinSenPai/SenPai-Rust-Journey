# پاسخ تشریحی

```rust
pub fn split_at_mut_demo<T>(slice: &mut [T], mid: usize) -> (&mut [T], &mut [T]) {
    let len = slice.len();
    assert!(mid <= len, "mid out of bounds");
    let ptr = slice.as_mut_ptr();
    // SAFETY: mid <= len؛ هر دو بازه داخل همان allocation و بدون هم‌پوشانی‌اند.
    unsafe {
        (
            std::slice::from_raw_parts_mut(ptr, mid),
            std::slice::from_raw_parts_mut(ptr.add(mid), len - mid),
        )
    }
}
```

proof ایمنی سه بخش دارد: `ptr` از slice معتبر آمده، `mid <= len` هر دو pointer را در محدوده یا یک-past-end نگه می‌دارد، و بازه‌های `[0, mid)` و `[mid, len)` overlap ندارند؛ پس ساخت هم‌زمان دو `&mut` قانون aliasing را نمی‌شکند.

بلوک unsafe این ادعا را بررسی نمی‌کند. اگر assertion ضعیف شود یا length اشتباه محاسبه شود، کد ممکن است undefined behavior ایجاد کند. به همین دلیل بلوک باید کوچک و SAFETY comment هم‌جوار invariant باشد.
