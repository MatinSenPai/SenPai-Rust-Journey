# راه‌حل

`matches!(status, Status::Cancelled)` شکل کوتاه matchی است که فقط boolean می‌خواهد. در `match status` با ورودی `&Status`، field عددی نیز ارجاع می‌شود؛ `*latest_chapter` مقدار `u32: Copy` را می‌خواند.

struct دارای `latest_chapter: Option<u32>`، `since_chapter`, `total_chapters` و `is_cancelled` می‌تواند چند field ناسازگار را هم‌زمان set کند یا هیچ وضعیت معناداری نداشته باشد. enum اصل «حالت نامعتبر را غیرقابل نمایش کن» را اجرا می‌کند.
