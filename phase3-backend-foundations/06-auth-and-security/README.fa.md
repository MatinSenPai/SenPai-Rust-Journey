# ماژول ۶ — احراز هویت و امنیت

APIهای قبلی باز بودند: هرکس به آن‌ها می‌رسید می‌توانست فراخوانی‌شان کند. این ماژول دو بخش ضروری یک سیستم واقعی را می‌سازد: اثبات password بدون ذخیره‌کردن آن، و اثبات هویت هر درخواست بعدی بدون فرستادن دوبارهٔ credentials.

1. [۰۱ — هش‌کردن password با `argon2`](01-password-hashing-argon2/README.fa.md): چرا plaintext را نگه نمی‌داریم، salt دقیقاً با چه حمله‌ای مقابله می‌کند، و چرا memory-hard بودن Argon2 مهم است.
2. [۰۲ — JWT و middleware در `tower`](02-jwt-and-tower-middleware/README.fa.md): JWT در واقع چیست، در login چگونه صادر می‌شود و `axum` چگونه در middleware آن را تأیید می‌کند.

تا پایان ماژول، جریان login واقعی را می‌سازی: password را hash و verify می‌کنی، سپس tokenی صادر و برای routeهای محافظت‌شده بررسی می‌کنی.
