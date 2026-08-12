# ایست بازرسی

1. چرا `live_postgres()` هم `PgPool` و هم `ContainerAsync<Postgres>` برمی‌گرداند؟ اگر `let (pool, _) = ...` بنویسی چه می‌شود؟
2. چرا این testها مثل درس‌های database مشترک، پیش از شروع `DELETE` نمی‌زنند؟
3. چرا `get_host_port_ipv4(5432)` معمولاً ۵۴۳۲ نیست؟ اجرای موازی با host port ثابت چه مشکلی دارد؟
4. `#[ignore]` دقیقاً چه چیزی را skip می‌کند و چه چیزی را همچنان compile/type-check می‌کند؟
5. برای تست Postgres واقعیِ `WidgetStore` درس قبل، مستقیماً testcontainers اضافه می‌کنی یا اول طراحی store را تغییر می‌دهی؟ چرا؟
