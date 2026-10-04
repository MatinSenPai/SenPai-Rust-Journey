# ۳.۷.۱ — هش‌کردنِ پسورد با argon2

## در یک نگاه

بعد از این درس می‌توانی:

- توضیح بدهی چرا پسورد هش می‌شود و رمزنگاری نمی‌شود، نمک چه چیزی اضافه می‌کند، و چرا `Argon2id` برای همین یک کار از `SHA-256` و `bcrypt` بهتر است.
- یک هشِ کدشده‌یِ Argon2 (رشته‌یِ PHC) را فیلد‌به‌فیلد بخوانی و تشخیص بدهی هشِ ذخیره‌شده ضعیف‌تر از تنظیماتِ فعلی‌ات هست یا نه.
- یک بررسیِ ورود بنویسی که «کاربرِ ناشناس» و «پسوردِ غلط» را یکسان و تقریباً در همان زمان جواب می‌دهد و رازها را بدونِ خروجِ زودهنگام مقایسه می‌کند.

**زمان:** حدود ۷۵ دقیقه · **پیش‌نیازها:**
[۲.۱.۲ — `HashMap` از نزدیک](../../../phase2-intermediate/01-collections/02-hashmap-in-depth/README.fa.md)،
[۳.۴.۱ — پیکربندیِ ۱۲فاکتوری و سکرت‌ها](../../04-configuration-and-app-structure/01-config-and-secrets/README.fa.md)

---

## چرا اهمیت دارد

دیر یا زود هر دیتابیسی نشت می‌کند: یک بکاپِ فراموش‌شده، یک SQL injection، یک ادمینِ اخراج‌شده. سؤال این است که مهاجم بعد از آن چه در دست دارد. اگر جدولِ `users` خودِ پسوردها را نگه دارد، مهاجم پسوردِ واقعیِ همه‌ی کاربران را دارد، پسوردی که قابلِ استفاده‌ی دوباره است، برایِ اپِ تو و برایِ هر سایتِ دیگری که کاربر همان را در آن تکرار کرده. برای همین OWASP Top 10 دسته‌ای برایش دارد: «شکستِ رمزنگاری» (Cryptographic Failures؛ در نسخه‌ی ۲۰۲۱ شماره‌ی A02)، که پسوردِ ذخیره‌شده به‌صورتِ آشکار یا با هشِ سریع و بدونِ نمک را هم شامل می‌شود.

نسخه‌ی امنش را بی‌آنکه بنویسی استفاده کرده‌ای. در Django، `make_password("hunter2")` رشته‌ای مثلِ `argon2$argon2id$v=19$...` برمی‌گرداند، `User.objects.create_user(...)` فقط همان رشته را ذخیره می‌کند و `check_password("hunter2", encoded)` راهِ بررسیِ ورود است. Django وقتی هشِ ذخیره‌شده با الگوریتمِ قدیمی یا تنظیماتِ قدیمی ساخته شده باشد، بعد از ورودِ موفق آن را دوباره هش هم می‌کند. این درس همین قطعه‌ها را دستی می‌سازد، در یک `UserStore` که کاربران را در حافظه نگه می‌دارد. هیچ‌چیزِ اینجا دیتابیس یا سرور نمی‌خواهد، پس هر تست در چند میلی‌ثانیه تمام می‌شود. وقتی به ماژولِ PostgreSQL برسی، [ماژولِ ۵](../../05-postgres-and-sqlx/README.fa.md)، جدولِ `users` یک ستونِ `TEXT` خواهد داشت با همین رشته‌ای که امروز می‌سازی.

---

## مفهوم

### هش کن، رمزنگاری نکن

هرگز پسوردِ اصلی را دوباره لازم نداری. ورود فقط می‌پرسد «آنچه تایپ شد با آنچه ذخیره شده می‌خواند؟» رمزنگاری برایِ داده‌ای است که باید دوباره بخوانی، مثلِ شماره‌ی کارتی که قرار است از آن پول بگیری، و کلید می‌خواهد. آن کلید یک چیزِ دیگر است که می‌شود دزدید، و هر که آن را بدزدد همه‌ی پسوردها را یکجا می‌خواند. **هشِ یک‌طرفه** (one-way hash) نه کلید دارد نه راهِ برگشت. مهاجم فقط می‌تواند پسوردی حدس بزند، حدسش را هش کند و مقایسه کند.

باقیِ درس این است که هر حدس را تا جای ممکن گران کنیم.

```senpai-visual
{"kind":"concept","labels":["ثبت‌نام: پسورد فقط یک بار وارد می‌شود","نمک + Argon2id: هشِ کند و پرمصرفِ حافظه","فقط رشته‌یِ PHC ذخیره می‌شود","ورود: پسوردِ تایپ‌شده با نمکِ ذخیره‌شده دوباره هش و مقایسه می‌شود"]}
```

