# ۰۶.۱ — `Option` و ایمنی در برابر null

یک مقدار از نوع `User` همیشه واقعاً `User` است. اگر ممکن است وجود نداشته باشد، نبودن بخشی از نوع می‌شود:

```rust
enum Option<T> {
    Some(T),
    None,
}
```

`find_user(id) -> Option<User>` فراخوان را مجبور می‌کند پیش از دسترسی به `User`، حالت `None` را مدیریت کند. این همان enum معمولی استاندارد است.

```rust
let maybe_age: Option<u32> = Some(25);

maybe_age.is_some();
maybe_age.is_none();
maybe_age.unwrap();
maybe_age.unwrap_or(0);
maybe_age.unwrap_or_default();
maybe_age.map(|age| age + 1);
if let Some(age) = maybe_age { /* ... */ }
```

`unwrap()` روی `None` panic می‌کند. `map` فقط مقدار داخل `Some` را تبدیل می‌کند؛ `None` بدون تغییر عبور می‌کند.

مثال بک‌اند: lookup کاربر ممکن است هیچ نتیجه‌ای نداشته باشد و آن را به HTTP 404 نگاشت کنیم. اما اگر دانستن علت شکست مهم است—مثلاً database قطع است—`Result` لازم داریم، چون `None` دلیل حمل نمی‌کند.

تشبیه `Option` به nullable field محدود است: `Option<T>` یک enum صریح و pattern-matchable است و niche optimization گاهی بدون overhead اندازه‌ای اضافه پیاده می‌شود، اما این تضمین برای همه‌ی `T` یکسان نیست.

```senpai-visual
{"kind":"result","labels":["find user","Some(User)","None"]}
```
