# ۰۸.۴ — `TryFrom` و تبدیل‌های شکست‌پذیر

## کوچک‌کردن عدد؛ مشکلی که پایتون پنهان می‌کند

integer پایتون اندازه‌ی دلخواه دارد، اما `u64` و `u8` Rust بازه‌ی ثابت دارند. `300u64 as u8` truncate می‌شود و `44` می‌دهد، چون مقدار modulo 256 نگه داشته می‌شود. `u8::try_from(300u64)` به‌جای از‌دست‌دادن داده، `Err(TryFromIntError)` می‌دهد.

قاعده: `From` برای تبدیل همیشه‌موفق و بدون از‌دست‌دادن معنا؛ `TryFrom` برای validation یا امکان شکست. `From`ای که گاهی panic کند به type system دروغ می‌گوید، چون caller حق دارد آن را infallible بداند.

## الگوی newtype معتبر: «parse کن، صرفاً validate نکن»

```rust
pub struct Percentage(u8);
```

field خصوصی است و تنها سازنده‌ی عمومی `TryFrom<u8>` بازه‌ی `0..=100` را می‌پذیرد. پس هر تابعی که `Percentage` می‌گیرد می‌داند invariant از قبل برقرار است و لازم نیست دوباره bool بررسی کند. `TryInto` به‌طور blanket از `TryFrom` متناظر فراهم می‌شود.

```senpai-visual
{"kind":"result","labels":["ورودی خام","TryFrom","Percentage معتبر","ValidationError"]}
```

برای `EmailAddress` هم گرفتن `String` با value اجازه می‌دهد در خطا همان ورودی ردشده را به caller پس بدهی و در موفقیت مالکیتش را بدون clone داخل newtype بگذاری. در Django form یا Pydantic validation مشابهی می‌بینی، اما Rust پس از ساخت نوع معتبر اجازه نمی‌دهد downstream مقدار نامعتبر را دور بزند.

تشبیه کارت مترو معتبر مفید است: پس از صدور، گیت لازم نیست هر بار فرایند ثبت‌نام را تکرار کند. مرز تشبیه: invariant فقط به اندازه‌ی constructorهای public و privacy قوی است؛ publicکردن field راه دورزدن را باز می‌کند.

## تمرین تو

`Percentage`، `EmailAddress`، errorها و conversionهای `TryFrom` را پیاده کن.