### جدولِ رنگین‌کمانی و نمک

هشِ ساده قطعی است: `sha256("hunter2")` روی هر ماشین و تا همیشه یکی است. پس مهاجم می‌تواند یک بار جدولی از `هش -> پسورد` برایِ میلیون‌ها پسوردِ رایج از پیش بسازد (**جدولِ رنگین‌کمانی** یا rainbow table، به معنایِ وسیع: هر جدولِ جست‌وجویِ از پیش‌محاسبه‌شده) و علیه هر دیتابیسِ نشت‌کرده دوباره استفاده‌اش کند. نشتِ تو برایِ او یک جست‌وجو در جدول است، نه یک کارِ شکستن.

**نمک** (salt) داده‌یِ تصادفی است، تازه برایِ هر پسورد، که در هش مخلوط می‌شود. `examples/01-hash-and-verify.rs` یک پسورد را دو بار هش می‌کند:

```text
first : $argon2id$v=19$m=19456,t=2,p=1$StrpfuYFfUCnPJmCaN1u8g$xl7Dv7aJqja+rp26ugAItu08nxRhC/P1ZS80YH3Fl7M
second: $argon2id$v=19$m=19456,t=2,p=1$fuye9QoksAjIGHYCjr9Fkg$rdrENzbLC7spjBvpGHyYIlUDyKpwAn5WyoBkusFXggk
equal strings? false
right password: Ok(())
wrong password: Err(Password)
```

(نمک‌ها و هش‌هایِ تو فرق می‌کنند؛ همین منظور است.) یک ورودی دو رشته‌یِ متفاوت می‌دهد و هر دو درست verify می‌شوند. دو کاربر با پسوردِ یکسان ردیف‌هایِ متفاوت می‌گیرند، پس جدولِ از پیش‌ساخته بی‌فایده است: مهاجم برایِ هر نمک جدولِ تازه می‌خواهد، که هزینه‌اش با شکستنِ مستقیمِ هر ردیف برابر است. نمک راز نیست. داخلِ رشته‌یِ ذخیره‌شده است و تنها کارش یکتا و غیرقابلِ پیش‌بینی بودن است.

### چرا Argon2id، نه `SHA-256` یا `bcrypt`

- **`SHA-256`** برایِ سرعت ساخته شده؛ برایِ checksum درست است و برایِ پسورد غلط. یک GPU در ثانیه میلیاردها حدس را روی هشِ سریع امتحان می‌کند.
- **`bcrypt`** عمداً کند است و هزینه‌اش تنظیم‌شدنی؛ در زمانِ خودش گامِ بزرگی بود. اما برایِ هر حدس فقط حافظه‌یِ کمِ ثابتی می‌خواهد، پس سخت‌افزارِ شکستن (GPU، FPGA، ASIC) می‌تواند حدس‌هایِ بسیار زیادی را ارزان و موازی اجرا کند.
- **`Argon2id`** **حافظه‌سخت** (memory-hard) است: هر حدس تا وقتی اجرا می‌شود باید مقدارِ تنظیم‌شدنی‌ای RAM نگه دارد. RAM همان منبعی است که ارزان موازی نمی‌شود؛ هر حدسِ موازیِ مهاجم بلوکِ خودش را می‌خواهد. Argon2 در سالِ ۲۰۱۵ برنده‌یِ Password Hashing Competition شد و نوعِ `Argon2id` همانی است که Password Storage Cheat Sheet‌ِ OWASP اول از همه پیشنهاد می‌کند. این درس عددهای آن برگه را نقل نمی‌کند چون بازبینی می‌شوند؛ برایِ عددهای فعلی خودش را بخوان.

پیش‌فرضِ Django هنوز PBKDF2 است، اما `PASSWORD_HASHERS` مقدارِ `Argon2PasswordHasher` را می‌پذیرد (با `pip install django[argon2]`) و مستنداتِ Django آن را اول توصیه می‌کند. جایی که قیاس تمام می‌شود: Django الگوریتم و پارامترها را از تنظیمات برایِ تو انتخاب می‌کند. در Rust خودت می‌دهی، و برای همین `hash_password` در این درس هزینه را به‌صورتِ آرگومان می‌گیرد.

```senpai-visual
{"kind":"concept","labels":["SHA-256: سریع، پس میلیاردها حدس در ثانیه","bcrypt: کند، اما حافظه‌یِ کم برایِ هر حدس","Argon2id: کند و حافظه‌سخت","هزینه‌یِ هر حدس: کم، متوسط، زیاد"]}
```

### خواندنِ یک رشته‌یِ PHC

