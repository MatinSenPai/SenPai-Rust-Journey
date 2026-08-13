# ایست بازرسی

1. چرا برای password می‌گوییم «hash کن، encrypt نکن» اما برای شماره‌کارتی که بعداً باید بخوانیم encryption درست است؟
2. salt تصادفیِ جداگانه برای هر password دقیقاً چگونه حملهٔ rainbow table را بی‌اثر می‌کند؟
3. چرا `first != second` برای دو hash از یک password رفتار مطلوب است، نه bug؟
4. memory-hard بودن Argon2 چه مزیت اقتصادیِ GPU/ASIC را از مهاجم می‌گیرد که bcryptِ صرفاً CPU-slow نمی‌گیرد؟
5. چرا `verify_password` برای hash خراب `false` می‌دهد، نه panic یا خطایی متفاوت برای کاربر؟
6. همراه‌بودن `m=...,t=...,p=...` با PHC string چطور افزایش هزینه در آینده را بدون migration غیرممکن ممکن می‌کند؟
