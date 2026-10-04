# راه‌حل — ۳.۷.۴ چرخشِ refresh token و باطل‌سازیش

## `hash_token`

```rust
use sha2::{Digest, Sha256};

pub fn hash_token(raw: &str) -> String {
    format!("{:x}", Sha256::digest(raw.as_bytes()))
}
```

`Sha256::digest` سی‌ودو بایتِ خام را برمی‌گرداند. فرمتِ `{:x}` آن آرایه را hex با حروفِ کوچک چاپ می‌کند، دو کاراکتر به ازایِ هر بایت، پس در کل ۶۴ تا. صفتِ `Digest` باید در دسترس باشد تا `digest` صدا زده شود، و برای همین اسکلت از تو می‌خواهد `use` را اضافه کنی.

## `rotate`

```rust
let hash = hash_token(presented);
let record = self.records.get(&hash).cloned().ok_or(RefreshError::Unknown)?;
if record.used {
    self.revoke_family(&record.family_id);
    return Err(RefreshError::ReuseDetected);
}
if record.expires_at <= self.clock.now() {
    return Err(RefreshError::Expired);
}
if let Some(r) = self.records.get_mut(&hash) {
    r.used = true;
}
Ok(self.issue(&record.user_id, &record.family_id))
```

ترتیب خودش مشخصات است: ناشناخته، بعد مصرف‌شده، بعد منقضی. رکورد از نقشه کلون می‌شود تا فراخوانی‌هایِ بعدیِ `&mut self` (`revoke_family` و `issue`) با یک قرضِ زنده‌ی `self.records` درگیر نشوند. `expires_at <= now` باعث می‌شود توکن دقیقاً در ثانیه‌ی انقضایش منقضی شود. توکنِ تازه در همان خانواده می‌رود، و همین است که اجازه می‌دهد یک تکرار کلِ زنجیره را باطل کند.

## `logout`

```rust
let Some(record) = self.records.get(&hash_token(presented)).cloned() else {
    return false;
};
self.revoke_family(&record.family_id);
true
```

بدونِ بررسیِ مصرف‌شده یا انقضا: کاربری که با توکنِ قدیمی یا منقضی خارج می‌شود هم یعنی «این نشست را تمام کن».

## `logout_all`

```rust
let mut families: Vec<String> = self
    .records
    .values()
    .filter(|r| r.user_id == user_id)
    .map(|r| r.family_id.clone())
    .collect();
families.sort();
families.dedup();
for family in &families {
    self.revoke_family(family);
}
families.len()
```

یک خانواده به ازایِ هر چرخش یک رکورد دارد، پس شناسه‌هایِ خانواده تکرار می‌شوند؛ مرتب‌کردن و `dedup` هر خانواده را یک بار می‌شمارد. اول جمع‌کردنِ شناسه‌ها قرضِ `self.records` را تمام می‌کند، قبل از اینکه `revoke_family` به `&mut self` نیاز پیدا کند.

## چالش

یک `used_at: Option<u64>` به `Record` اضافه کن. در `rotate`، وقتی رکورد مصرف‌شده است و `now - used_at <= GRACE_SECONDS`، بدونِ صدا زدنِ `revoke_family` مقدارِ `Err(RefreshError::Unknown)` را برگردان؛ بیرونِ پنجره مثلِ قبل باطل کن. برگرداندنِ رد، نه همان جفتِ تازه، قاعده‌ی «یک توکن حداکثر یک جفتِ تازه می‌دهد» را حفظ می‌کند. با `ManualClock` در ۹ و ۱۰ و ۱۱ ثانیه بعد از چرخش تست کن.
