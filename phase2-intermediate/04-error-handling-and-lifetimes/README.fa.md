# ۰۴ — مدیریت خطاها و طول‌عمرها (Error handling & lifetimes)

تو فاز ۱ یاد گرفتی که به جای استفاده از null/استثناها (exceptions) از `Option` و `Result` استفاده کنی. این ماژول این ابزارها رو برای استفاده‌ی واقعی و عملیاتی به بلوغ می‌رسونه: چطوری نوع‌های خطای (error types) سفارشیِ خودت رو بنویسی تا فراخواننده‌های تابع بتونن روشون pattern-match کنن، چطوری از `thiserror`/`anyhow` — که تقریباً تو تمام پایگاه‌های کد (codebase) زبان Rust برای کاهش کدهای تکراری (boilerplate) مدیریت خطاها حضور دارن — استفاده کنی، و در نهایت مفاهیم پایه‌ایِ «طول‌عمرها (lifetimes)» یعنی همون قوانینی که تضمین می‌کنن ارجاع‌ها (references) هیچ‌وقت به یه دیتای پاک‌شده و نامعتبر اشاره نکنن رو یاد می‌گیری.

1. [نوع‌های خطای سفارشی](01-custom-error-types/README.md)
2. [استفاده از `thiserror` و `anyhow`](02-thiserror-and-anyhow/README.md)
3. [مبانی طول‌عمر و elision](03-lifetime-basics-and-elision/README.md)