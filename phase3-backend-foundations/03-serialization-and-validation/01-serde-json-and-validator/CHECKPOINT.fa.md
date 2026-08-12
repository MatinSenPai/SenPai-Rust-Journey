# ایست بازرسی

۱. missing `title` و `rating: 11` در کدام خط `parse_review` و چرا با variantهای متفاوت شکست می‌خورند؟ چرا serde هرگز بازه‌ی ۱ تا ۱۰ را نمی‌فهمد؟
۲. validator چگونه بدون annotation مانند `required=False` متوجه می‌شود rule length را روی `comment: None` اجرا نکند؟
۳. حذف `.sort()` از `validation_summary` چه flaky failure مشخصی در assertionهای `messages[0]` و `messages[1]` می‌سازد؟
۴. مزیت قابل‌تست‌بودن مستقل parse و validation چیست و DRF تک‌مرحله‌ای چه output یکپارچه‌ای را آسان‌تر می‌دهد؟
۵. rule بین title و comment را با field attribute نمی‌شود نوشت؛ struct-level/custom validation چه شکلی دارد؟
