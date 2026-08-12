# ایست بازرسی

۱. چرا test ابتدا table و row tracking version 1 را پاک می‌کند، با وجود persistشدن `_sqlx_migrations` میان اجراها؟
۲. افزودن `0002_add_widget_price.sql` پس از اجرای 0001 چه می‌کند؟ ویرایش 0001 اجراشده چه خطایی می‌دهد؟
۳. embedشدن SQL در binary در deployment چه فایده‌ای نسبت به خواندن فایل runtime دارد؟
۴. tracking sqlx و `django_migrations` چه اشتراکی دارند و تفاوت واقعی workflow با `makemigrations` چیست؟
۵. `query_scalar` و `query_as` برای چه شکل queryهایی مناسب‌اند؟
۶. چرا prefix table، collision مربوط به `_sqlx_migrations` بین دو `migrate!` نامرتبط را حل نمی‌کند؟
۷. چرا حذف tag serial از فقط یک test هم suite را unsafe می‌کند؟
