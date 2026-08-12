# راه‌حل

```rust
pub fn drop_order_in_one_scope() -> Vec<String> {
    take_log();
    {
        let _a = Tracker::new("a");
        let _b = Tracker::new("b");
        let _c = Tracker::new("c");
    }
    take_log()
}
```

خروجی `c, b, a` است. localها برعکس ترتیب ساخت teardown می‌شوند تا مقدار جدیدتر پیش از چیزی که ممکن است به آن وابسته باشد از بین برود.

```rust
pub fn early_drop_demo() -> Vec<String> {
    take_log();
    {
        let first = Tracker::new("first");
        let _second = Tracker::new("second");
        drop(first);
    }
    take_log()
}
```

`drop(first)` مالکیت را به یک تابع معمولی می‌دهد و محدوده آن تابع فوراً تمام می‌شود؛ پس log برابر `first, second` است.

```rust
fn create_tracker(name: &str) -> Tracker {
    Tracker::new(name)
}

pub fn move_extends_lifetime() -> Vec<String> {
    take_log();
    {
        let _t = create_tracker("moved");
    }
    take_log()
}
```

return، مالکیت `Tracker` را به فراخوان منتقل می‌کند. destructor فقط یک‌بار و در پایان محدوده مالک نهایی اجرا می‌شود.
