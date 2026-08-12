# ایست بازرسی

۱. `self.get(id).await?` در update/delete چه دو کار را رایگان می‌کند و بدون آن چه کد تکراری لازم بود؟
۲. با اینکه crate فقط چهار string `as_str()` را می‌نویسد، چرا `WatchStatus::parse` باید `Result` بدهد؟ چه نویسنده‌ی دیگری ممکن است column را خراب کند؟
۳. چرا `BIGSERIAL` با `u64` مستقیم جور نیست و نگه‌داشتن u64 چه castهایی به همه queryها تحمیل می‌کرد؟
۴. چرا `ORDER BY id` database از sortکردن داده پس از fetch در application فراتر از کد کمتر است؟
۵. read-then-write update چگونه lost update می‌سازد و کدام patternهای `FOR UPDATE` یا conditional UPDATE آن را می‌بندند؟
۶. چرا tag serial باید در دو فایل test دقیقاً یک نام داشته باشد؟