رشته‌ای که `hash_password` برمی‌گرداند **قالبِ رشته‌یِ PHC** است. `examples/02-read-the-phc-string.rs` یکی را از هم باز می‌کند:

```text
algorithm  : argon2id
version    : Some(19)
memory KiB : 19456
iterations : 2
lanes      : 1
salt       : c29tZXNhbHQ
hash bytes : 24
```

ورودی `$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$RdescudvJCsgt3ub+b+dWRWJTmaaJObG` بود، پنج فیلدِ جداشده با `$`:

- `argon2id` نوع است و `v=19` نسخه‌یِ Argon2 (`0x13`).
- `m=19456,t=2,p=1` هزینه است: حافظه به KiB، تعدادِ گذر روی حافظه، و تعدادِ lane‌ها (موازی‌سازی). پیش‌فرضِ خودِ کریتِ `argon2` نسخه‌یِ ۰٫۵ دقیقاً همین سه عدد است.
- نمک و خودِ هش، هر دو base64 بدونِ padding. `c29tZXNhbHQ` به بایت‌هایِ `somesalt` دیکد می‌شود.

هر چه برایِ verify لازم است داخلِ رشته است، پس یک ستونِ `TEXT` کافی است. یعنی سالِ بعد می‌توانی هزینه را بالا ببری: هش‌هایِ قدیمی با تنظیماتِ نوشته‌شده در خودشان verify می‌شوند و هش‌هایِ تازه از تنظیماتِ جدید استفاده می‌کنند. `needs_rehash` در این درس همین سؤال است: «این هشِ ذخیره‌شده ضعیف‌تر از چیزی است که امروز می‌ساختم؟» و لحظه‌یِ ورودِ موفق وقتِ جواب دادن به آن است، چون فقط آن موقع پسوردِ خام دستِ توست.

### هزینه همان هدف است، و تست‌ها نباید آن را بپردازند

`examples/03-cost-and-parameters.rs` یک پسورد را با سه تنظیم هش می‌کند:

```text
m=8      t=1  took    1 ms  $argon2id$v=19$m=8,t=1,p=1$IoUnv
m=19456  t=2  took  439 ms  $argon2id$v=19$m=19456,t=2,p=1$l
m=65536  t=3  took 2238 ms  $argon2id$v=19$m=65536,t=3,p=1$h
cheap hash, default hasher: Ok(())
```

این زمان‌ها از build در حالتِ debug روی یک لپ‌تاپ‌اند و روی ماشینِ تو فرق می‌کنند، ولی الگو ثابت است: هزینه با حافظه و تعدادِ گذر بالا می‌رود، و هشی که با `m=8` ساخته شده هنوز از طریقِ hasher‌ای با پیش‌فرض‌هایِ دیگر verify می‌شود، چون پارامترهایش را از رشته می‌خواند.

این کندی هنگامِ ورود یک ویژگی است و در مجموعه‌یِ تست یک مزاحم. برای همین هر تابعِ این درس هزینه را به‌صورتِ `Params` می‌گیرد و هر تست `cheap_params()` می‌دهد: ۸ KiB حافظه، یک گذر، یک lane. این کوچک‌ترین حافظه‌ای است که `Argon2` برایِ یک lane می‌پذیرد (۸ برابرِ تعدادِ lane‌ها). ۱۸ تست در حدودِ یک‌دهم ثانیه تمام می‌شوند. هرگز این تنظیمات را برایِ کاربرانِ واقعی نگذار؛ فقط برایِ سریع ماندنِ تست‌ها هستند.

### یک جواب برایِ «چنین کاربری نیست» و «پسورد غلط است»

اگر ورود برایِ یکی بگوید «چنین کاربری نیست» و برایِ دیگری «پسورد غلط است»، مهاجم می‌تواند بی‌آنکه حتی یک پسورد بداند بفهمد کدام نام‌های کاربری در سایتِ تو وجود دارند. برایِ هر دو یک خطا برگردان. اما همین کافی نیست. `examples/04-unknown-user-timing.rs` سه مسیر را زمان می‌گیرد:

```text
known user, wrong password :  432 ms
unknown user, early return :    0 ms
unknown user, dummy verify :  437 ms
```

پاسخی که برایِ کاربرِ ناشناس فوری برمی‌گردد و برایِ کاربرِ شناخته‌شده بعد از ۴۰۰ میلی‌ثانیه، همان را به مهاجم می‌گوید، فقط از راهِ ساعت. راهِ حل این است که روی هر دو مسیر یک‌قدر کار خرج کنی. برایِ کاربرِ ناشناس، پسوردِ تایپ‌شده را با یک **هشِ ساختگی** (dummy hash) که یک بار هنگامِ ساختنِ store ساخته شده verify کن و نتیجه را دور بینداز. Django هم معادلش را می‌کند: وقتی `ModelBackend` کاربری پیدا نکند، پیش از برگشتن hasher را روی ورودی اجرا می‌کند.

