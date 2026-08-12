# فاز ۴ — Backend پیشرفته و طراحی سیستم

فاز سه یک backend قابل‌استفاده ساخت. اینجا واژه‌ها و ابزارهای scale را به آن وصل می‌کنی: cache، rate limit، job پس‌زمینه، observability و ایده‌هایی مثل CAP، load balancing، idempotency و distributed locking. هر مفهوم کنار کدی می‌آید که واقعاً به آن نیاز دارد، نه در یک سخنرانی جدا.

1. **Caching با Redis** — cache-aside، TTL و invalidation
2. **Rate limiting و backpressure** — token bucket و رفتار سرویس زیر فشار
3. **Background job و message queue** — `SKIP LOCKED` و brokerها
4. **gRPC و GraphQL** — دو قرارداد متفاوت برای API
5. **Observability** — log ساخت‌یافته و metric
6. **مبانی طراحی سیستم** — CAP، scaling، load balancer، idempotency و lock توزیع‌شده
7. **Deployment و عملیات** — Docker، CI، config و secret
8. **Performance و profiling** — benchmark و flamegraph

پس از پایان این فاز، [TaskForge](../capstone-taskforge/README.fa.md) جایی است که قطعه‌ها را کنار هم می‌گذاری.
