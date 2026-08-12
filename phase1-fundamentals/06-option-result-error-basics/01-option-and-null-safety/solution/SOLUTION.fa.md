# راه‌حل

```rust
pub fn find_by_id(users: &[(u32, String)], id: u32) -> Option<String> {
    users.iter().find(|(uid, _)| *uid == id).map(|(_, name)| name.clone())
}
```

`find` یک `Option<&(u32, String)>` می‌دهد و `map` فقط حالت `Some` را به نام cloned تبدیل می‌کند. خروجی دارای مالکیت از borrow فهرست مستقل می‌شود. این clone یک انتخاب قراردادی است؛ اگر فراخوان و lifetime اجازه دهند، return کردن ارجاع می‌تواند تخصیص حافظه را حذف کند.

```rust
pub fn average_known_age(ages: &[Option<u32>]) -> Option<f64> {
    let known: Vec<u32> = ages.iter().filter_map(|a| *a).collect();
    if known.is_empty() {
        return None;
    }
    let sum: u32 = known.iter().sum();
    Some(sum as f64 / known.len() as f64)
}
```

`filter_map` فقط مقدارهای `Some` را نگه می‌دارد. میانگینِ مجموعه‌ی بدون عضو، مقدار معناداری از نوع `f64` ندارد؛ بنابراین `None` بخشی از معنای مسئله است، نه صرفاً یک برنامه‌نویسی دفاعی.