```senpai-visual
{"kind":"result","labels":["login(username, password)","کاربر پیدا شد: با هشِ خودش verify","کاربر پیدا نشد: با هشِ ساختگی verify","هر دو مسیر: Err(InvalidCredentials)، زمانِ مشابه"]}
```

### مقایسه‌یِ رازها بدونِ خروجِ زودهنگام

`==` معمولی روی دو برشِ بایت در اولین تفاوت می‌ایستد. اگر بایتِ اول فرق کند فوری برمی‌گردد و اگر هزار بایتِ اول یکی باشد بیشتر طول می‌کشد. با اندازه‌گیریِ دقیقِ کافی، زمان فاش می‌کند حدس تا کجا درست بوده. **مقایسه‌یِ زمانِ‌ثابت** (constant-time comparison) هر بایت را می‌بیند، هر داده‌ای که باشد: هر جفت را XOR کن، نتیجه‌ها را با OR جمع کن و فقط آخرِ کار یک بار بسنج.

برایِ پسورد لازم نیست خودت صدایش کنی: کریتِ `password-hash` که `argon2` رویش ساخته شده خروجی‌هایِ هش را داخلِ `verify_password` در زمانِ ثابت مقایسه می‌کند (`ConstantTimeEq`ِ کریتِ `subtle` را پیاده‌سازی می‌کند). با این حال `constant_time_eq` را می‌نویسی، چون همین ایده برایِ هر رازی که خودت مقایسه می‌کنی مهم است، مثلِ کلیدِ API یا توکن. در کدِ واقعی به‌جایِ حلقه‌یِ خودت سراغِ کریتِ `subtle` می‌روی.

---

## دست‌به‌کد

```sh
cargo run -p p3-07-01-password-hashing-argon2 --example 01-hash-and-verify
cargo run -p p3-07-01-password-hashing-argon2 --example 02-read-the-phc-string
cargo run -p p3-07-01-password-hashing-argon2 --example 03-cost-and-parameters
cargo run -p p3-07-01-password-hashing-argon2 --example 04-unknown-user-timing
```

بعد سه مثالِ خراب. دوتایشان feature‌یِ `broken` می‌خواهند؛ سومی عادی اجرا می‌شود و فقط غلط است:

```sh
cargo build -p p3-07-01-password-hashing-argon2 --example 05-missing-trait-import-broken --features broken
cargo run -p p3-07-01-password-hashing-argon2 --example 06-unwrap-malformed-hash-broken --features broken
cargo run -p p3-07-01-password-hashing-argon2 --example 07-compare-hashes-trap
```

پنج تابع و store در `src/lib.rs` با `todo!()` شروع می‌شوند. پیش از نوشتنِ هر چیز تست‌ها را اجرا کن و ببین به دلیلِ درست شکست می‌خورند:

```sh
cargo test -p p3-07-01-password-hashing-argon2 --test hashing hash_then_verify
```

```text
running 1 test
test hash_then_verify_round_trips ... FAILED

failures:

---- hash_then_verify_round_trips stdout ----

thread 'hash_then_verify_round_trips' (29316) panicked at phase3-backend-foundations\07-auth-and-security\01-password-hashing-argon2\src\lib.rs:22:5:
not yet implemented: return the encoded Argon2id PHC string for this password, using a fresh random salt and the given cost settings
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    hash_then_verify_round_trips

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 11 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p p3-07-01-password-hashing-argon2 --test hashing`
```

(عددِ داخلِ پرانتز شناسه‌یِ ریسمان است و هر اجرا فرق می‌کند. پیش از `running 1 test` کامپایلر برایِ بدنه‌یِ هر `todo!()` هشدارِ «unused variable» هم چاپ می‌کند؛ با پیاده‌سازی از بین می‌روند.)

بعد این‌ها را امتحان کن:

۱. در `01-hash-and-verify` به‌جایِ پیش‌فرض با `Params::new(8, 1, 1, None)` هش کن. ابتدایِ رشته حالا چه می‌گوید؟
۲. در `02-read-the-phc-string` یک نویسه‌یِ نمک را عوض کن. آیا `PasswordHash::new` هنوز می‌پذیردش؟ آیا هش هنوز به آن نمک «تعلق» دارد؟
۳. در `04-unknown-user-timing` کاری کن که verifyِ ساختگی با `m=8` باشد. آیا دو مسیر هنوز زمانِ یکسان دارند؟ مهاجم چه می‌بیند؟

---

## خطاهایی که خواهی دید

### `E0599` — متدِ یک trait که در scope نیست

```text
error[E0599]: no method named `hash_password` found for struct `Argon2<'key>` in the current scope
   --> phase3-backend-foundations\07-auth-and-security\01-password-hashing-argon2\examples\05-missing-trait-import-broken.rs:10:34
    |
 10 |     let hash = Argon2::default().hash_password(b"hunter2", &salt).unwrap();
    |                                  ^^^^^^^^^^^^^
    |
   ::: C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\password-hash-0.5.0\src\traits.rs:33:8
    |
 33 |     fn hash_password<'a>(
    |        ------------- the method is available for `Argon2<'_>` here
    |
    = help: items from traits can only be used if the trait is in scope
