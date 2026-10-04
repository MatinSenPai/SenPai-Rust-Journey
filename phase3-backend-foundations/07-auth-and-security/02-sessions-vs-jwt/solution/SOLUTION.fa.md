# راه‌حل — ۳.۷.۲ Session در برابرِ JWT: مصالحه‌یِ واقعی

کدِ کامل `solution/src/lib.rs` است؛ همه‌ی تست‌هایِ `solution/tests/` را پاس می‌کند (۴۵ تست برایِ پله‌ی «پیاده‌سازی»، به‌علاوه‌یِ `build_test.rs` برایِ «بساز» و «چالش»). `solution/src/main.rs` اپ را رویِ `127.0.0.1:3180` سرو می‌کند.

## `SessionStore`

```rust
pub fn create(&self, user: &str) -> String {
    let id = new_session_id();
    let expires_at = self.clock.now().saturating_add(self.ttl_secs);
    let session = Session { user: user.to_string(), expires_at };
    self.sessions.lock().unwrap().insert(id.clone(), session);
    id
}
```

انقضا یک بار، هنگامِ ساخت، از ساعتِ تزریق‌شده حساب می‌شود. `saturating_add` یعنی عمرِ خیلی بزرگ نمی‌تواند سرریز کند. هر فراخوانی یک شناسه‌یِ نو می‌سازد، پس یک کاربر با دو دستگاه دو ردیف دارد.

```rust
pub fn lookup(&self, id: &str) -> Option<String> {
    let now = self.clock.now();
    let mut sessions = self.sessions.lock().unwrap();
    match sessions.get(id) {
        Some(s) if now < s.expires_at => Some(s.user.clone()),
        Some(_) => { sessions.remove(id); None }
        None => None,
    }
}
```

گارد عمداً در تمامِ `match` نگه داشته می‌شود: بررسی و حذف باید زیرِ یک قفل باشند، وگرنه دو جست‌وجو می‌توانستند با هم مسابقه بدهند. `now < expires_at` مرز را دقیق می‌کند (`a_session_is_live_until_the_instant_it_expires`). حذفِ ورودیِ منقضی در همین‌جا تنها پاک‌سازیِ درس است.

`revoke` اول ورودی را برمی‌دارد و بعد `matches!(removed, Some(s) if now < s.expires_at)` را گزارش می‌دهد، پس یک نشستِ منقضی پاک می‌شود ولی `false` جواب می‌دهد. `revoke_all_for` همین را در `retain` می‌کند و نشست‌هایِ زنده‌ای را که حذف می‌کند می‌شمارد.

## `set_cookie` و همراهانش

```rust
let mut out = format!("{name}={value}; Path=/; HttpOnly");
if opts.secure { out.push_str("; Secure"); }
out.push_str(match opts.same_site { /* Strict, Lax, None */ });
if let Some(n) = opts.max_age { out.push_str(&format!("; Max-Age={n}")); }
```

اعتبارسنجی اول می‌آید: `bad_char` هر چیزِ بیرونِ `!`..=`~` به‌علاوه‌یِ `; , " \` را رد می‌کند، که همان چیزی است که جلویِ مقداری مثلِ `a; Domain=evil.example` یا یک CRLF را می‌گیرد که یک ویژگی یا هدرِ دوم را قاچاق کند. فقط بعد از آن قاعده‌یِ «`SameSite=None` به `Secure` نیاز دارد» اجرا می‌شود، و به همین دلیل `invalid_input_is_reported_before_the_samesite_rule`. `Some(0)` نوشته می‌شود چون `if let Some(n)` صفر را «نبود» حساب نمی‌کند. `expire_cookie` یک کپی از `opts` با `max_age: Some(0)` می‌سازد و `set_cookie` را با مقدارِ خالی صدا می‌زند، پس همه‌ی قاعده‌ها را به ارث می‌برد.

`session_id_from_cookie_header` رویِ `;` می‌شکند، trim می‌کند، `split_once('=')` می‌زند (فقط اولین `=`، پس `sid=a=b` مقدارِ `a=b` را نگه می‌دارد)، اولین تطابقِ نامِ دقیق را برمی‌دارد، و مقدارِ خالی را کنار می‌گذارد. `origin_ok` همان `None => true, Some(o) => o == expected` است: برابریِ دقیقِ رشته، پس اسلشِ آخر و `"null"` هر دو رد می‌شوند.

## بساز: `touch`

`touch` ورودی را با `get_mut` پیدا می‌کند، و فقط وقتی `now < expires_at` است `expires_at = now + ttl` می‌گذارد. ورودیِ منقضی دست‌نخورده می‌ماند و جواب `false` است: نشستی که مرده نباید برگردد. برایِ سیم‌کشی، `touch` را در `me` بعد از یک `lookup`ِ موفق صدا بزن و کوکی را با `set_cookie(.., Some(ttl))` دوباره بفرست تا `Max-Age`ِ مرورگر با انقضایِ سرور حرکت کند. بدونِ `Set-Cookie`ِ نو مرورگر کوکی را در زمانِ اصلی دور می‌ریخت، با اینکه سرور هنوز قبولش می‌کرد.

## چالش: `revoke_others`

باز `retain`، با نگه‌داشتنِ ورودی وقتی به کاربری دیگر تعلق دارد یا شناسه‌اش با `keep` برابر است، و شمارشِ نشست‌هایِ زنده‌ای که حذف می‌کند. مسیر نشستِ فراخوان را می‌خواند، کاربر را پیدا می‌کند، `revoke_others(user, id)` را صدا می‌زند و `204` جواب می‌دهد. باید مثلِ مسیرهایِ تغییردهنده‌یِ دیگر `origin_allowed` را اعمال کند، وگرنه یک `POST`ِ بین‌سایتیِ جعلی می‌توانست از همه‌جایِ دیگر خارجت کند.

## درباره‌یِ گرم‌کردن‌ها و خطاها

- **مثالِ ۰۴ (`E0004`).** `SameSite::None => "SameSite=None"` را اضافه کن. از `_ =>` یا `todo!()`ِ پیشنهادی استفاده نکن.
- **مثالِ ۰۵ (`E0277`).** `Cell<u64>` را با `AtomicU64` عوض کن (`get` می‌شود `load(Ordering::SeqCst)` و `set` می‌شود `store`).
- **مثالِ ۰۳ (تثبیت).** `login` باید یک شناسه‌یِ نو بسازد، شناسه‌یِ ورودی را از جدول حذف کند، شناسه‌یِ نو را واردشده درج کند و آن را برگرداند. تستی که مهم است: شناسه‌یِ کاشته‌شده بعدش دیگر وجود ندارد.

## این درس واقعاً درباره‌یِ چه بود

هیچ‌کدام از کدها بلند نیست. آنچه مهم است این است که هر قطعه چه *کاری* می‌تواند بکند: یک جدولِ سمتِ سرور می‌تواند با پاک‌کردنِ یک ردیف یک لاگین را تمام کند، و یک توکنِ بی‌حالت نمی‌تواند. بقیه (پرچم‌هایِ کوکی، بررسیِ `Origin`، شناسه‌یِ تازه در لاگین) هزینه‌یِ اجرایِ امنِ طراحیِ حالت‌دار است، و کوچک است.
