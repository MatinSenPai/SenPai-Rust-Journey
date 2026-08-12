# ۰۲ — Rate limiting و backpressure

caching database را از کار تکراری محافظت می‌کند. rate limiting خودِ service را از کار بیش‌ازحد محافظت می‌کند: client، bug یا attacker نباید endpoint را سریع‌تر از توان امن سرویس بکوبد. **Backpressure** یک لایه پایین‌تر می‌پرسد وقتی درخواست از ظرفیت بیشتر است چه می‌کنیم: زود رد کنیم، با سقف queue کنیم یا caller را کند کنیم؛ نه اینکه queue بی‌نهایت و memory تمام‌شده و timeout زنجیره‌ای بسازیم.

1. [Token bucket و `tower::limit`](01-token-bucket-and-tower-limit/README.fa.md)
