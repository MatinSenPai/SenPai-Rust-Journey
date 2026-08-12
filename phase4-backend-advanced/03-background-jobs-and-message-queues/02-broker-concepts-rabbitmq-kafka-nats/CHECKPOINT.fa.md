# ایست بازرسی

1. پنج سیستم مستقل باید event سفارش را با سرعت خودشان بخوانند و سیستم ششم بعداً از گذشته شروع کند. کدام broker و کدام قابلیت دقیقش مناسب است؟
2. برای resize image با یک worker برای هر job، priority و dead-letter بعد از سه failure، چه انتخابی می‌کنی؟ آیا هزینهٔ broker تازه با ADR-0002 توجیه دارد؟
3. در at-least-once delivery، consumer چگونه باید پردازشِ دوباره را بی‌خطر کند؟ یک operation غیر-idempotent و راه امن‌کردنش مثال بزن.
4. ordering فقط در Kafka partition چه زمانی bug می‌سازد، اگر consumer اشتباهاً ترتیب کل topic را فرض کند؟
5. triggerهای خروج از Postgres queue چیستند و برای multi-datacenter، fan-out واقعی و throughputِ فراتر از Postgres، کدام ابزار محتمل‌تر است؟
