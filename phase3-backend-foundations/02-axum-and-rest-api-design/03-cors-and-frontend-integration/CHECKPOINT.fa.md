# ایست بازرسی

۱. preflight از `https://evil.example.com` برای origin مجاز `https://anime.example.com` چه status و کدام header غایب می‌گیرد؟ چرا 403 نیست و چه کسی و چه زمانی صفحه‌ی بدخواه را متوقف می‌کند؟
۲. کدام‌یک preflight می‌گیرند و چرا: GET ساده، POST با `application/json` یا GET با `Authorization: Bearer ...`؟
۳. با نبود route `OPTIONS`، چه کسی request preflight را پاسخ می‌دهد و ترتیب layer و router چه چیزی را نشان می‌دهد؟
۴. ترکیب `Any` origin و `allow_credentials(true)` چه حمله‌ای را ممکن می‌کند؟
۵. wildcard در production بدون cookie auth چه چیزی را همچنان آشکار می‌کند و چرا origin دقیق انتخاب درست است؟
