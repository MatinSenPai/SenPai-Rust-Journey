# ایست بازرسی

۱. چرا `parse_request` قبل از هر کار `std::str::from_utf8(raw)` را چک می‌کند؟ اگر byteها را text معتبر فرض کند، در مراحل بعد چه می‌شکند؟

۲. چهار variant جدا در `HttpParseError` نسبت به `ParseFailed` عمومی چه مزیتی برای caller و `assert!(matches!(...))` دارند؟ تست‌های request line و header malformed را مقایسه کن.

۳. چرا نام headerها هنگام parsing به lowercase تبدیل نمی‌شوند و مقایسه‌ی `eq_ignore_ascii_case` فقط در lookup رخ می‌دهد؟

۴. bodyای بساز که `self.body.len()` و `self.body.chars().count()` در آن فرق دارند. اگر `Content-Length` تعداد character باشد، clientی که دقیقاً همان byteها را می‌خواند چه خطایی می‌کند؟

۵. برای تبدیل `Some("status=watching&page=2")` به چیزی مثل `QueryDict` یا `HashMap<String, String>` چه مراحل parsing و URL decoding لازم است؟ تکراری‌بودن key چه اثری بر انتخاب نوع دارد؟
