# ایست بازرسی

1. یک JWT را بدون secret در jwt.io باز کن. چه چیزهایی خواندنی‌اند و چه چیزی بدون secret قابل‌تأیید نیست؟ این دقیقاً چرا «signed، نه encrypted» است؟
2. چرا header گمشده، token خراب، secret اشتباه و token منقضی همگی باید از دید client یک `401` باشند؟
3. اگر `Claims` اصلاً `exp` نداشت و فقط signature بررسی می‌شد، token دزدیده‌شده چه خطری داشت؟
4. `require_auth` را با `taskforge-api::auth::require_bearer_token` مقایسه کن. handler پایین‌دستِ هرکدام چه چیزی دربارهٔ caller می‌داند؟
5. چرا تست با `app().oneshot(...)` از فراخوانی مستقیم `require_auth` بهتر است، با توجه به `Request` و `Next` واقعی؟
6. طرح یک denylist برای revoke زودهنگام JWT چیست و چرا بخشی از مزیت stateless بودن را پس می‌گیرد؟
