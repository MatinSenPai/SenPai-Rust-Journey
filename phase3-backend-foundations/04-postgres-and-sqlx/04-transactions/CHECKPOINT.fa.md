# ایست بازرسی

۱. پس از debit موفق و credit شکست‌خورده، rollback در کدام خط transfer رخ می‌دهد؟ trick: چرا خطی ندارد و چه زمانی رخ می‌دهد؟
۲. کدام دو سازوکار Rust/sqlx معادل rollback exception در `transaction.atomic()` هستند و چرا `?` rollback point است؟
۳. `FOR UPDATE` کدام interleaving دو transfer را جلوگیری می‌کند و کدام line migration حتی در نبود check محافظ است؟
۴. چرا create_account بدون transaction و transfer با transaction درست است؟
۵. `sqlx migrate revert` با `.down.sql` و row migration چه می‌کند و run بعدی چه می‌کند؟
۶. چرا testهای این درس بدون `#[serial]` امن‌اند، برخلاف دو درس قبل؟
