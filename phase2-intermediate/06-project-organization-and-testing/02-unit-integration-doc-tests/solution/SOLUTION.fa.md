# پاسخ تشریحی

unit test می‌تواند helper خصوصی `round_to_one_decimal` را آزمایش کند، اما integration test باید فقط `celsius_to_fahrenheit` عمومی را import کند. تلاش برای import helper از crate root معمولاً به unresolved import یا private item می‌رسد؛ در هر دو حالت می‌گوید این نام بخشی از API عمومی نیست.

doc fence روی تابع عمومی یک تست واقعی می‌سازد. `# use ...` هنگام compile حضور دارد ولی rustdoc آن را از نمایش پنهان می‌کند تا مثال روی نکته‌ی اصلی متمرکز بماند. حذف `#` رفتار تست را عوض نمی‌کند، اما خط setup را به خواننده نشان می‌دهد.

این تفکیک عمدی است: unit test correctness جزئیات را می‌سنجد؛ integration test تجربه‌ی crate وابسته و visibility/re-exportها را؛ doc test تضمین می‌کند مثال مستند با API جدید کهنه نشود.