help: there is a method `hash_password_into` with a similar name, but with different arguments
   --> C:\Users\khmja\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\argon2-0.5.3\src\lib.rs:229:5
    |
229 |     pub fn hash_password_into(&self, pwd: &[u8], salt: &[u8], out: &mut [u8]) -> Result<()> {
    |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
help: trait `PasswordHasher` which provides `hash_password` is implemented but not in scope; perhaps you want to import it
    |
  5 + use argon2::PasswordHasher;
    |

For more information about this error, try `rustc --explain E0599`.
error: could not compile `p3-07-01-password-hashing-argon2` (example "05-missing-trait-import-broken") due to 1 previous error
```

(هشدارهایِ `todo!()` از `src/lib.rs` پیش از این چاپ می‌شوند و اینجا حذف شده‌اند. مسیرهایِ `.cargo\registry` و پسوندهایِ هش به ماشینِ تو بستگی دارند.)

**کامپایلر به چه ایراد می‌گیرد:** `hash_password` متدِ خودِ `Argon2` نیست؛ از trait‌ی `PasswordHasher` می‌آید. متدهایِ یک trait فقط وقتی روی یک نوع وجود دارند که trait در scope باشد. خطا همین را صریح می‌گوید (`items from traits can only be used if the trait is in scope`). اولین `help:` گمراه‌کننده است: `hash_password_into` متدِ دیگری است، سطح‌پایین‌تر و با امضایِ متفاوت.

**راهِ حل:** `help:`ِ دوم را دنبال کن و trait را import کن.

```rust
use argon2::password_hash::{rand_core::OsRng, PasswordHasher, SaltString};
```

**چرا این راهِ حل است:** `PasswordHasher` و `PasswordVerifier` دو traitی هستند که برایِ هش و verify لازم داری و هیچ‌کدام در prelude نیست. هر دو را اضافه کن تا متدهایِ کریت ظاهر شوند.

### یک panicِ زمانِ اجرا: unwrap روی هشِ ذخیره‌شده‌ای که parse نمی‌شود

```text
thread 'main' (24420) panicked at phase3-backend-foundations\07-auth-and-security\01-password-hashing-argon2\examples\06-unwrap-malformed-hash-broken.rs:8:51:
called `Result::unwrap()` on an `Err` value: PhcStringField
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

(شناسه‌یِ ریسمان هر اجرا فرق می‌کند.)

**واقعاً چه چیزی خراب است:** `PasswordHash::new(from_database).unwrap()`. مقدارِ ذخیره‌شده از ستونی آمده که کسی بد migrate کرده، بریده شده یا هشِ قدیمیِ سیستمی دیگر در آن ریخته‌اند. این داده از بیرونِ تابعِ توست و گذاشتی درخواست را از کار بیندازد. حالا یک ردیفِ خراب کلِ مسیرِ ورود را می‌خواباند، و مهاجمی که بتواند آن مقدار را تحت‌تأثیر قرار دهد، کرش را در دست دارد.

**راهِ حل:** تصمیم بگیر هشِ ذخیره‌شده‌یِ خراب یعنی چه. برایِ ورود یعنی «رد کن»:

```rust
let Ok(parsed) = PasswordHash::new(from_database) else {
    println!("stored hash is not a PHC string: reject the login");
    return;
};
```

**چرا این راهِ حل است:** `let ... else` خطا را به مقداری تبدیل می‌کند که خودت انتخاب کرده‌ای. برایِ همین `verify_password` تو `bool` برمی‌گرداند و هرگز panic نمی‌کند: تصمیمِ caller «بپذیر» یا «رد کن» است، و ردیفِ خراب و پسوردِ غلط هر دو یعنی «رد کن».

### بدونِ هیچ خطایی: ورود با هشِ دوباره

```text
correct password accepted? false
```

**واقعاً چه چیزی خراب است:** `examples/07-compare-hashes-trap.rs` پسوردِ تایپ‌شده را دوباره هش می‌کند و دو رشته را با `==` مقایسه می‌کند. کامپایل می‌شود و اجرا می‌شود، و هر بار پسوردِ درست را رد می‌کند. نمکِ تازه رشته‌یِ جدید را با رشته‌یِ ذخیره‌شده متفاوت می‌کند، پسورد هر قدر درست باشد. همین اشتباه در یک «اصلاحِ» عجولانه بدتر می‌شود: کسی می‌بیند ورودها شکست می‌خورند، نمکِ تصادفی را برمی‌دارد تا رشته‌ها بخوانند، و ضعفِ جدولِ رنگین‌کمانی را وارد تولید می‌کند.

**راهِ حل:** هرگز خودت هش‌ها را مقایسه نکن. رشته‌یِ ذخیره‌شده را parse کن و از کتابخانه بخواه با نمکی که داخلِ آن است دوباره حساب و مقایسه کند:

```rust
let parsed = PasswordHash::new(&stored).expect("stored hash");
let login_ok = Argon2::default().verify_password(b"hunter2", &parsed).is_ok();
```

**چرا این راهِ حل است:** نمک باید همانی باشد که پسورد اولین‌بار با آن هش شد، و برای همین داخلِ رشته‌یِ ذخیره‌شده است. `verify_password` آن را از آنجا می‌خواند، پارامترهایِ داخلِ رشته را به کار می‌برد و در زمانِ ثابت مقایسه می‌کند. هیچ کامپایلری این باگ را نمی‌گیرد؛ تستی که کاربری را ثبت‌نام می‌کند و بعد با همان کاربر وارد می‌شود می‌گیرد.

---

## تمرین

### گرم‌کردن

<details>
<summary>دو کاربر با پسوردِ یکسان ثبت‌نام می‌کنند. هش‌هایِ ذخیره‌شده‌شان برابرند؟</summary>

پیش از دیدنِ جواب فکر کن.

</details>

<details>
<summary>جواب</summary>

نه. هر هش نمکِ تصادفیِ خودش را دارد، پس رشته‌ها فرق می‌کنند. هش‌هایِ برابر به هر که بتواند جدول را بخواند می‌گفت دو حساب پسورد مشترک دارند.

</details>

<details>
<summary>هشی با <code>m=8,t=1,p=1</code> ساخته شده. تنظیماتِ فعلیِ سرورِ تو خیلی قوی‌تر است. <code>verify_password</code> هنوز پسوردِ درست را می‌پذیرد؟</summary>

پیش از دیدنِ جواب فکر کن.

</details>

<details>
<summary>جواب</summary>

بله. تنظیماتِ هزینه داخلِ رشته‌اند و verify از آن‌ها استفاده می‌کند، نه از تنظیماتِ فعلی. برای همین `needs_rehash` وجود دارد: هشِ قدیمی کار می‌کند، پس باید چیزی ضعیف بودنش را تشخیص بدهد.

</details>

<details>
<summary>ورود برایِ نامِ کاربریِ ناشناس در ۰ میلی‌ثانیه و برایِ نامِ شناخته‌شده در ۴۳۰ میلی‌ثانیه جواب می‌دهد، با متنِ خطای یکسان. چه چیزی فاش می‌شود؟</summary>

پیش از دیدنِ جواب فکر کن.

</details>

<details>
<summary>جواب</summary>

اینکه کدام نام‌هایِ کاربری وجود دارند. متنِ خطا یکی است ولی زمان یکی نیست، پس مهاجم به‌جایِ پیام ساعت را می‌خواند. راهِ حل verify با هشِ ساختگی است وقتی کاربر ناشناس است.

</details>

<details>
<summary>چرا مقایسه‌یِ دو راز با <code>==</code> اطلاعات فاش می‌کند، با اینکه جواب فقط «برابر» یا «نابرابر» است؟</summary>

پیش از دیدنِ جواب فکر کن.

</details>

<details>
<summary>جواب</summary>

`==` می‌تواند در اولین بایتِ متفاوت بایستد، پس زمانِ اجرا به تعدادِ بایت‌هایِ ابتدایی که یکی بوده بستگی دارد. با اندازه‌گیریِ بسیار، مهاجم می‌فهمد حدسش چقدر نزدیک است. مقایسه‌یِ زمانِ‌ثابت همیشه همه‌ی بایت‌ها را می‌بیند.

</details>

### تعمیر

هر سه مثالِ خراب را درست کن:

۱. `examples/05-missing-trait-import-broken.rs` build شود و یک رشته‌یِ PHC چاپ کند.
۲. `examples/06-unwrap-malformed-hash-broken.rs` به‌جایِ panic پیامِ رد را چاپ کند.
۳. `examples/07-compare-hashes-trap.rs` برایِ پسوردِ درست `true` چاپ کند.

### پیاده‌سازی

پنج تابع در `src/lib.rs`:

```sh
cargo test -p p3-07-01-password-hashing-argon2 --test hashing
```

هر کدام در کامنتِ مستنداتِ خودش کامل مشخص شده، پس برایِ فهمیدنِ آنچه باید بسازی لازم نیست تست‌ها را باز کنی:

- `hash_password(password, params)`: یک رشته‌یِ PHC از نوعِ Argon2id با نمکِ تازه‌یِ تصادفی در هر بار.
- `verify_password(password, phc)`: فقط برایِ تطابق `true`؛ و `false` (هرگز panic) برایِ پسوردِ غلط یا مقدارِ ذخیره‌شده‌ای که رشته‌یِ PHC نیست.
- `describe_hash(phc)`: رشته‌یِ PHC را به یک `HashInfo` بخوان، یا `None` وقتی هشِ Argon2ِ کامل و معتبر نیست.
- `needs_rehash(phc, target)`: آیا این هشِ ذخیره‌شده الگوریتمی متفاوت از تنظیماتِ هدف دارد یا ضعیف‌تر از آن است؟
- `constant_time_eq(a, b)`: برابریِ بایت‌ها که هرگز زود خارج نمی‌شود.

همه‌شان در تست‌ها `cheap_params()` استفاده می‌کنند، پس ۱۲ تستِ `tests/hashing.rs` در کسری از ثانیه تمام می‌شوند.

### بساز

`UserStore` در همان فایل: یک جدولِ کاربرِ درون‌حافظه که هرگز پسورد نگه نمی‌دارد.

```sh
cargo test -p p3-07-01-password-hashing-argon2 --test store
```

- `new(params)` یک store خالی و یک هشِ ساختگیِ آماده برایِ ورود می‌سازد.
- `register(username, password)` پسوردِ کوتاه (کمتر از ۸ نویسه، با شمارشِ `char` نه بایت) را پیش از بررسیِ نامِ کاربری با `PasswordTooShort` رد می‌کند، بعد نامِ تکراری را با `UsernameTaken`، و در غیر این صورت فقط هش را ذخیره می‌کند.
- `login(username, password)` فقط برایِ کاربرِ شناخته‌شده با پسوردِ درست `Ok(())` برمی‌گرداند. کاربرِ ناشناس و پسوردِ غلط هر دو `InvalidCredentials` می‌گیرند، و کاربرِ ناشناس باز هم هزینه‌یِ یک verifyِ کامل را با هشِ ساختگی می‌پردازد.

شش تستِ `tests/store.rs` این را بررسی می‌کنند، از جمله اینکه مقدارِ ذخیره‌شده با `$argon2id$` شروع می‌شود و هرگز پسورد را در خود ندارد.

### چالش (اختیاری)

کاری کن که `login`ِ موفق، وقتی `needs_rehash` می‌گوید هشِ ذخیره‌شده از پارامترهایِ فعلیِ store ضعیف‌تر است، آن را با پسوردِ خامی که همان لحظه داری ارتقا بدهد. ورودِ ناموفق نباید هیچ‌چیز را تغییر بدهد. تستِ خودت را به `tests/store.rs` اضافه کن: با `cheap_params()` ثبت‌نام کن، با `set_params` تنظیماتِ قوی‌تر بده، یک بار وارد شو و هزینه را با `describe_hash` بخوان.

---

## جمع‌بندی

| اصطلاح | یعنی چه | کجا به کار می‌آید |
|---|---|---|
| هشِ پسورد | چکیده‌ی یک‌طرفه و عمداً کندِ پسورد | تنها ستونی که جدولِ `users` نگه می‌دارد |
| نمک (salt) | داده‌یِ تصادفیِ یکتا برایِ هر هش، داخلِ رشته‌یِ هش | بی‌اثر کردنِ جدول‌هایِ از پیش‌ساخته |
| جدولِ رنگین‌کمانی | جست‌وجویِ از پیش‌محاسبه‌شده از هش به پسورد | چرا هشِ بدونِ نمک یا سریع شکست می‌خورد |
| حافظه‌سخت (memory-hard) | هر حدس باید تا وقتی اجرا می‌شود RAM واقعی نگه دارد | چرا `Argon2id` در برابرِ GPU و ASIC مقاوم است |
| رشته‌یِ PHC | `$alg$v=..$params$salt$hash`، هر چه برایِ verify لازم است | ذخیره و خواندنِ هش‌ها |
| پارامترهایِ هزینه | `m` (حافظه)، `t` (گذرها)، `p` (laneها) | تنظیم، و `needs_rehash` |
| هشِ ساختگی (dummy hash) | هشی دورانداختنی که وقتی کاربر وجود ندارد verify می‌شود | ورودی که در هر دو حالت زمانِ یکسان می‌برد |
| مقایسه‌یِ زمانِ‌ثابت | مقایسه بدونِ خروجِ زودهنگام | هر رازی که خودت مقایسه می‌کنی |

### الان می‌دانی

- پسورد هش می‌شود، هرگز رمزنگاری نمی‌شود و هرگز آشکار ذخیره نمی‌شود، و OWASP ذخیره‌یِ بدِ آن را زیرِ «شکستِ رمزنگاری» ثبت می‌کند.
- نمکِ تازه برایِ هر هش، پسوردهایِ برابر را متفاوت نشان می‌دهد و جدول‌هایِ از پیش‌ساخته را بی‌ارزش می‌کند.
- `Argon2id` حافظه‌سخت است، `bcrypt` فقط کند است و `SHA-256` سریع است، که اینجا ویژگیِ غلط است.
- رشته‌یِ PHC الگوریتم، نسخه، هزینه، نمک و هش را با خود دارد، پس هزینه بعداً بالا می‌رود و هش‌هایِ قدیمی هنوز verify می‌شوند.
- ورود باید برایِ کاربرِ ناشناس همان خطا و تقریباً همان زمانی را داشته باشد که برایِ پسوردِ غلط دارد.
- تست‌ها هزینه را به‌صورتِ `cheap_params()` می‌دهند تا اجرایِ واقعیِ Argon2 مجموعه‌یِ تست را کند نکند.

### بعداً کامل‌تر می‌بینی

- **جدولِ `users` و ستونِ `TEXT`ِ هش واقعاً کجا زندگی می‌کنند** — [۳.۵ — دیتابیس PostgreSQL و `sqlx`](../../05-postgres-and-sqlx/README.fa.md)
- **خواندنِ تنظیماتِ هزینه و رازها از پیکربندی** — [۳.۴.۱ — پیکربندیِ ۱۲فاکتوری و سکرت‌ها](../../04-configuration-and-app-structure/01-config-and-secrets/README.fa.md)
- **کاربرِ واردشده بعد از بررسیِ پسورد چه در دست دارد، stateful یا stateless** — [۳.۷.۲ — Session در برابرِ JWT: مصالحه‌یِ واقعی](../02-sessions-vs-jwt/README.fa.md)
- **صدور و بررسیِ توکن‌هایِ امضاشده در میان‌افزار** — [۳.۷.۳ — JWT و میان‌افزار در `tower`](../03-jwt-and-tower-middleware/README.fa.md)
- **تبدیلِ `InvalidCredentials` به یک خطایِ HTTPِ یکدست** — [۳.۸.۱ — پاکت‌هایِ خطایِ یکدست](../../08-error-handling-and-testing-at-scale/01-consistent-error-envelopes/README.fa.md)

### می‌توانی توضیح بدهی؟

- چرا پسورد هش می‌شود و رمزنگاری نمی‌شود، و مهاجم بعد از نشت در هر طراحی چه در دست دارد؟
- نمک چه چیزی را عوض می‌کند و چرا لازم نیست راز باشد؟
- چرا `SHA-256` هشِ غلط برایِ پسورد است، و `Argon2id` چه چیزی بیشتر از `bcrypt` دارد؟
- هر فیلدِ `$argon2id$v=19$m=19456,t=2,p=1$salt$hash` به verifier چه می‌گوید؟
- چرا ورود باید کارِ هشِ واقعی انجام بدهد حتی وقتی نامِ کاربری وجود ندارد؟
- چرا `==` روی رازها نشت می‌دهد و `constant_time_eq` چطور از آن دوری می‌کند؟

---

## بیشتر

- [OWASP Password Storage Cheat Sheet](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html): تنظیماتِ فعلیِ پیشنهادیِ Argon2id و دلیلشان.
- [OWASP Top 10 — Cryptographic Failures](https://owasp.org/Top10/A02_2021-Cryptographic_Failures/): دسته‌ای که حالتِ شکستِ این درس به آن تعلق دارد.
- [Password Hashing Competition](https://www.password-hashing.net/): جایی که Argon2 در ۲۰۱۵ برنده شد.
- [مستنداتِ کریتِ `argon2`](https://docs.rs/argon2/0.5.3/argon2/): `Argon2`، `Params` و نوع‌هایِ `password_hash` که اینجا استفاده شدند.
- [Django — Password management](https://docs.djangoproject.com/en/stable/topics/auth/passwords/): `make_password`، `check_password`، `PASSWORD_HASHERS` و اینکه Django هش‌هایِ قدیمی را چگونه ارتقا می‌دهد.
